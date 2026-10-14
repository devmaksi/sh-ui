// story: e01s02
// scenarios: SC-e01s02-P0-02, SC-e01s02-P0-03, SC-e01s02-P0-04, SC-e01s02-P0-05
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::{Registry, RegistryError};
use std::os::unix::fs::PermissionsExt;

#[test]
fn registry_rejects_a_directory() {
    let temp = TempRegistry::new("reject-dir");

    let mut registry = Registry::empty();
    let error = registry
        .add_path(&temp.dir_path())
        .expect_err("a directory is not a script");

    assert!(
        matches!(error, RegistryError::NotAFile(_)),
        "a directory must be reported as NotAFile, got {error:?}"
    );
    assert!(registry.list().entries.is_empty());
}

#[test]
fn registry_rejects_a_missing_path() {
    let temp = TempRegistry::new("reject-missing");
    let missing = temp.dir_path().join("absent.sh");

    let mut registry = Registry::empty();
    let error = registry
        .add_path(&missing)
        .expect_err("a missing path is not a script");

    assert!(
        matches!(error, RegistryError::NotFound(_)),
        "a missing path must be reported as NotFound, got {error:?}"
    );
}

#[test]
fn registry_rejects_a_file_that_is_not_a_shell_script() {
    let temp = TempRegistry::new("reject-extension");
    let notes = temp.write_file("notes.txt", "not a script");

    let mut registry = Registry::empty();
    let error = registry
        .add_path(&notes)
        .expect_err("a .txt file is not a script");

    assert!(
        matches!(error, RegistryError::NotShellScript(_)),
        "a non-.sh file must be reported as NotShellScript, got {error:?}"
    );
}

#[test]
fn registry_rejects_a_file_it_cannot_read() {
    let temp = TempRegistry::new("reject-unreadable");
    let locked = temp.write_file("locked.sh", "#!/bin/sh\n");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000))
        .expect("permissions must be changeable in the temp dir");

    let mut registry = Registry::empty();
    let error = registry
        .add_path(&locked)
        .expect_err("an unreadable file cannot be registered");

    assert!(
        matches!(error, RegistryError::NotReadable(_)),
        "an unreadable file must be reported as NotReadable, got {error:?}"
    );
}
