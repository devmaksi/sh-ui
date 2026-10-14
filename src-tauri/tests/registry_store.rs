// story: e01s02
// scenario: SC-e01s02-P0-06
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::RegistryStore;

#[test]
fn store_persists_every_mutation_without_an_explicit_save() {
    let temp = TempRegistry::new("store-mutation");
    let script = temp.write_file("a.sh", "#!/bin/sh\n");

    let mut store = RegistryStore::open(&temp.path()).expect("open works before the file exists");
    let added = store.add(&script).expect("add must succeed");

    let mut reopened = RegistryStore::open(&temp.path()).expect("open works after the file exists");
    let entries = reopened.list().entries;

    assert_eq!(entries.len(), 1, "the add must already be on disk");
    assert_eq!(entries[0].id, added.id, "the id must survive the round trip");

    reopened.remove(&added.id).expect("remove must succeed");

    let after_remove = RegistryStore::open(&temp.path()).expect("open works after remove");
    assert!(
        after_remove.list().entries.is_empty(),
        "the remove must already be on disk"
    );
}
