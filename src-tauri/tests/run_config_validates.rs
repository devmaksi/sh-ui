// story: e02s01
// scenarios: SC-e02s01-P1-02, SC-e02s01-P1-03
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::{Registry, RegistryError, ScriptEntry};

fn registry_with_entry(temp: &TempRegistry) -> Registry {
    let mut registry = Registry::empty();
    registry
        .add_path(&temp.write_script("build.sh", "#!/bin/sh\nexit 0\n"))
        .expect("fixture entry must register");
    registry
}

fn kind_of(result: Result<ScriptEntry, RegistryError>) -> &'static str {
    result.expect_err("update must be refused").kind()
}

#[test]
fn missing_working_dir_is_refused() {
    let temp = TempRegistry::new("cfg_missing_dir");
    let mut registry = registry_with_entry(&temp);
    let id = registry.list().entries[0].id.clone();

    let result = registry.update_config(&id, vec![], vec![], "/definitely/not/here".to_string());
    assert_eq!(
        result.expect_err("refusal expected").kind(),
        "badWorkingDir",
        "a working directory that does not exist must be refused"
    );
}

#[test]
fn directory_must_be_a_directory() {
    let temp = TempRegistry::new("cfg_dir_is_file");
    let mut registry = registry_with_entry(&temp);
    let id = registry.list().entries[0].id.clone();
    let file = temp.write_file("not-a-dir", "x");

    let result = registry.update_config(&id, vec![], vec![], file.display().to_string());
    assert_eq!(
        result.expect_err("refusal expected").kind(),
        "badWorkingDir",
        "a file cannot be a working directory"
    );
}

#[test]
fn valid_working_dir_is_stored_and_empty_means_default() {
    let temp = TempRegistry::new("cfg_dir_ok");
    let mut registry = registry_with_entry(&temp);
    let id = registry.list().entries[0].id.clone();
    let dir = temp.dir_path();

    let entry = registry
        .update_config(&id, vec![], vec![], dir.display().to_string())
        .expect("a real directory must be accepted");
    assert_eq!(entry.config.working_dir, Some(dir.display().to_string()));

    let cleared = registry
        .update_config(&id, vec![], vec![], String::new())
        .expect("an empty working directory means the default");
    assert_eq!(
        cleared.config.working_dir, None,
        "empty input must clear the override so DEC-10 applies"
    );
}

#[test]
fn malformed_env_lines_are_refused() {
    let temp = TempRegistry::new("cfg_env_bad");
    for line in ["1BAD=x", "NO_EQUALS", "=value", "BAD KEY=x"] {
        let mut registry = registry_with_entry(&temp);
        let id = registry.list().entries[0].id.clone();
        assert_eq!(
            kind_of(registry.update_config(&id, vec![], vec![line.to_string()], String::new())),
            "badEnvVar",
            "env line {line:?} must be refused"
        );
    }
}

#[test]
fn valid_env_and_args_are_stored_verbatim() {
    let temp = TempRegistry::new("cfg_ok");
    let mut registry = registry_with_entry(&temp);
    let id = registry.list().entries[0].id.clone();

    let entry = registry
        .update_config(
            &id,
            vec!["a b".to_string(), String::new(), "--flag".to_string()],
            vec!["DEMO_VAR=hello world".to_string(), "EMPTY=".to_string()],
            String::new(),
        )
        .expect("a well-formed draft must be accepted");

    assert_eq!(
        entry.config.args,
        vec!["a b".to_string(), "--flag".to_string()],
        "empty lines are skipped, argument lines stay verbatim"
    );
    assert_eq!(entry.config.env.get("DEMO_VAR").map(String::as_str), Some("hello world"));
    assert_eq!(entry.config.env.get("EMPTY").map(String::as_str), Some(""));
}

#[test]
fn unknown_entry_is_not_found() {
    let temp = TempRegistry::new("cfg_unknown");
    let mut registry = registry_with_entry(&temp);

    let result = registry.update_config("no-such-id", vec![], vec![], String::new());
    assert_eq!(kind_of(result), "notFound");
}
