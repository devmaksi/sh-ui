// story: e03s02
// scenario: SC-e03s02-P1-03
use sh_ui_lib::metrics::{tree_sum, ProcReader, ProcessSample};
use std::collections::HashMap;
use std::sync::Mutex;

struct MapReader(Mutex<HashMap<u32, ProcessSample>>);

impl ProcReader for MapReader {
    fn sample(&self, pid: u32) -> Option<ProcessSample> {
        self.0.lock().expect("map reader lock").get(&pid).copied()
    }

    fn children(&self, _pid: u32) -> Vec<u32> {
        Vec::new()
    }
}

fn sample(cpu_ticks: u64, rss_bytes: u64) -> ProcessSample {
    ProcessSample { cpu_ticks, rss_bytes }
}

#[test]
fn tree_sum_handles_exit() {
    let reader = MapReader(Mutex::new(HashMap::from([
        (1, sample(200, 1000)),
        (2, sample(200, 2000)),
        (3, sample(200, 3000)),
    ])));
    let previous = HashMap::from([
        (1, sample(100, 900)),
        (2, sample(100, 1900)),
        (3, sample(100, 2900)),
    ]);

    let (first, baseline) = tree_sum(&reader, &[1, 2, 3], &previous, 1000, 100);
    assert_eq!(first.ram_bytes, 6000);

    // Pid 2 dies after the walk already listed it: the next read must drop it exactly
    // once — no stale contribution, no carry into the next baseline.
    {
        let mut live = reader.0.lock().expect("map reader lock");
        live.remove(&2);
        live.insert(1, sample(300, 1000));
        live.insert(3, sample(300, 3000));
    }

    let pids_after_walk = vec![1, 2, 3];
    let (second, next_baseline) = tree_sum(&reader, &pids_after_walk, &baseline, 1000, 100);

    assert_eq!(
        second.ram_bytes, 4000,
        "the dead child must leave the sum in full, not overshoot"
    );
    assert!(
        (second.cpu_percent - 200.0).abs() < f64::EPSILON,
        "only the two living processes may contribute tick deltas: got {}",
        second.cpu_percent
    );
    assert!(!next_baseline.contains_key(&2), "a dead pid must not survive into the next baseline");
    assert_eq!(next_baseline.len(), 2);
}
