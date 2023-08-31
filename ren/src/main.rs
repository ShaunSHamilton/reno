use ble::{get_bt_adapter, handle_device_events, scan_for_devices};
use clap::Parser;
use std::error::Error;

mod ble;
mod clapper;
mod data;

use clapper::Args;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let central = get_bt_adapter().await;
    scan_for_devices(&central).await;
    handle_device_events(&central, &args).await?;

    Ok(())
}
