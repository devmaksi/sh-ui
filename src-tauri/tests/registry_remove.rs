// story: e01s02
// scenario: SC-e01s02-P1-08
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::{Registry, RegistryError};

#[test]
fn registry_remove_deletes_only_the_target_entry() {
    let temp = TempRegistry::new("remove-one");
    let first = temp.write_file("one.sh", "#!/bin/sh\n");
    let second = temp.write_file("two.sh", "#!/bin/sh\n");

    let mut registry = Registry::empty();
    let removed = registry.add_path(&first).expect("first accepted");
    let kept = registry.add_path(&second).expect("second accepted");
    registry.save(&temp.path()).expect("save works");

    registry.remove(&removed.id).expect("remove must succeed");
    registry.save(&temp.path()).expect("save after remove works");

    let reloaded = Registry::load(&temp.path()).expect("load works");
    let remaining: Vec<String> = reloaded
        .list()
        .entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect();

    assert_eq!(remaining, vec![kept.path], "only the target entry disappears");
}

#[test]
fn registry_remove_reports_an_unknown_id() {
    let mut registry = Registry::empty();
    let error = registry
        .remove("entry-that-does-not-exist")
        .expect_err("an unknown id must be reported");

    assert!(
        matches!(error, RegistryError::NotFound(_)),
        "an unknown id must be reported as NotFound, got {error:?}"
    );
}
