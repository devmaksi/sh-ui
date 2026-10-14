// story: e01s01
// scenario: SC-e01s01-P0-01
use sh_ui_lib::registry::{Registry, RegistryList};

#[test]
fn list_scripts_returns_empty_registry() {
    let list: RegistryList = Registry::empty().list();
    assert!(
        list.entries.is_empty(),
        "a fresh registry must list zero entries, not fail"
    );
}
