// story: e03s02
// scenario: SC-e03s02-P1-02
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
fn tree_sum_adds_every_process_in_the_tree() {
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

    let (snapshot, baseline) = tree_sum(&reader, &[1, 2, 3], &previous, 1000, 100);

    // Three processes each consumed 100 ticks in one wall second on a 100 Hz clock:
    // the tree used three full cores.
    assert!(
        (snapshot.cpu_percent - 300.0).abs() < f64::EPSILON,
        "got {}",
        snapshot.cpu_percent
    );
    assert_eq!(snapshot.ram_bytes, 6000, "rss must be summed over the tree");
    assert_eq!(baseline.len(), 3, "the new baseline must cover the whole tree");
}

#[test]
fn tree_sum_counts_a_new_child_from_zero_ticks() {
    let reader = MapReader(Mutex::new(HashMap::from([
        (1, sample(200, 1000)),
        (2, sample(500, 2000)),
    ])));
    let previous = HashMap::from([(1, sample(100, 900))]);

    let (snapshot, baseline) = tree_sum(&reader, &[1, 2], &previous, 1000, 100);

    // Only pid 1 had a baseline: 100 ticks. The child born between intervals
    // contributes its memory now and no cpu history yet.
    assert!((snapshot.cpu_percent - 100.0).abs() < f64::EPSILON);
    assert_eq!(snapshot.ram_bytes, 3000);
    assert_eq!(baseline.len(), 2);
}
