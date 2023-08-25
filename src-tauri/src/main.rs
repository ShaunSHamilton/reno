// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use btleplug::api::bleuuid::BleUuid;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::api::{CentralEvent, CharPropFlags, Characteristic};
use btleplug::platform::{Adapter, Manager, Peripheral};
use data::{
    Data, DataType, DataView, RX_CHARACTERISTIC, RX_SERVICE, TX_CHARACTERISTIC, TX_SERVICE,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::iter::Iterator;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Manager as _;
use tokio::{select, time};
use tokio_stream::StreamExt;

mod data;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn greet(name: &str) -> Result<String, ()> {
    Ok(format!("Hello, {}! You've been greeted from Rust!", name))
}

#[tauri::command]
async fn refresh_bluetooth_adapters(
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
async fn connect_to_adapter(
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
async fn search_for_peripherals(
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
                    _ => {
                        println!("Other event: {:?}", event);
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
async fn connect_to_peripheral(
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
async fn request_single_event(
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
async fn request_multiple_events() -> Result<(), String> {
    Ok(())
}

async fn subscribe_to_service(peripheral: &Peripheral) -> Result<Characteristic, Box<dyn Error>> {
    let services = peripheral.services();

    let rx_service = services
        .into_iter()
        .find(|s| s.uuid.to_string() == RX_SERVICE)
        .unwrap();
    let rx_char = rx_service
        .characteristics
        .into_iter()
        .find(|c| {
            c.uuid.to_string() == RX_CHARACTERISTIC
                && c.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE)
        })
        .expect("RX_CHARACTERISTIC not found");

    let services = peripheral.services();
    let tx_service = services
        .into_iter()
        .find(|s| s.uuid.to_string() == TX_SERVICE)
        .unwrap();
    let tx_char = tx_service
        .characteristics
        .into_iter()
        .find(|c| {
            c.uuid.to_string() == TX_CHARACTERISTIC && c.properties.contains(CharPropFlags::NOTIFY)
        })
        .expect("TX_CHARACTERISTIC not found");

    // Start notifications on TX_CHARACTERISTIC
    peripheral.subscribe(&tx_char).await?;
    Ok(rx_char)
}

fn handle_data(payload: DataView, request_type: &RequestType) -> DataType {
    match request_type {
        RequestType::GetLevels => {
            let current = f32::from(payload.get_int16(0)) / 100.0;
            let volt = f32::from(payload.get_uint16(2)) / 10.0;
            let charge_level = payload.get_uint32(4) as f32 / 1000.0;
            let capacity = payload.get_uint32(8) as f32 / 1000.0;

            // println!(
            //     "Current: {:?}A, Voltage: {:?}V, Charge Level: {:?}%, Capacity: {:?}Ah",
            //     current, volt, charge_level, capacity
            // );
            DataType::Levels {
                current,
                volt,
                charge_level,
                capacity,
            }
        }
        RequestType::GetCellVolts => {
            let num_cells = payload.get_uint16(0);
            let volts = (0..num_cells)
                .map(|i| f32::from(payload.get_uint16((1 + i) as usize * 2)) / 10.0)
                .collect::<Vec<_>>();
            // println!("Cell voltages: {:?}", volts);
            DataType::CellVolts { cell_volts: volts }
        }
        RequestType::GetTemps => {
            let num_sensors = payload.get_uint16(0);
            let mut temps = Vec::new();
            for i in 0..num_sensors {
                let temp = f32::from(payload.get_int16((1 + i) as usize * 2)) / 10.0;
                temps.push(temp);
            }
            // println!("Temperatures: {:?}", temps);
            DataType::Temps { temps }
        }
    }
}

async fn get_levels(
    peripheral: &Peripheral,
    rx_char: &Characteristic,
) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0u8; 8];
    let mut view = DataView::new(&mut buffer);
    view.set_uint16(0, 0x3003);
    view.set_uint16(2, 0x13b2);
    view.set_uint16(4, 0x0006);
    view.set_uint16(6, 0x654a);

    peripheral
        .write(rx_char, &buffer, WriteType::WithResponse)
        .await?;
    Ok(())
}

async fn get_cell_volts(
    peripheral: &Peripheral,
    rx_char: &Characteristic,
) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0u8; 8];
    let mut view = DataView::new(&mut buffer);
    view.set_uint16(0, 0x3003);
    view.set_uint16(2, 0x1388);
    view.set_uint16(4, 0x0011);
    view.set_uint16(6, 0x0549);

    peripheral
        .write(rx_char, &buffer, WriteType::WithResponse)
        .await?;
    Ok(())
}

async fn get_temps(
    peripheral: &Peripheral,
    rx_char: &Characteristic,
) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0u8; 8];
    let mut view = DataView::new(&mut buffer);
    view.set_uint16(0, 0x3003);
    view.set_uint16(2, 0x1399);
    view.set_uint16(4, 0x0005);
    view.set_uint16(6, 0x5543);

    peripheral
        .write(rx_char, &buffer, WriteType::WithResponse)
        .await?;
    Ok(())
}

enum RequestType {
    GetLevels,
    GetCellVolts,
    GetTemps,
}

#[derive(Debug, Deserialize, Serialize)]
enum PeripheralError {
    Fail,
    UnableToUseAdapter,
}

#[derive(Debug, Deserialize, Serialize)]
enum AdapterError {
    Fail,
}

struct AppState {
    adapters: Mutex<Vec<Adapter>>,
    chosen_adapter: Mutex<Option<Adapter>>,
    peripherals: Mutex<Vec<Peripheral>>,
    chosen_peripheral: Mutex<Option<Peripheral>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            adapters: Mutex::new(Vec::new()),
            chosen_adapter: Mutex::new(None),
            peripherals: Mutex::new(Vec::new()),
            chosen_peripheral: Mutex::new(None),
        }
    }
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            refresh_bluetooth_adapters,
            connect_to_adapter,
            search_for_peripherals,
            connect_to_peripheral,
            request_single_event,
            request_multiple_events
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
