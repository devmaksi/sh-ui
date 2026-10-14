// story: e01s01
use serde::{Deserialize, Serialize};

/// One registered script. `id` is stable across reloads so later stories can
/// attribute runs and metrics to the entry that started them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptEntry {
    pub id: String,
    pub path: String,
}

/// The payload the UI receives when it asks for the registry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RegistryList {
    pub entries: Vec<ScriptEntry>,
}

#[derive(Debug, Default)]
pub struct Registry {
    entries: Vec<ScriptEntry>,
}

impl Registry {
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn list(&self) -> RegistryList {
        RegistryList {
            entries: self.entries.clone(),
        }
    }
}
