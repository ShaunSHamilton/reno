use std::error::Error;

use btleplug::{
    api::{CharPropFlags, Characteristic, Peripheral as _, WriteType},
    platform::Peripheral,
};
use serde::{Deserialize, Serialize};

use crate::data::{
    DataType, DataView, RX_CHARACTERISTIC, RX_SERVICE, TX_CHARACTERISTIC, TX_SERVICE,
};

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

pub async fn get_levels(
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

pub async fn get_cell_volts(
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

pub async fn get_temps(
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

pub enum RequestType {
    GetLevels,
    GetCellVolts,
    GetTemps,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum PeripheralError {
    Fail,
    UnableToUseAdapter,
}
