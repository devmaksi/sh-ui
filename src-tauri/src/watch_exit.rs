// story: e01s03
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use crate::events::{EventSink, RunEvent, RunStatus};
use crate::output::{Completion, mark_exit};
use crate::recovery::LiveRunFile;

/// Everything one exit-watch thread needs for one run. Bundled because the thread
/// owns the whole end-of-run sequence: reap, drain, stop the poller, record removal,
/// verdict.
pub(crate) struct ExitWatch {
    pub(crate) child: std::process::Child,
    pub(crate) run_id: String,
    pub(crate) sink: Arc<dyn EventSink>,
    pub(crate) finished: Arc<AtomicBool>,
    pub(crate) stopped: Arc<AtomicBool>,
    pub(crate) completion: Arc<(Mutex<Completion>, Condvar)>,
    pub(crate) state_file: Option<LiveRunFile>,
    pub(crate) poller: thread::JoinHandle<()>,
}

pub(crate) fn watch_exit(watch: ExitWatch) {
    thread::spawn(move || {
        let ExitWatch {
            mut child,
            run_id,
            sink,
            finished,
            stopped,
            completion,
            state_file,
            poller,
        } = watch;

        let pid = child.id();
        let status = child.wait();
        mark_exit(&completion, status.ok().and_then(|status| status.code()));

        let (lock, signal) = &*completion;
        let mut guard = lock.lock().expect("completion lock");
        while guard.readers_done < 2 {
            guard = signal.wait(guard).expect("completion lock not poisoned");
        }

        let code = guard.exit_code.flatten();
        let stopped = stopped.load(Ordering::SeqCst);
        drop(guard);

        let status = if stopped {
            RunStatus::Stopped
        } else if code == Some(0) {
            RunStatus::Exited
        } else {
            RunStatus::Failed
        };

        // story: e02s04 — stop the poller first and wait for it to leave, so Ended is
        // structurally the last event of the run.
        finished.store(true, Ordering::SeqCst);
        let _ = poller.join();

        // story: e02s04 — the record goes away with the live process, before the
        // verdict is announced, so no observer can see Ended and a stale record together.
        if let Some(file) = &state_file {
            file.remove(pid);
        }

        sink.emit(RunEvent::Ended { run_id, code, status });
    });
}
