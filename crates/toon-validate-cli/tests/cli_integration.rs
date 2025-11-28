use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_analyze_command_with_toon_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.toon");
    fs::write(&file_path, r#"name: "test"
age: 42
active: true"#).unwrap();

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "tval", "--"]);
    cmd.arg("analyze")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Total Estimated Tokens"))
        .stdout(predicate::str::contains("Keys"))
        .stdout(predicate::str::contains("Strings"));
}

#[test]
fn test_analyze_command_with_json_output() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.json");
    fs::write(&file_path, r#"{"name": "test", "age": 42}"#).unwrap();

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "tval", "--"]);
    cmd.arg("analyze")
        .arg(&file_path)
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total_tokens\""))
        .stdout(predicate::str::contains("\"breakdown\""));
}

#[test]
fn test_check_command_valid_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("valid.toon");
    fs::write(&file_path, r#"users[2]:
  - id: 1
    name: "Alice"
  - id: 2
    name: "Bob""#).unwrap();

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "tval", "--"]);
    cmd.arg("check")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: VALID"));
}

#[test]
fn test_check_command_invalid_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("invalid.toon");
    fs::write(&file_path, r#"users[3]:
  - id: 1
    name: "Alice"
  - id: 2
    name: "Bob""#).unwrap();

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "tval", "--"]);
    cmd.arg("check")
        .arg(&file_path)
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::contains("Status: INVALID"))
        .stdout(predicate::str::contains("declared with 3 rows but found 2"));
}

#[test]
fn test_profile_command() {
    let dir = tempdir().unwrap();
    
    fs::write(dir.path().join("file1.toon"), "name: \"test1\"").unwrap();
    fs::write(dir.path().join("file2.json"), "{\"name\": \"test2\"}").unwrap();
    
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "tval", "--"]);
    cmd.arg("profile")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Total Files"))
        .stdout(predicate::str::contains("Total Estimated Tokens"));
}

#[test]
fn test_help_command() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "tval", "--"]);
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("TOON Analyzer"))
        .stdout(predicate::str::contains("analyze"))
        .stdout(predicate::str::contains("profile"))
        .stdout(predicate::str::contains("check"));
}