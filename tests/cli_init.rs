// @specre 01KHAGG8NQQ7RSNYZ6SWBCYH3N
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

fn specre() -> assert_cmd::Command {
    cargo_bin_cmd!("specre")
}

#[test]
fn init_creates_default_directory_and_config() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Created docs/specres/")
                .and(predicate::str::contains("Created specre.toml")),
        );

    assert!(tmp.path().join("docs/specres").is_dir());
    assert!(tmp.path().join("specre.toml").is_file());
}

#[test]
fn init_config_contains_correct_defaults() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();
    assert!(content.contains(r#"specre_dir = "docs/specres""#));
    assert!(content.contains(r#"source_dirs = ["src"]"#));
}

#[test]
fn init_custom_specre_dir() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init", "--specre-dir", "specs/specres"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created specs/specres/"));

    assert!(tmp.path().join("specs/specres").is_dir());

    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();
    assert!(content.contains(r#"specre_dir = "specs/specres""#));
}

#[test]
fn init_custom_source_dirs() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init", "--source-dirs", "src,lib"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();
    assert!(content.contains(r#"source_dirs = ["src", "lib"]"#));
}

#[test]
fn init_errors_when_config_already_exists() {
    let tmp = TempDir::new().unwrap();
    tmp.child("specre.toml").touch().unwrap();

    specre()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "specre.toml already exists. This project is already initialized.",
        ));
}

#[test]
fn init_preserves_existing_specre_directory() {
    let tmp = TempDir::new().unwrap();

    // Create specre dir with an existing file
    let specre_dir = tmp.path().join("docs/specres");
    fs::create_dir_all(&specre_dir).unwrap();
    fs::write(specre_dir.join("existing.md"), "keep me").unwrap();

    specre()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Exists  docs/specres/")
                .and(predicate::str::contains("Created specre.toml")),
        );

    // Existing file should be untouched
    let existing = fs::read_to_string(specre_dir.join("existing.md")).unwrap();
    assert_eq!(existing, "keep me");

    // Config should still be created
    assert!(tmp.path().join("specre.toml").is_file());
}

#[test]
fn init_does_not_modify_anything_when_config_exists() {
    let tmp = TempDir::new().unwrap();
    tmp.child("specre.toml").write_str("original").unwrap();

    specre()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .failure();

    // Config should not be modified
    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();
    assert_eq!(content, "original");

    // specre directory should not be created
    assert!(!tmp.path().join("docs/specres").exists());
}

#[test]
fn init_with_language_ja() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init", "--language", "ja"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();
    assert!(content.contains(r#"language = "ja""#));
}

#[test]
fn init_without_language_omits_field() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();
    assert!(!content.contains("language"));
}

// -- Scenario: Custom target extensions --

#[test]
fn init_with_ext_creates_target_extensions() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init", "--ext", "rs,ts"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();
    assert!(content.contains(r#"target_extensions = ["rs", "ts"]"#));
}

#[test]
fn init_without_ext_omits_target_extensions() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();
    assert!(!content.contains("target_extensions"));
}
