// story: e01s01 e01s02 e01s03
pub mod commands;
pub mod errors;
pub mod events;
pub mod gpu;
pub mod instance;
pub mod metrics;
pub mod output;
pub mod recovery;
pub mod registry;
pub mod runs;
pub mod spawn;
pub mod supervisor;
pub mod watch_exit;

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

            // story: e05s01 — single host: the instance lock is taken before anything
            // reads live-runs.json, so a second instance can never reap the first one's
            // live groups. A host that cannot take the lock exits with one stderr line
            // and no side effects (no window, no file writes, no spawns).
            let instance_lock = match instance::acquire_instance_lock(&data_dir.join("app.lock")) {
                Ok(lock) => lock,
                Err(instance::InstanceError::AlreadyRunning(_)) => {
                    eprintln!("sh-ui: another instance is already running");
                    std::process::exit(0);
                }
                Err(error) => return Err(Box::new(error)),
            };
            // The lock lives as long as this File; managed state keeps it for the
            // whole process. The OS releases flock on exit, including a crash.
            app.manage(instance_lock);

            // story: e02s04 — reap groups a crashed host left before anything can start.
            let reaped = recovery::reap_stale(&data_dir.join("live-runs.json"), &|pgid, _pid| {
                let finished = std::sync::atomic::AtomicBool::new(false);
                spawn::terminate_group(pgid, &finished);
            });
            // pgid == pid for a group leader; the closure only needs the group.
            if !reaped.is_empty() {
                eprintln!("recovery: reaped {} orphaned run group(s)", reaped.len());
            }

            let store = RegistryStore::open(&data_dir.join("registry.json"))?;
            app.manage(Mutex::new(store));

            let supervisor = Supervisor::new(TauriRunSink::new(app.handle().clone()))
                // story: e02s04 — same file the reap above reads.
                .with_state_file(data_dir.join("live-runs.json"));
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
            commands::start_script,
            // story: e02s02
            commands::stop_run
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
