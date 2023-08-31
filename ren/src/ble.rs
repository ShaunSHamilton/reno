use btleplug::api::bleuuid::BleUuid;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::api::{CentralEvent, CharPropFlags, Characteristic};
use btleplug::platform::{Adapter, Manager, Peripheral};
use serde_json;
use std::error::Error;
use std::iter::Iterator;
use std::path::Path;
use std::time::Duration;
use tokio::{select, time};
use tokio_stream::StreamExt;

use crate::clapper::Args;
use crate::data::{
    Data, DataType, DataView, RX_CHARACTERISTIC, RX_SERVICE, TX_CHARACTERISTIC, TX_SERVICE,
};

pub async fn get_bt_adapter() -> Adapter {
    let manager = Manager::new().await.unwrap();
    let adapters = manager.adapters().await.unwrap();
    adapters
        .into_iter()
        .nth(0)
        .expect("Bluetooth adapter to be turned on")
}

pub async fn scan_for_devices(central: &Adapter) {
    central
        .start_scan(ScanFilter::default())
        .await
        .expect("Adapter closed unexpectedly");
}

pub async fn handle_device_events(central: &Adapter, args: &Args) -> Result<(), Box<dyn Error>> {
    let mut events = central.events().await?;
    let mut connected_to_first_discovered = false;
    loop {
        select! {
            event = events.next() => {
                match event {
                    Some(CentralEvent::DeviceDiscovered(id)) => {
                        println!("DeviceDiscovered: {:?}", id);
                        if let Some(peripheral_id) = args.peripheral_id.as_ref() {
                            if &id.to_string() == peripheral_id {
                                let peripheral = central.peripheral(&id).await?;
                                peripheral.connect().await?;
                                read_data(&peripheral, &args).await?;
                            }
                        } else if let Some(peripheral_name) = args.peripheral_name.as_ref() {
                            let peripheral = central.peripheral(&id).await?;
                            if let Some(peripheral_properties) = peripheral.properties().await? {
                                if &peripheral_properties.local_name.unwrap_or_default() == peripheral_name {
                                    peripheral.connect().await?;
                                    read_data(&peripheral, &args).await?;
                                }
                            }
                        } else if !connected_to_first_discovered {
                            println!("No device name specified. Connecting to first discovered device.");
                            let peripheral = central.peripheral(&id).await?;
                            peripheral.connect().await?;
                            connected_to_first_discovered = true;
                            read_data(&peripheral, &args).await?;
                        }
                    }
                    Some(CentralEvent::DeviceConnected(id)) => {
                        println!("DeviceConnected: {:?}", id);
                    }
                    Some(CentralEvent::DeviceDisconnected(id)) => {
                        println!("DeviceDisconnected: {:?}", id);
                    }
                    Some(CentralEvent::ManufacturerDataAdvertisement { id, manufacturer_data }) => {
                        println!("ManufacturerDataAdvertisement: {:?}, {:?}", id, manufacturer_data);
                    }
                    Some(CentralEvent::ServiceDataAdvertisement { id, service_data }) => {
                        println!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
                    }
                    Some(CentralEvent::ServicesAdvertisement { id, services }) => {
                        let services: Vec<String> = services.into_iter().map(|s| s.to_short_string()).collect();
                        println!("ServicesAdvertisement: {:?}, {:?}", id, services);
                    }
                    _ => {
                        println!("Other event: {:?}", event);
                    }
                }
            },
            //  Listen for Ctrl-C and quit the program.
            _ = tokio::signal::ctrl_c() => {
                println!("Ctrl-C received, quitting...");
                break;
            }
        }
    }
    Ok(())
}

pub async fn read_data(peripheral: &Peripheral, args: &Args) -> Result<(), Box<dyn Error>> {
    // Prioritise temps > levels > cell volts
    // Do not get data if we have not received a response from the previous request

    peripheral.discover_services().await?;

    let rx_char = subscribe_to_service(peripheral).await?;

    let mut notification_stream = peripheral.notifications().await?;

    let mut count: u64 = 0;
    let mut request_type = RequestType::GetLevels;
    let interval = args.inverval;
    loop {
        select! {
            notification = notification_stream.next() => {
                let notification = notification.unwrap();
                let mut packet = notification.value;
                let payload = DataView::new(&mut packet[3..]);

                let system_time = std::time::SystemTime::now();
                let timestamp = system_time.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
                let data_type = handle_data(payload, &request_type);
                let data = Data {
                    data: data_type,
                    timestamp,
                };


                if let Some(logs) = args.logs.as_ref() {
                    if count % (interval * 5) == 0 {
                        println!("{data:?}");
                    }
                    let file_path = Path::new(&logs);
                    if let Err(e) = save_data_to_file(data, file_path).await {
                        println!("Error saving data to file: {:?}", e);
                    }
                } else {
                    println!("{data:?}");
                }

            },
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
            },
            //  Listen for Ctrl-C and quit the program.
            _ = tokio::signal::ctrl_c() => {
                println!("Ctrl-C received, quitting...");
                break;
            }
        }
    }
    peripheral.disconnect().await?;
    Ok(())
}

/// Pushes the data as a JSON array element to the current directory in a file called `bt-data.json`
pub async fn save_data_to_file(data: Data, file_path: &Path) -> Result<(), Box<dyn Error>> {
    // Create file if it does not exist, and add `[]` to it if not. Otherwise, just open.
    if !tokio::fs::metadata(file_path).await.is_ok() {
        tokio::fs::write(file_path, "[]").await?;
    }

    let file_str = tokio::fs::read_to_string("bt-data.json").await?;
    let mut current_json: Vec<Data> = serde_json::from_str(&file_str)?;
    current_json.push(data);
    let new_json = serde_json::to_string(&current_json)?;

    tokio::fs::write("bt-data.json", new_json).await?;

    Ok(())
}

pub enum RequestType {
    GetLevels,
    GetCellVolts,
    GetTemps,
}

pub fn handle_data(payload: DataView, request_type: &RequestType) -> DataType {
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

pub async fn subscribe_to_service(
    peripheral: &Peripheral,
) -> Result<Characteristic, Box<dyn Error>> {
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
