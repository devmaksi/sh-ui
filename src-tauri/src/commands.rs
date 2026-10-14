// story: e01s01 e01s02
use std::path::Path;
use std::sync::Mutex;

use serde::Serialize;

use crate::registry::{RegistryError, RegistryList, RegistryStore, ScriptEntry};
use crate::supervisor::{StartError, Supervisor};

/// The error the UI receives. `kind` is the stable tag; `detail` names the path involved.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryErrorPayload {
    pub kind: &'static str,
    pub detail: String,
}

fn to_payload(error: RegistryError) -> RegistryErrorPayload {
    RegistryErrorPayload {
        kind: error.kind(),
        detail: error.to_string(),
    }
}

fn lock(registry: &Mutex<RegistryStore>) -> std::sync::MutexGuard<'_, RegistryStore> {
    registry.lock().expect("registry mutex must not be poisoned")
}

#[tauri::command]
pub fn list_scripts(registry: tauri::State<'_, Mutex<RegistryStore>>) -> RegistryList {
    lock(&registry).list()
}

#[tauri::command]
pub fn add_script(
    registry: tauri::State<'_, Mutex<RegistryStore>>,
    path: String,
) -> Result<ScriptEntry, RegistryErrorPayload> {
    lock(&registry).add(Path::new(&path)).map_err(to_payload)
}

#[tauri::command]
pub fn remove_script(
    registry: tauri::State<'_, Mutex<RegistryStore>>,
    id: String,
) -> Result<(), RegistryErrorPayload> {
    lock(&registry).remove(&id).map_err(to_payload)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartErrorPayload {
    pub kind: &'static str,
    pub detail: String,
}

fn to_start_payload(error: StartError) -> StartErrorPayload {
    StartErrorPayload {
        kind: error.kind(),
        detail: error.to_string(),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunHandlePayload {
    pub run_id: String,
    pub entry_id: String,
    pub pid: u32,
}

#[tauri::command]
pub fn start_script(
    registry: tauri::State<'_, Mutex<RegistryStore>>,
    supervisor: tauri::State<'_, Mutex<Supervisor>>,
    entry_id: String,
) -> Result<RunHandlePayload, StartErrorPayload> {
    let path = {
        let entries = lock(&registry).list().entries;
        entries
            .into_iter()
            .find(|entry| entry.id == entry_id)
            .map(|entry| entry.path)
    };

    let Some(path) = path else {
        return Err(to_start_payload(StartError::EntryNotFound(entry_id)));
    };

    let handle = supervisor
        .lock()
        .expect("supervisor mutex must not be poisoned")
        .start(&entry_id, std::path::Path::new(&path))
        .map_err(to_start_payload)?;

    Ok(RunHandlePayload {
        run_id: handle.run_id,
        entry_id: handle.entry_id,
        pid: handle.pid,
    })
}
