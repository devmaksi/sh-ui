// story: e01s03
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::errors::StartError;
use crate::registry::RunConfig;

pub const TERM_GRACE: Duration = Duration::from_secs(2);
const KILL_GRACE: Duration = Duration::from_secs(1);

// ETXTBSY means the file is open for writing somewhere — an editor saving, a backup tool
// copying. It clears in milliseconds, so the host tries a few times before reporting it.
const SPAWN_ATTEMPTS: u8 = 8;
const SPAWN_RETRY_WAIT: Duration = Duration::from_millis(25);

/// story: e02s01 — the stored directory wins; otherwise the directory holding the script
/// (DEC-10), so the same script behaves the same under cargo test and under npm run dev.
pub(crate) fn working_directory(path: &Path, config: &RunConfig) -> Result<std::path::PathBuf, StartError> {
    if let Some(dir) = &config.working_dir {
        let dir = Path::new(dir);
        if !dir.is_dir() {
            return Err(StartError::BadWorkingDir(dir.display().to_string()));
        }
        return Ok(dir.to_path_buf());
    }

    Ok(path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .to_path_buf())
}

pub(crate) fn spawn_capturing(
    path: &Path,
    config: &RunConfig,
    working_dir: &Path,
) -> Result<Child, std::io::Error> {
    let mut attempt = 0;

    loop {
        // argv array only. The path is one argument to exec, never a shell command string.
        // story: e02s01 — stored argv items are appended as separate arguments; stored
        // environment pairs are applied over the inherited host environment.
        let mut command = Command::new(path);
        command
            .args(&config.args)
            .envs(&config.env)
            .current_dir(working_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        unsafe {
            command.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }

        match command.spawn() {
            Ok(child) => return Ok(child),
            Err(error) if error.raw_os_error() == Some(libc::ETXTBSY) && attempt < SPAWN_ATTEMPTS => {
                attempt += 1;
                thread::sleep(SPAWN_RETRY_WAIT);
            }
            Err(error) => return Err(error),
        }
    }
}

/// SIGTERM the whole group, then SIGKILL whatever is still alive.
/// story: e02s04 — public so lib.rs setup can reap a crashed host's groups through
/// the same path a user stop uses.
pub fn terminate_group(pgid: u32, finished: &AtomicBool) {
    let group = pgid as libc::pid_t;

    let term_accepted = unsafe { libc::killpg(group, libc::SIGTERM) } == 0;

    if term_accepted && wait_for_exit(pgid, TERM_GRACE) {
        finished.store(true, Ordering::SeqCst);
        return;
    }

    unsafe { libc::killpg(group, libc::SIGKILL) };
    wait_for_exit(pgid, KILL_GRACE);
    finished.store(true, Ordering::SeqCst);
}

fn wait_for_exit(pid: u32, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let proc = format!("/proc/{pid}");

    while Instant::now() < deadline {
        if !std::path::Path::new(&proc).exists() {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }

    !std::path::Path::new(&proc).exists()
}
