// story: e01s02
// scenario: SC-e01s02-P0-06
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::{Registry, RunConfig, ScriptEntry};

#[test]
fn registry_survives_reload() {
    let temp = TempRegistry::new("reload");
    let file = temp.path();

    let mut registry = Registry::empty();
    registry.insert(ScriptEntry {
        id: "entry-1".to_string(),
        path: "/home/user/build.sh".to_string(),
        config: RunConfig::default(),
    });
    registry.insert(ScriptEntry {
        id: "entry-2".to_string(),
        path: "/home/user/deploy.sh".to_string(),
        config: RunConfig::default(),
    });

    registry.save(&file).expect("save must succeed");
    let reloaded = Registry::load(&file).expect("load must succeed");

    assert_eq!(
        reloaded.list().entries,
        registry.list().entries,
        "entries and ids must survive a save and reload"
    );
}
