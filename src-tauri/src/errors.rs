// story: e01s03
#[derive(Debug)]
pub enum StartError {
    EntryNotFound(String),
    NotExecutable(String),
    SpawnFailed(String),
    /// story: e02s01 — a stored working directory vanished after the save.
    BadWorkingDir(String),
}

impl StartError {
    /// Stable tag for the UI error union. Changing a tag here changes the TypeScript union.
    pub fn kind(&self) -> &'static str {
        match self {
            StartError::EntryNotFound(_) => "entryNotFound",
            StartError::NotExecutable(_) => "notExecutable",
            StartError::SpawnFailed(_) => "spawnFailed",
            StartError::BadWorkingDir(_) => "badWorkingDir",
        }
    }
}

impl std::fmt::Display for StartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartError::EntryNotFound(detail) => write!(f, "no registry entry: {detail}"),
            StartError::NotExecutable(path) => {
                write!(f, "{path} is not executable; add the execute bit or remove the entry")
            }
            StartError::SpawnFailed(detail) => write!(f, "the process could not be started: {detail}"),
            StartError::BadWorkingDir(dir) => {
                write!(f, "working directory {dir} no longer exists; edit the entry or remove it")
            }
        }
    }
}

/// Refusal of a stop request. The tag is part of the UI error union.
#[derive(Debug)]
pub enum StopError {
    RunNotFound(String),
}

impl StopError {
    pub fn kind(&self) -> &'static str {
        match self {
            StopError::RunNotFound(_) => "runNotFound",
        }
    }
}

impl std::fmt::Display for StopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StopError::RunNotFound(run_id) => write!(f, "no live run with id {run_id}"),
        }
    }
}
