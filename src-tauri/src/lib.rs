// story: e01s01 e01s02 e01s03
pub mod commands;
pub mod gpu;
pub mod metrics;
pub mod registry;
pub mod runs;
pub mod supervisor;

use registry::RegistryStore;
use runs::TauriRunSink;
use std::sync::Mutex;
use supervisor::Supervisor;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let store = RegistryStore::open(&data_dir.join("registry.json"))?;
            app.manage(Mutex::new(store));

            let supervisor = Supervisor::new(TauriRunSink::new(app.handle().clone()));
            let mut supervisor = supervisor;
            // story: e04s01 — probe vendor GPU tooling once at startup.
            supervisor.attach_gpu();
            app.manage(Mutex::new(supervisor));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_scripts,
            commands::add_script,
            commands::remove_script,
            // story: e02s01
            commands::update_script_config,
            commands::start_script
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    // A run must not outlive the host. The supervisor also kills its children on drop,
    // but the exit event is the promise we can state out loud.
    app.run(|app_handle, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            let Some(supervisor) = app_handle.try_state::<Mutex<Supervisor>>() else {
                return;
            };

            supervisor.lock().expect("supervisor mutex").shutdown();
        }
    });
}
