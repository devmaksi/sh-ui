// story: e01s01 e01s02
use std::path::Path;
use std::sync::Mutex;

use serde::Serialize;

use crate::registry::{RegistryError, RegistryList, RegistryStore, ScriptEntry};

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
