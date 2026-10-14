// story: e01s01
pub mod commands;
pub mod registry;

use registry::Registry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Registry::empty())
        .invoke_handler(tauri::generate_handler![commands::list_scripts])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
