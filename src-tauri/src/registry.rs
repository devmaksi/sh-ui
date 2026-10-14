// story: e01s01 e01s02
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Loads once, saves on every mutation. Reason for depth: an unpersisted mutation is the
/// failure mode this product cannot tolerate, so the type makes it impossible to forget.
pub struct RegistryStore {
    path: PathBuf,
    registry: Registry,
}

impl RegistryStore {
    pub fn open(path: &Path) -> Result<Self, RegistryError> {
        let registry = if path.exists() {
            Registry::load(path)?
        } else {
            Registry::empty()
        };

        Ok(Self {
            path: path.to_path_buf(),
            registry,
        })
    }

    pub fn list(&self) -> RegistryList {
        self.registry.list()
    }

    pub fn add(&mut self, script: &Path) -> Result<ScriptEntry, RegistryError> {
        let entry = self.registry.add_path(script)?;
        self.registry.save(&self.path)?;
        Ok(entry)
    }

    pub fn remove(&mut self, id: &str) -> Result<(), RegistryError> {
        self.registry.remove(id)?;
        self.registry.save(&self.path)?;
        Ok(())
    }

    /// story: e02s01 — validate, store, and persist one entry's run configuration.
    pub fn update_config(
        &mut self,
        id: &str,
        args_lines: Vec<String>,
        env_lines: Vec<String>,
        working_dir: String,
    ) -> Result<ScriptEntry, RegistryError> {
        let entry = self.registry.update_config(id, args_lines, env_lines, working_dir)?;
        self.registry.save(&self.path)?;
        Ok(entry)
    }
}

/// Version of the on-disk registry document. Bump when the shape changes.
/// Version 1 documents (no run config) still load; entries get the empty config.
pub const REGISTRY_FILE_VERSION: u32 = 2;
const LOADED_VERSIONS: [u32; 2] = [1, 2];

/// Per-entry run configuration (e02s01). Arguments are exec argv items in order — never
/// shell text. Environment entries override the inherited host environment. A missing
/// working directory means "the directory holding the script" (DEC-10).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RunConfig {
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub working_dir: Option<String>,
}

/// One registered script. `id` is stable across reloads so later stories can
/// attribute runs and metrics to the entry that started them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptEntry {
    pub id: String,
    pub path: String,
    #[serde(default)]
    pub config: RunConfig,
}

/// The payload the UI receives when it asks for the registry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RegistryList {
    pub entries: Vec<ScriptEntry>,
}

/// The on-disk document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct RegistryFile {
    version: u32,
    entries: Vec<ScriptEntry>,
}

#[derive(Debug)]
pub enum RegistryError {
    /// The registry file could not be read or written.
    Io(String),
    /// The registry file exists but is not a readable registry document.
    Corrupt(String),
    /// The path does not exist.
    NotFound(String),
    /// The path exists but is not a regular file.
    NotAFile(String),
    /// The path is a file without a `.sh` suffix.
    NotShellScript(String),
    /// The file exists but cannot be read.
    NotReadable(String),
    /// story: e02s01 — a configured working directory does not exist or is not a directory.
    BadWorkingDir(String),
    /// story: e02s01 — an environment line is not a valid KEY=VALUE pair.
    BadEnvVar(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::Io(detail) => write!(f, "registry file error: {detail}"),
            RegistryError::Corrupt(detail) => {
                write!(f, "registry file is not readable as JSON: {detail}")
            }
            RegistryError::NotFound(path) => write!(f, "no file at {path}"),
            RegistryError::NotAFile(path) => write!(f, "{path} is not a regular file"),
            RegistryError::NotShellScript(path) => {
                write!(f, "{path} is not a .sh script")
            }
            RegistryError::NotReadable(path) => write!(f, "{path} cannot be read"),
            RegistryError::BadWorkingDir(path) => {
                write!(f, "working directory {path} does not exist or is not a directory")
            }
            RegistryError::BadEnvVar(line) => {
                write!(f, "{line} is not a valid KEY=VALUE environment line")
            }
        }
    }
}

impl RegistryError {
    /// The stable tag the UI maps to its own error union. Changing a tag here means the
    /// TypeScript union changes in the same commit.
    pub fn kind(&self) -> &'static str {
        match self {
            RegistryError::Io(_) => "io",
            RegistryError::Corrupt(_) => "corrupt",
            RegistryError::NotFound(_) => "notFound",
            RegistryError::NotAFile(_) => "notAFile",
            RegistryError::NotShellScript(_) => "notShellScript",
            RegistryError::NotReadable(_) => "notReadable",
            RegistryError::BadWorkingDir(_) => "badWorkingDir",
            RegistryError::BadEnvVar(_) => "badEnvVar",
        }
    }
}

impl std::error::Error for RegistryError {}

fn parse_run_config(
    args_lines: Vec<String>,
    env_lines: Vec<String>,
    working_dir: String,
) -> Result<RunConfig, RegistryError> {
    let args: Vec<String> = args_lines
        .into_iter()
        .filter(|line| !line.is_empty())
        .collect();

    let mut env = BTreeMap::new();
    for line in env_lines {
        if line.is_empty() {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| RegistryError::BadEnvVar(line.clone()))?;
        if !is_env_key(key) {
            return Err(RegistryError::BadEnvVar(line));
        }
        env.insert(key.to_string(), value.to_string());
    }

    let working_dir = {
        let trimmed = working_dir.trim();
        if trimmed.is_empty() {
            None
        } else {
            if !std::path::Path::new(trimmed).is_dir() {
                return Err(RegistryError::BadWorkingDir(trimmed.to_string()));
            }
            Some(trimmed.to_string())
        }
    };

    Ok(RunConfig {
        args,
        env,
        working_dir,
    })
}

/// POSIX-ish environment name: ASCII letter or underscore first, then alphanumerics.
fn is_env_key(key: &str) -> bool {
    let mut characters = key.chars();
    match characters.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    characters.all(|c| c.is_ascii_alphanumeric() || c == '_')
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

    pub fn insert(&mut self, entry: ScriptEntry) {
        self.entries.push(entry);
    }

    /// Register a script file. The id is generated once here and then persists with the
    /// document, so later stories can attribute runs to the entry that started them.
    pub fn add_path(&mut self, path: &Path) -> Result<ScriptEntry, RegistryError> {
        let shown = path.display().to_string();

        let metadata = match std::fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(RegistryError::NotFound(shown));
            }
            Err(error) => return Err(RegistryError::Io(error.to_string())),
        };

        if !metadata.is_file() {
            return Err(RegistryError::NotAFile(shown));
        }

        if path.extension().and_then(|e| e.to_str()) != Some("sh") {
            return Err(RegistryError::NotShellScript(shown));
        }

        std::fs::File::open(path).map_err(|_| RegistryError::NotReadable(shown.clone()))?;

        let entry = ScriptEntry {
            id: self.next_id(),
            path: shown,
            config: RunConfig::default(),
        };
        self.insert(entry.clone());
        Ok(entry)
    }

    pub fn remove(&mut self, id: &str) -> Result<(), RegistryError> {
        let before = self.entries.len();
        self.entries.retain(|entry| entry.id != id);

        if self.entries.len() == before {
            return Err(RegistryError::NotFound(format!(
                "no registry entry with id {id}"
            )));
        }

        Ok(())
    }

    /// story: e02s01 — replace one entry's run configuration. The draft arrives as raw
    /// lines: one argv item per line, one KEY=VALUE per line, one directory path. Empty
    /// lines are skipped; argument lines stay verbatim because there is no shell.
    pub fn update_config(
        &mut self,
        id: &str,
        args_lines: Vec<String>,
        env_lines: Vec<String>,
        working_dir: String,
    ) -> Result<ScriptEntry, RegistryError> {
        let config = parse_run_config(args_lines, env_lines, working_dir)?;
        let entry = self
            .entries
            .iter_mut()
            .find(|entry| entry.id == id)
            .ok_or_else(|| RegistryError::NotFound(format!("no registry entry with id {id}")))?;
        entry.config = config;
        Ok(entry.clone())
    }

    fn next_id(&self) -> String {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("e{}-{:x}", self.entries.len() + 1, stamp)
    }

    pub fn list(&self) -> RegistryList {
        RegistryList {
            entries: self.entries.clone(),
        }
    }

    pub fn load(path: &Path) -> Result<Registry, RegistryError> {
        let text = std::fs::read_to_string(path).map_err(|e| RegistryError::Io(e.to_string()))?;
        let file: RegistryFile =
            serde_json::from_str(&text).map_err(|e| RegistryError::Corrupt(e.to_string()))?;

        if !LOADED_VERSIONS.contains(&file.version) {
            return Err(RegistryError::Corrupt(format!(
                "unsupported registry version {} — this build writes version {}",
                file.version, REGISTRY_FILE_VERSION
            )));
        }

        Ok(Registry {
            entries: file.entries,
        })
    }

    /// Write to a sibling temporary file, then rename. A crash mid-write leaves the
    /// previous document intact instead of a half-written one.
    pub fn save(&self, path: &Path) -> Result<(), RegistryError> {
        let file = RegistryFile {
            version: REGISTRY_FILE_VERSION,
            entries: self.entries.clone(),
        };
        let text = serde_json::to_string_pretty(&file).map_err(|e| RegistryError::Io(e.to_string()))?;

        let temp = path.with_extension("json.tmp");
        std::fs::write(&temp, text).map_err(|e| RegistryError::Io(e.to_string()))?;
        std::fs::rename(&temp, path).map_err(|e| RegistryError::Io(e.to_string()))?;
        Ok(())
    }
}
