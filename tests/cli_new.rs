// @specre 01JMBJK7QRVX3N4P5G6H8W9Y0Z
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

fn specre() -> assert_cmd::Command {
    cargo_bin_cmd!("specre")
}

#[test]
fn new_creates_file_with_name() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().to_str().unwrap();

    specre()
        .args(["new", target, "--name", "user_can_sign_up"])
        .assert()
        .success()
        .stdout(predicate::str::contains("user_can_sign_up.md"));

    let file = tmp.child("user_can_sign_up.md");
    file.assert(predicate::path::exists());
}

#[test]
fn new_defaults_to_untitled() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().to_str().unwrap();

    specre()
        .args(["new", target])
        .assert()
        .success()
        .stdout(predicate::str::contains("untitled.md"));

    let file = tmp.child("untitled.md");
    file.assert(predicate::path::exists());
}

#[test]
fn new_creates_directory_recursively() {
    let tmp = TempDir::new().unwrap();
    let nested = tmp.path().join("a").join("b").join("c");
    let target = nested.to_str().unwrap();

    specre()
        .args(["new", target, "--name", "foo"])
        .assert()
        .success();

    let file = nested.join("foo.md");
    assert!(file.exists());
}

#[test]
fn new_file_contains_valid_frontmatter() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().to_str().unwrap();

    specre()
        .args(["new", target, "--name", "my_specre"])
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("my_specre.md")).unwrap();
    assert!(content.starts_with("---\n"));
    assert!(content.contains(r#"name: "my_specre""#));
    assert!(content.contains(r#"status: "draft""#));
    assert!(content.contains("id: \""));
}

#[test]
fn new_file_contains_valid_ulid() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().to_str().unwrap();

    specre()
        .args(["new", target, "--name", "test_ulid"])
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("test_ulid.md")).unwrap();
    let id_line = content.lines().find(|l| l.starts_with("id:")).unwrap();
    let id = id_line.trim_start_matches("id: \"").trim_end_matches('"');
    assert_eq!(id.len(), 26);
    assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn new_file_contains_required_sections() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().to_str().unwrap();

    specre()
        .args(["new", target, "--name", "sections_test"])
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("sections_test.md")).unwrap();
    assert!(content.contains("## Related Files"));
    assert!(content.contains("## Functional Overview"));
    assert!(content.contains("## Scenarios"));
}

#[test]
fn new_errors_when_file_already_exists() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().to_str().unwrap();

    tmp.child("duplicate.md").touch().unwrap();

    specre()
        .args(["new", target, "--name", "duplicate"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn new_errors_when_target_is_a_file() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.child("not_a_dir");
    file.touch().unwrap();
    let target = file.path().to_str().unwrap();

    specre()
        .args(["new", target, "--name", "anything"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("is a file, not a directory"));
}

#[test]
fn new_stdout_is_the_created_path() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().to_str().unwrap();

    let expected_suffix = format!("{}my_spec.md", std::path::MAIN_SEPARATOR);

    specre()
        .args(["new", target, "--name", "my_spec"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(expected_suffix)
                .and(predicate::str::is_match("^[^\n]+\n$").unwrap()),
        );
}
