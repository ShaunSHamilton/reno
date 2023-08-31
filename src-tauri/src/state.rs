use std::sync::Mutex;

use btleplug::platform::{Adapter, Peripheral};

pub struct AppState {
    pub adapters: Mutex<Vec<Adapter>>,
    pub chosen_adapter: Mutex<Option<Adapter>>,
    pub peripherals: Mutex<Vec<Peripheral>>,
    pub chosen_peripheral: Mutex<Option<Peripheral>>,
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
