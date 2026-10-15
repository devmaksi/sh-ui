// story: e02s04
//
// The host records every live run's process group while the run is live. A clean exit
// clears the record; a crashed host leaves it behind, and the next start reaps the
// recorded groups that are still alive. The start-time match is the guard against pid
// reuse: a record is only acted on when the live process started at the recorded time.
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// One live run's group identity. `pgid` equals the leader pid because every run is
/// spawned as a group leader (DEC-07). `start_time` is milliseconds since boot from
/// procfs (DEC-11), never a hand-parsed /proc field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveGroupRecord {
    pub pgid: u32,
    pub pid: u32,
    pub start_time: u64,
}

/// Process start time in clock ticks since boot (procfs `Stat::starttime`), or None
/// when the pid is gone or unreadable. Used as the identity check against pid reuse.
pub fn start_time_ticks(pid: u32) -> Option<u64> {
    let stat = procfs::process::Process::new(pid as i32).ok()?.stat().ok()?;
    Some(stat.starttime)
}

/// A missing or corrupt file loads as empty: a broken record file must never block
/// app start.
pub fn load_records(path: &Path) -> Vec<LiveGroupRecord> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Reason for depth: after a crash the new host process has no in-memory way to find
/// the groups the old one left; one small file is the minimum record that makes
/// reaping possible. Callers = Supervisor (insert/remove/clear), lib.rs setup (reap).
#[derive(Debug, Clone)]
pub struct LiveRunFile {
    path: PathBuf,
}

impl LiveRunFile {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn records(&self) -> Vec<LiveGroupRecord> {
        load_records(&self.path)
    }

    pub fn insert(&self, record: &LiveGroupRecord) {
        let mut records = self.records();
        records.retain(|existing| existing.pid != record.pid);
        records.push(record.clone());
        self.save(&records);
    }

    pub fn remove(&self, pid: u32) {
        let mut records = self.records();
        records.retain(|existing| existing.pid != pid);
        self.save(&records);
    }

    pub fn clear(&self) {
        self.save(&[]);
    }

    /// Write to a sibling temporary file, then rename — the same discipline as
    /// Registry::save, so a crash mid-write cannot corrupt the record.
    fn save(&self, records: &[LiveGroupRecord]) {
        let body = match serde_json::to_string(records) {
            Ok(body) => body,
            Err(error) => {
                eprintln!("recovery: cannot serialize live runs: {error}");
                return;
            }
        };

        let temp = self.path.with_extension("json.tmp");
        if let Err(error) = std::fs::write(&temp, body) {
            eprintln!("recovery: cannot write {}: {error}", temp.display());
            return;
        }

        if let Err(error) = std::fs::rename(&temp, &self.path) {
            eprintln!("recovery: cannot rename {} to {}: {error}", temp.display(), self.path.display());
        }
    }
}

/// Reap every recorded group that is still alive and start-time-matched; drop every
/// other record without killing. `terminate(pgid, pid)` is the supervisor's group
/// termination path. Returns the records the sweep acted on and clears the file.
pub fn reap_stale(path: &Path, terminate: &dyn Fn(u32, u32)) -> Vec<LiveGroupRecord> {
    let records = load_records(path);
    let mut acted = Vec::new();

    for record in &records {
        match start_time_ticks(record.pid) {
            Some(current_start) if current_start == record.start_time => {
                terminate(record.pgid, record.pid);
                acted.push(record.clone());
            }
            Some(_) => eprintln!(
                "recovery: pid {} was reused by another process; record dropped without killing",
                record.pid
            ),
            None => eprintln!("recovery: pid {} is gone; record dropped", record.pid),
        }
    }

    LiveRunFile::new(path).clear();
    acted
}
