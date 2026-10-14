// story: e01s01 e01s02
pub mod commands;
pub mod registry;

use registry::RegistryStore;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let store = RegistryStore::open(&data_dir.join("registry.json"))?;
            app.manage(Mutex::new(store));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_scripts,
            commands::add_script,
            commands::remove_script
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
