use std::fmt::Debug;
use std::fmt::Formatter;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub data: DataType,
    pub timestamp: u128,
}

impl Debug for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.data {
            DataType::Levels {
                current,
                volt,
                charge_level,
                capacity: _,
            } => {
                let watts = current * volt;
                write!(
                    f,
                    "Power: {}; Current: {}; Voltage: {}; SoC: {};",
                    watts, current, volt, charge_level
                )
            }
            DataType::CellVolts { cell_volts } => {
                write!(f, "Cell Voltages: {:?}", cell_volts)
            }
            DataType::Temps { temps } => {
                write!(f, "Temperature: {:?}", temps)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DataType {
    Levels {
        current: f32,
        volt: f32,
        charge_level: f32,
        capacity: f32,
    },
    CellVolts {
        cell_volts: Vec<f32>,
    },
    Temps {
        temps: Vec<f32>,
    },
}

pub struct DataView<'a> {
    buffer: &'a mut [u8],
}

impl<'a> DataView<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        DataView { buffer }
    }

    pub fn set_uint16(&mut self, offset: usize, value: u16) {
        self.buffer[offset] = (value >> 8) as u8;
        self.buffer[offset + 1] = value as u8;
    }

    pub fn get_int16(&self, offset: usize) -> i16 {
        ((self.buffer[offset] as i16) << 8) | self.buffer[offset + 1] as i16
    }

    pub fn get_uint16(&self, offset: usize) -> u16 {
        ((self.buffer[offset] as u16) << 8) | self.buffer[offset + 1] as u16
    }

    pub fn get_uint32(&self, offset: usize) -> u32 {
        ((self.buffer[offset] as u32) << 24)
            | ((self.buffer[offset + 1] as u32) << 16)
            | ((self.buffer[offset + 2] as u32) << 8)
            | self.buffer[offset + 3] as u32
    }
}

pub const CHARACTERISTICS: [&str; 19] = [
    "00002a00-0000-1000-8000-00805f9b34fb",
    "00002a01-0000-1000-8000-00805f9b34fb",
    "00002a04-0000-1000-8000-00805f9b34fb",
    "00002a23-0000-1000-8000-00805f9b34fb",
    "00002a24-0000-1000-8000-00805f9b34fb",
    "00002a25-0000-1000-8000-00805f9b34fb",
    "00002a26-0000-1000-8000-00805f9b34fb",
    "00002a27-0000-1000-8000-00805f9b34fb",
    "00002a28-0000-1000-8000-00805f9b34fb",
    "00002a29-0000-1000-8000-00805f9b34fb",
    "00002a2a-0000-1000-8000-00805f9b34fb",
    "00002a50-0000-1000-8000-00805f9b34fb",
    "0000ffd1-0000-1000-8000-00805f9b34fb",
    "0000ffd2-0000-1000-8000-00805f9b34fb",
    "0000ffd3-0000-1000-8000-00805f9b34fb",
    "0000ffd4-0000-1000-8000-00805f9b34fb",
    "0000ffd5-0000-1000-8000-00805f9b34fb",
    "0000fff1-0000-1000-8000-00805f9b34fb",
    "f000ffd1-0451-4000-b000-000000000000",
];

pub const SERVICES: [&str; 5] = [
    "00001800-0000-1000-8000-00805f9b34fb",
    "0000180a-0000-1000-8000-00805f9b34fb",
    "0000ffd0-0000-1000-8000-00805f9b34fb",
    "0000fff0-0000-1000-8000-00805f9b34fb",
    "f000ffd0-0451-4000-b000-000000000000",
];

pub const RX_CHARACTERISTIC: &str = CHARACTERISTICS[12];
pub const TX_CHARACTERISTIC: &str = CHARACTERISTICS[17];

pub const RX_SERVICE: &str = SERVICES[2];
pub const TX_SERVICE: &str = SERVICES[3];

// Characteristic { uuid: 00002a00-0000-1000-8000-00805f9b34fb, service_uuid: 00001800-0000-1000-8000-00805f9b34fb, properties: READ, descriptors: {} },
// Characteristic { uuid: 00002a01-0000-1000-8000-00805f9b34fb, service_uuid: 00001800-0000-1000-8000-00805f9b34fb, properties: READ, descriptors: {} },
// Characteristic { uuid: 00002a04-0000-1000-8000-00805f9b34fb, service_uuid: 00001800-0000-1000-8000-00805f9b34fb, properties: READ, descriptors: {} },
// Characteristic { uuid: 00002a23-0000-1000-8000-00805f9b34fb, service_uuid: 0000180a-0000-1000-8000-00805f9b34fb, properties: READ | WRITE_WITHOUT_RESPONSE | WRITE, descriptors: {} },
// Characteristic { uuid: 00002a24-0000-1000-8000-00805f9b34fb, service_uuid: 0000180a-0000-1000-8000-00805f9b34fb, properties: READ, descriptors: {} },
// Characteristic { uuid: 00002a25-0000-1000-8000-00805f9b34fb, service_uuid: 0000180a-0000-1000-8000-00805f9b34fb, properties: READ, descriptors: {} },
// Characteristic { uuid: 00002a26-0000-1000-8000-00805f9b34fb, service_uuid: 0000180a-0000-1000-8000-00805f9b34fb, properties: READ, descriptors: {} },
// Characteristic { uuid: 00002a27-0000-1000-8000-00805f9b34fb, service_uuid: 0000180a-0000-1000-8000-00805f9b34fb, properties: READ, descriptors: {} },
// Characteristic { uuid: 00002a28-0000-1000-8000-00805f9b34fb, service_uuid: 0000180a-0000-1000-8000-00805f9b34fb, properties: READ, descriptors: {} },
// Characteristic { uuid: 00002a29-0000-1000-8000-00805f9b34fb, service_uuid: 0000180a-0000-1000-8000-00805f9b34fb, properties: READ, descriptors: {} },
// Characteristic { uuid: 00002a2a-0000-1000-8000-00805f9b34fb, service_uuid: 0000180a-0000-1000-8000-00805f9b34fb, properties: READ, descriptors: {} },
// Characteristic { uuid: 00002a50-0000-1000-8000-00805f9b34fb, service_uuid: 0000180a-0000-1000-8000-00805f9b34fb, properties: READ, descriptors: {} },
// Characteristic { uuid: 0000ffd1-0000-1000-8000-00805f9b34fb, service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb, properties: READ | WRITE_WITHOUT_RESPONSE | WRITE, descriptors: {Descriptor { uuid: 00002901-0000-1000-8000-00805f9b34fb, service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb, characteristic_uuid: 0000ffd1-0000-1000-8000-00805f9b34fb }} },
// Characteristic { uuid: 0000ffd2-0000-1000-8000-00805f9b34fb, service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb, properties: READ | NOTIFY, descriptors: {Descriptor { uuid: 00002901-0000-1000-8000-00805f9b34fb, service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb, characteristic_uuid: 0000ffd2-0000-1000-8000-00805f9b34fb }, Descriptor { uuid: 00002902-0000-1000-8000-00805f9b34fb, service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb, characteristic_uuid: 0000ffd2-0000-1000-8000-00805f9b34fb }} },
// Characteristic { uuid: 0000ffd3-0000-1000-8000-00805f9b34fb, service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb, properties: WRITE, descriptors: {Descriptor { uuid: 00002901-0000-1000-8000-00805f9b34fb, service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb, characteristic_uuid: 0000ffd3-0000-1000-8000-00805f9b34fb }} },
// Characteristic { uuid: 0000ffd4-0000-1000-8000-00805f9b34fb, service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb, properties: READ, descriptors: {Descriptor { uuid: 00002901-0000-1000-8000-00805f9b34fb, service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb, characteristic_uuid: 0000ffd4-0000-1000-8000-00805f9b34fb }} },
// Characteristic { uuid: 0000ffd5-0000-1000-8000-00805f9b34fb, service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb, properties: READ | WRITE, descriptors: {Descriptor { uuid: 00002901-0000-1000-8000-00805f9b34fb, service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb, characteristic_uuid: 0000ffd5-0000-1000-8000-00805f9b34fb }} },
// Characteristic { uuid: 0000fff1-0000-1000-8000-00805f9b34fb, service_uuid: 0000fff0-0000-1000-8000-00805f9b34fb, properties: READ | NOTIFY, descriptors: {Descriptor { uuid: 00002901-0000-1000-8000-00805f9b34fb, service_uuid: 0000fff0-0000-1000-8000-00805f9b34fb, characteristic_uuid: 0000fff1-0000-1000-8000-00805f9b34fb }, Descriptor { uuid: 00002902-0000-1000-8000-00805f9b34fb, service_uuid: 0000fff0-0000-1000-8000-00805f9b34fb, characteristic_uuid: 0000fff1-0000-1000-8000-00805f9b34fb }} },
// Characteristic { uuid: f000ffd1-0451-4000-b000-000000000000, service_uuid: f000ffd0-0451-4000-b000-000000000000, properties: WRITE_WITHOUT_RESPONSE | WRITE, descriptors: {Descriptor { uuid: 00002901-0000-1000-8000-00805f9b34fb, service_uuid: f000ffd0-0451-4000-b000-000000000000, characteristic_uuid: f000ffd1-0451-4000-b000-000000000000 }} }}

// service_uuid: 00001800-0000-1000-8000-00805f9b34fb
// service_uuid: 00001800-0000-1000-8000-00805f9b34fb
// service_uuid: 00001800-0000-1000-8000-00805f9b34fb
// service_uuid: 0000180a-0000-1000-8000-00805f9b34fb
// service_uuid: 0000180a-0000-1000-8000-00805f9b34fb
// service_uuid: 0000180a-0000-1000-8000-00805f9b34fb
// service_uuid: 0000180a-0000-1000-8000-00805f9b34fb
// service_uuid: 0000180a-0000-1000-8000-00805f9b34fb
// service_uuid: 0000180a-0000-1000-8000-00805f9b34fb
// service_uuid: 0000180a-0000-1000-8000-00805f9b34fb
// service_uuid: 0000180a-0000-1000-8000-00805f9b34fb
// service_uuid: 0000180a-0000-1000-8000-00805f9b34fb
// service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb
// service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb
// service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb
// service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb
// service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb
// service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb
// service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb
// service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb
// service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb
// service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb
// service_uuid: 0000ffd0-0000-1000-8000-00805f9b34fb
// service_uuid: 0000fff0-0000-1000-8000-00805f9b34fb
// service_uuid: 0000fff0-0000-1000-8000-00805f9b34fb
// service_uuid: 0000fff0-0000-1000-8000-00805f9b34fb
// service_uuid: f000ffd0-0451-4000-b000-000000000000
// service_uuid: f000ffd0-0451-4000-b000-000000000000
