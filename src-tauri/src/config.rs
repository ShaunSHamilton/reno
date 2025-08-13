//! Handle config such as the file location for saving data, default adapter, peripheral(s), and graph stuff

// Handle app data
// Handle defaults (settings)

use std::{
    error::Error,
    path::{self, PathBuf},
};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::data::Data;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub data_file_path: Option<PathBuf>,
    pub default_adapter: Option<String>,
    pub default_peripheral: Option<String>,
}

#[tauri::command]
pub async fn get_config(app: AppHandle) -> Result<Config, ()> {
    let path_resolver = app.path_resolver();
    if let Some(config_path) = path_resolver.app_config_dir() {
        let config_path = config_path.join("config.json");
        if config_path.exists() {
            let config = tauri::api::file::read_string(config_path).unwrap();
            let config: Config = serde_json::from_str(&config).unwrap();
            return Ok(config);
        }
    }
    return Err(());
}

pub async fn save_data_to_file(data: Data, file_path: PathBuf) -> Result<(), Box<dyn Error>> {
    // Create file if it does not exist, and add `[]` to it if not. Otherwise, just open.
    if !tokio::fs::metadata(&file_path).await.is_ok() {
        tokio::fs::write(&file_path, "[]").await?;
    }

    let mut current_json: Vec<Data> = serde_json::from_str("[]")?;
    if let Ok(file_str) = tokio::fs::read_to_string(&file_path).await {
        current_json = serde_json::from_str(&file_str)?;
    } else {
        tokio::fs::create_dir_all(path::Path::new(&file_path).parent().unwrap()).await?;
    }
    current_json.push(data);
    let new_json = serde_json::to_string(&current_json)?;

    tokio::fs::write(file_path, new_json).await?;
    // println!("Data: {:?}", data);

    Ok(())
}
