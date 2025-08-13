use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use btleplug::{
    api::{bleuuid::BleUuid, Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter},
    platform::{Manager, Peripheral},
};
use tauri::Manager as _;
use tokio::{select, time};
use tokio_stream::StreamExt;

use crate::{
    adapter::AdapterError,
    config::save_data_to_file,
    data::{Data, DataView},
    peripheral::{
        get_cell_volts, get_levels, get_temps, handle_data, subscribe_to_service, PeripheralError,
        RequestType,
    },
    state::AppState,
};

#[tauri::command]
pub async fn refresh_bluetooth_adapters(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, AdapterError> {
    let mut adapter_names = Vec::new();
    if let Ok(manager) = Manager::new().await {
        if let Ok(adapters) = manager.adapters().await {
            for adapter in adapters {
                if let Ok(adapter_name) = adapter.adapter_info().await {
                    adapter_names.push(adapter_name);
                }
                state.adapters.lock().unwrap().push(adapter);
            }
        }
    } else {
        return Err(AdapterError::Fail);
    }
    Ok(adapter_names)
}

#[tauri::command]
pub async fn connect_to_adapter(
    state: tauri::State<'_, AppState>,
    name: &str,
) -> Result<(), AdapterError> {
    let adapters = state.adapters.lock().unwrap().to_vec();
    for adapter in adapters {
        if let Ok(adapter_name) = adapter.adapter_info().await {
            if adapter_name == name {
                if let Err(_scan_err) = adapter.start_scan(ScanFilter::default()).await {
                    return Err(AdapterError::Fail);
                } else {
                    *state.chosen_adapter.lock().unwrap() = Some(adapter);
                    return Ok(());
                }
            }
        }
    }
    Err(AdapterError::Fail)
}

#[tauri::command]
pub async fn search_for_peripherals(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), PeripheralError> {
    let mut peripherals = Vec::new();
    let mut adapter = state.chosen_adapter.lock().unwrap().to_owned();

    let stop_searching = Arc::new(Mutex::new(false));
    let stop_searching_clone = stop_searching.clone();
    let _event_id = app.listen_global("stop-searching", move |_event| {
        println!("search stopped");
        *stop_searching_clone.lock().unwrap() = true;
    });

    if let Some(adapter) = adapter.as_mut() {
        println!("adapter is scanning");
        adapter.start_scan(ScanFilter::default()).await.unwrap();
        println!("adapter not scanning");
        let mut events = adapter.events().await.unwrap();
        println!("eventing");
        loop {
            if *stop_searching.lock().unwrap() {
                println!("search stopped");
                adapter.stop_scan().await.unwrap();
                break;
            }
            if let Some(event) = events.next().await {
                match event {
                    CentralEvent::DeviceDiscovered(peripheral_id) => {
                        if peripherals
                            .iter()
                            .all(|p: &Peripheral| p.id() != peripheral_id)
                        {
                            let peripheral = adapter.peripheral(&peripheral_id).await.unwrap();
                            peripherals.push(peripheral);
                            app.emit_all(
                                "DeviceDiscovered",
                                peripherals
                                    .iter()
                                    .map(|p: &Peripheral| p.id().to_string())
                                    .collect::<Vec<_>>(),
                            )
                            .unwrap();
                        }
                    }
                    CentralEvent::DeviceUpdated(peripheral_id) => {
                        if peripherals.iter().all(|p| p.id() != peripheral_id) {
                            let peripheral = adapter.peripheral(&peripheral_id).await.unwrap();
                            peripherals.push(peripheral);
                            app.emit_all(
                                "DeviceDiscovered",
                                peripherals
                                    .iter()
                                    .map(|p: &Peripheral| p.id().to_string())
                                    .collect::<Vec<_>>(),
                            )
                            .unwrap();
                        }
                    }
                    CentralEvent::DeviceConnected(peripheral_id) => {
                        let peripheral = adapter.peripheral(&peripheral_id).await.unwrap();
                        app.emit_all("DeviceConnected", peripheral_id.to_string())
                            .unwrap();
                        // Remove peripheral from list if it is already there
                        peripherals.retain(|p| p.id() != peripheral.id());
                        break;
                    }
                    CentralEvent::DeviceDisconnected(id) => {
                        println!("DeviceDisconnected: {:?}", id);
                        peripherals.retain(|p| p.id() != id);
                    }
                    CentralEvent::ManufacturerDataAdvertisement {
                        id,
                        manufacturer_data,
                    } => {
                        println!(
                            "ManufacturerDataAdvertisement: {:?}, {:?}",
                            id, manufacturer_data
                        );
                    }
                    CentralEvent::ServiceDataAdvertisement { id, service_data } => {
                        println!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
                    }
                    CentralEvent::ServicesAdvertisement { id, services } => {
                        let services: Vec<String> =
                            services.into_iter().map(|s| s.to_short_string()).collect();
                        println!("ServicesAdvertisement: {:?}, {:?}", id, services);
                    }
                }
            } else {
                println!("no event");
            }
        }
    } else {
        return Err(PeripheralError::UnableToUseAdapter);
    }

    *state.peripherals.lock().unwrap() = peripherals;

    Ok(())
}

#[tauri::command]
pub async fn connect_to_peripheral(
    state: tauri::State<'_, AppState>,
    id: &str,
) -> Result<(), PeripheralError> {
    let peripherals = state.peripherals.lock().unwrap().to_owned();
    for peripheral in peripherals {
        if peripheral.id().to_string() == id {
            if let Err(_connect_err) = peripheral.connect().await {
                return Err(PeripheralError::Fail);
            } else {
                peripheral.discover_services().await.unwrap();
                *state.chosen_peripheral.lock().unwrap() = Some(peripheral);
                return Ok(());
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn request_single_event(
    state: tauri::State<'_, AppState>,
) -> Result<String, PeripheralError> {
    let peripheral = state.chosen_peripheral.lock().unwrap().to_owned().unwrap();
    let rx_char = subscribe_to_service(&peripheral).await.unwrap();

    let mut notification_stream = peripheral.notifications().await.unwrap();

    get_levels(&peripheral, &rx_char).await.unwrap();
    if let Some(notification) = notification_stream.next().await {
        let mut packet = notification.value;
        let payload = DataView::new(&mut packet[3..]);

        let system_time = std::time::SystemTime::now();
        let timestamp = system_time
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let request_type = RequestType::GetLevels;
        let data_type = handle_data(payload, &request_type);
        let data = Data {
            data: data_type,
            timestamp,
        };

        let data_json = serde_json::to_string(&data).unwrap();
        return Ok(data_json);
    } else {
        return Err(PeripheralError::Fail);
    }
}

#[tauri::command]
pub async fn request_multiple_events(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let path_resolver = app.path_resolver();
    let data_dir = path_resolver.app_data_dir().unwrap();

    let peripheral = state.chosen_peripheral.lock().unwrap().to_owned().unwrap();
    let rx_char = subscribe_to_service(&peripheral).await.unwrap();

    let mut notification_stream = peripheral.notifications().await.unwrap();

    let stop_recording = Arc::new(Mutex::new(false));
    let stop_recording_clone = stop_recording.clone();
    let _event_id = app.listen_global("stop-recording", move |_event| {
        println!("Recording stopped");
        *stop_recording_clone.lock().unwrap() = true;
    });

    let mut count: u64 = 0;
    let mut request_type = RequestType::GetLevels;
    let interval = 3;
    loop {
        if stop_recording.lock().unwrap().to_owned() {
            println!("Stopping recording");
            break;
        }
        select! {
            notification = notification_stream.next() => {
                // get_levels(&peripheral, &rx_char).await.unwrap();
                if let Some(notification) = notification {
                    let mut packet = notification.value;
                    let payload = DataView::new(&mut packet[3..]);

                    let system_time = std::time::SystemTime::now();
                    let timestamp = system_time
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();

                    let data_type = handle_data(payload, &request_type);
                    let data = Data {
                        data: data_type,
                        timestamp,
                    };
                    app.emit_all("Data", data.clone()).unwrap();
                    // let data_json = serde_json::to_string(&data).unwrap();
                    let file_path = data_dir.join("data").join("bt-data.json");
                    if let Err(e) = save_data_to_file(data, file_path).await {
                        println!("Error saving data: {:?}", e);
                    }

                }

            }
            _ = time::sleep(Duration::from_secs(interval)) => {
                count += 1;

                if count % (interval * 20) == 0 {
                    request_type = RequestType::GetTemps;
                    if let Err(e) = get_temps(&peripheral, &rx_char).await {
                        println!("Error getting temps: {:?}", e);
                    }
                } else if count % (interval * 10) == 0 {
                    request_type = RequestType::GetCellVolts;
                    if let Err(e) = get_cell_volts(&peripheral, &rx_char).await {
                        println!("Error getting cell volts: {:?}", e);
                    }
                } else {
                    request_type = RequestType::GetLevels;
                    if let Err(e) = get_levels(&peripheral, &rx_char).await {
                        println!("Error getting levels: {:?}", e);
                    }
                }
            }
        }
    }
    Ok(())
}
