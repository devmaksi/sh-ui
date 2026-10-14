// story: e01s01
use crate::registry::{Registry, RegistryList};

#[tauri::command]
pub fn list_scripts(registry: tauri::State<'_, Registry>) -> RegistryList {
    registry.list()
}
