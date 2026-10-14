// story: e01s02
// Shared test fixture. Every registry test gets its own throwaway directory so no test
// touches the real application data directory.
// Each integration test crate compiles this module and uses only part of it, so the
// compiler sees the rest as dead. The fixture is shared on purpose.
#![allow(dead_code)]

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TempRegistry {
    dir: PathBuf,
}

impl TempRegistry {
    pub fn new(tag: &str) -> Self {
        let seq = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "sh-ui-test-{}-{}-{}",
            tag,
            std::process::id(),
            seq
        ));
        std::fs::create_dir_all(&dir).expect("temp registry dir must be creatable");
        Self { dir }
    }

    pub fn path(&self) -> PathBuf {
        self.dir.join("registry.json")
    }

    /// The throwaway directory itself, for tests that need a non-file path.
    pub fn dir_path(&self) -> PathBuf {
        self.dir.clone()
    }

    /// Write a file inside the temp directory and return its path.
    pub fn write_file(&self, name: &str, body: &str) -> PathBuf {
        let file = self.dir.join(name);
        std::fs::write(&file, body).expect("fixture file must be writable");
        file
    }

    /// Write an executable script the host is allowed to spawn.
    pub fn write_script(&self, name: &str, body: &str) -> PathBuf {
        let file = self.write_file(name, body);
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755))
            .expect("fixture script must become executable");
        file
    }

    /// Write a non-executable file, for tests that assert the host refuses it.
    pub fn write_inert_file(&self, name: &str, body: &str) -> PathBuf {
        let file = self.write_file(name, body);
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644))
            .expect("fixture file must stay non-executable");
        file
    }
}

impl Drop for TempRegistry {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
