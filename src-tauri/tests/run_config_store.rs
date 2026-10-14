// story: e02s01
// scenario: SC-e02s01-P1-01
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::{Registry, RunConfig, ScriptEntry};
use std::collections::BTreeMap;

#[test]
fn run_config_round_trips() {
    let temp = TempRegistry::new("run_config_round_trip");
    let file = temp.path();

    let mut registry = Registry::empty();
    registry.insert(ScriptEntry {
        id: "entry-1".to_string(),
        path: "/home/user/build.sh".to_string(),
        config: RunConfig {
            args: vec!["a b".to_string(), "--flag".to_string()],
            env: BTreeMap::from([("DEMO_VAR".to_string(), "hello".to_string())]),
            working_dir: Some("/tmp".to_string()),
        },
    });

    registry.save(&file).expect("save must succeed");
    let reloaded = Registry::load(&file).expect("load must succeed");

    assert_eq!(
        reloaded.list().entries,
        registry.list().entries,
        "the run configuration must survive a save and reload"
    );
}

#[test]
fn version_one_document_loads_with_empty_config() {
    let temp = TempRegistry::new("run_config_v1");
    let file = temp.path();

    std::fs::write(
        &file,
        r#"{ "version": 1, "entries": [ { "id": "entry-1", "path": "/home/user/build.sh" } ] }"#,
    )
    .expect("fixture document must be writable");

    let registry = Registry::load(&file).expect("a version 1 document must still load");
    let entries = registry.list().entries;

    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].config,
        RunConfig::default(),
        "a stored entry without a config gets the empty configuration"
    );
}

#[test]
fn save_writes_version_two() {
    let temp = TempRegistry::new("run_config_version");
    let file = temp.path();

    Registry::empty().save(&file).expect("save must succeed");
    let text = std::fs::read_to_string(&file).expect("document must be readable");

    assert!(
        text.contains("\"version\": 2"),
        "the written document must declare version 2, got: {text}"
    );
}
