// story: e03s01
// scenario: SC-e03s01-P2-04
use sh_ui_lib::metrics::cpu_percent;

#[test]
fn cpu_percent_math_one_core_busy() {
    // 100 consumed ticks over 1 wall second on a 100 Hz clock = one core fully busy.
    let percent = cpu_percent(100, 1000, 100);
    assert!((percent - 100.0).abs() < f64::EPSILON, "got {percent}");
}

#[test]
fn cpu_percent_math_multithread_exceeds_100() {
    // DEC-12: percent of one core. 250 ticks in one second on a 100 Hz clock = 250%.
    let percent = cpu_percent(250, 1000, 100);
    assert!((percent - 250.0).abs() < f64::EPSILON, "got {percent}");
}

#[test]
fn cpu_percent_math_zero_wall_never_divides() {
    assert_eq!(cpu_percent(100, 0, 100), 0.0);
}

#[test]
fn cpu_percent_math_half_core() {
    let percent = cpu_percent(50, 1000, 100);
    assert!((percent - 50.0).abs() < f64::EPSILON, "got {percent}");
}
