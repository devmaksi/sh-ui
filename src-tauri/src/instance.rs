// story: e05s01
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::path::Path;

/// Refusal of the instance lock. The tag is part of the startup contract: a host that
/// cannot take the lock performs no recovery work and no supervision.
#[derive(Debug)]
pub enum InstanceError {
    /// Another live process holds the lock.
    AlreadyRunning(String),
    /// The lock file itself could not be opened or locked.
    LockFailed(String),
}

impl std::fmt::Display for InstanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceError::AlreadyRunning(path) => {
                write!(f, "another instance is already running (lock: {path})")
            }
            InstanceError::LockFailed(detail) => write!(f, "instance lock failed: {detail}"),
        }
    }
}

impl std::error::Error for InstanceError {}

/// Take the exclusive instance lock on the given path (DEC: one host owns live-runs.json).
/// flock is advisory and released by the OS when the holding process dies — a crashed
/// host never leaves a stale lock. The lock lives as long as the returned File is open;
/// the caller must keep it alive for the whole process.
pub fn acquire_instance_lock(path: &Path) -> Result<File, InstanceError> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|error| InstanceError::LockFailed(format!("{}: {error}", path.display())))?;

    // Non-blocking exclusive attempt: EWOULDBLOCK means a live sibling holds it.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
        return Ok(file);
    }

    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::EWOULDBLOCK) {
        return Err(InstanceError::AlreadyRunning(path.display().to_string()));
    }

    Err(InstanceError::LockFailed(format!(
        "{}: {error}",
        path.display()
    )))
}
