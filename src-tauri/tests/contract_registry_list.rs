// story: e01s01
// scenario: SC-e01s01-P1-03
use sh_ui_lib::registry::{RegistryList, RunConfig, ScriptEntry};

fn fixture_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("registry-list.json")
}

#[test]
fn contract_registry_list_matches_the_shared_fixture() {
    let fixture = std::fs::read_to_string(fixture_path())
        .expect("tests/fixtures/registry-list.json must exist");

    let list = RegistryList {
        entries: vec![
            ScriptEntry {
                id: "a1".to_string(),
                path: "/home/user/build.sh".to_string(),
                config: RunConfig::default(),
            },
            ScriptEntry {
                id: "b2".to_string(),
                path: "/home/user/deploy.sh".to_string(),
                config: RunConfig::default(),
            },
        ],
    };

    let produced: serde_json::Value =
        serde_json::to_value(&list).expect("registry list must serialise");
    let expected: serde_json::Value =
        serde_json::from_str(&fixture).expect("fixture must be valid JSON");

    assert_eq!(
        produced, expected,
        "the Rust payload no longer matches the fixture the UI is tested against"
    );
}
