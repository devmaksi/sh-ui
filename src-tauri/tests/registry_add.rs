// story: e01s02
// scenario: SC-e01s02-P0-01
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::Registry;

#[test]
fn registry_add_accepts_a_valid_shell_script() {
    let temp = TempRegistry::new("add-valid");
    let script = temp.write_file("build.sh", "#!/bin/sh\necho building\n");

    let mut registry = Registry::empty();
    let entry = registry
        .add_path(&script)
        .expect("a readable .sh file must be accepted");

    assert_eq!(entry.path, script.to_string_lossy());
    assert!(!entry.id.is_empty(), "the entry must carry a usable id");
    assert_eq!(registry.list().entries.len(), 1);
}

#[test]
fn registry_add_gives_each_entry_a_distinct_id() {
    let temp = TempRegistry::new("add-ids");
    let first = temp.write_file("one.sh", "#!/bin/sh\n");
    let second = temp.write_file("two.sh", "#!/bin/sh\n");

    let mut registry = Registry::empty();
    let a = registry.add_path(&first).expect("first accepted");
    let b = registry.add_path(&second).expect("second accepted");

    assert_ne!(a.id, b.id, "ids must not collide");
}
