// story: e01s03
use std::io::BufRead;
use std::sync::{Arc, Condvar, Mutex};

use crate::events::{EventSink, OutputStream, RunEvent};

/// One event payload cannot exceed this. A script that writes fast must not flood the UI.
pub const OUTPUT_CHUNK_MAX_BYTES: usize = 8 * 1024;

/// When both output readers have reached EOF and the child has been reaped, the run is
/// complete. `Ended` is emitted only after that, so the UI never sees "finished" while
/// output is still in flight.
pub(crate) struct Completion {
    pub(crate) exit_code: Option<Option<i32>>,
    pub(crate) readers_done: u8,
}

pub(crate) fn mark_reader_done(state: &Arc<(Mutex<Completion>, Condvar)>) {
    let (lock, signal) = &**state;
    let mut completion = lock.lock().expect("completion lock");
    completion.readers_done += 1;
    signal.notify_all();
}

pub(crate) fn mark_exit(state: &Arc<(Mutex<Completion>, Condvar)>, code: Option<i32>) {
    let (lock, signal) = &**state;
    let mut completion = lock.lock().expect("completion lock");
    completion.exit_code = Some(code);
    signal.notify_all();
}

pub(crate) fn read_stream(
    stream: impl std::io::Read + Send + 'static,
    which: OutputStream,
    run_id: String,
    sink: Arc<dyn EventSink>,
    completion: Arc<(Mutex<Completion>, Condvar)>,
) {
    std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stream);
        for line in reader.lines() {
            let Ok(line) = line else { break };
            for chunk in split_chunk(&line) {
                sink.emit(RunEvent::Output {
                    run_id: run_id.clone(),
                    stream: which,
                    chunk,
                });
            }
        }
        mark_reader_done(&completion);
    });
}

/// A line longer than the cap becomes several chunks. No single event is bigger than the cap.
fn split_chunk(line: &str) -> Vec<String> {
    if line.len() <= OUTPUT_CHUNK_MAX_BYTES {
        return vec![line.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();

    for character in line.chars() {
        if current.len() + character.len_utf8() > OUTPUT_CHUNK_MAX_BYTES {
            chunks.push(std::mem::take(&mut current));
        }
        current.push(character);
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}
