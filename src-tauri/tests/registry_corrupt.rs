// story: e01s02
// scenario: SC-e01s02-P1-07
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::{Registry, RegistryError};

#[test]
fn registry_reports_a_corrupt_file_instead_of_an_empty_registry() {
    let temp = TempRegistry::new("corrupt");
    std::fs::write(temp.path(), "{ not json at all").expect("fixture write must succeed");

    let error = Registry::load(&temp.path()).expect_err("a corrupt document must not load");

    assert!(
        matches!(error, RegistryError::Corrupt(_)),
        "a corrupt registry must be reported as Corrupt, got {error:?}"
    );
}

#[test]
fn registry_reports_a_document_it_cannot_understand() {
    let temp = TempRegistry::new("wrong-version");
    std::fs::write(temp.path(), r#"{ "version": 99, "entries": [] }"#)
        .expect("fixture write must succeed");

    let error = Registry::load(&temp.path()).expect_err("an unknown version must not load");

    assert!(
        matches!(error, RegistryError::Corrupt(_)),
        "an unsupported version must be reported as Corrupt, got {error:?}"
    );
}
