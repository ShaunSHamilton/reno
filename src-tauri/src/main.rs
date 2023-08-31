// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adapter;
mod commands;
mod config;
mod data;
mod peripheral;
mod state;

use commands::{
    connect_to_adapter, connect_to_peripheral, refresh_bluetooth_adapters, request_multiple_events,
    request_single_event, search_for_peripherals,
};
use state::AppState;

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
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
