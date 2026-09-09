use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn version_works() {
    let mut command = Command::cargo_bin("pdecl").unwrap();

    command
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("pdecl"));
}

#[test]
fn rejects_invalid_config() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("packages.toml");

    fs::write(
        &config,
        r#"
version = 99

packages = [
    "git",
]
"#,
    )
    .unwrap();

    let mut command = Command::cargo_bin("pdecl").unwrap();

    command
        .args(["diff", "--file"])
        .arg(&config)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unsupported config version",
        ));
}

#[test]
fn rejects_duplicate_packages() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("packages.toml");

    fs::write(
        &config,
        r#"
version = 1

packages = [
    "git",
    "git",
]
"#,
    )
    .unwrap();

    let mut command = Command::cargo_bin("pdecl").unwrap();

    command
        .args(["diff", "--file"])
        .arg(&config)
        .assert()
        .failure()
        .stderr(predicate::str::contains("duplicate package"));
}

#[test]
fn rejects_empty_apply_without_allow_empty() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("packages.toml");

    fs::write(
        &config,
        r#"
version = 1
packages = []
"#,
    )
    .unwrap();

    let mut command = Command::cargo_bin("pdecl").unwrap();

    command
        .args(["apply", "--file"])
        .arg(&config)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Refusing to remove every",
        ));
}
