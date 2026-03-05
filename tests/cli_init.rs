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
fn init_without_language_has_language_as_comment() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();
    // Without --language, the field should appear only as a commented-out example, not as an active setting.
    assert!(content.contains("# language"));
    assert!(!content.contains("\nlanguage"));
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
fn init_without_ext_has_target_extensions_as_comment() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();
    // Without --ext, the field should appear only as a commented-out example, not as an active setting.
    assert!(content.contains("# target_extensions"));
    assert!(!content.contains("\ntarget_extensions"));
}

// -- Scenario: Default init includes commented-out optional settings --

#[test]
fn init_default_includes_commented_optional_settings() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();
    assert!(content.contains("# target_extensions"));
    assert!(content.contains("# exclude_patterns"));
    assert!(content.contains("# language"));
    assert!(content.contains("# [health_check]"));
    assert!(content.contains("# coverage = 0.30"));
    assert!(content.contains("# orphans = 10"));
    assert!(content.contains("# index_age_hours = 48"));
}

#[test]
fn init_with_ext_omits_target_extensions_comment() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init", "--ext", "rs,ts"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();
    // Active setting is present; commented-out example should be absent.
    assert!(content.contains(r#"target_extensions = ["rs", "ts"]"#));
    assert!(!content.contains("# target_extensions"));
}

#[test]
fn init_with_language_omits_language_comment() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init", "--language", "en"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();
    // Active setting is present; commented-out example should be absent.
    assert!(content.contains(r#"language = "en""#));
    assert!(!content.contains("# language"));
}

// -- Scenario: Special characters in arguments --

#[test]
fn init_escapes_backslash_in_specre_dir() {
    let tmp = TempDir::new().unwrap();

    // A backslash in specre_dir (common on Windows paths) would produce invalid TOML
    // when manually formatted: `specre_dir = "my\ndir"` would be parsed as a newline.
    // With proper toml::to_string() serialization, it becomes `specre_dir = "my\\ndir"`.
    specre()
        .args(["init", "--specre-dir", r"my\ndir"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();

    // The generated TOML must be parseable and round-trip correctly
    let parsed: toml::Value = toml::from_str(&content).expect("generated TOML must be valid");
    assert_eq!(
        parsed["specre_dir"].as_str().unwrap(),
        r"my\ndir",
        "specre_dir value must round-trip through TOML correctly"
    );
}

#[test]
fn init_escapes_backslash_in_source_dirs() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init", "--source-dirs", r"src\main,lib\\test"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("specre.toml")).unwrap();

    let parsed: toml::Value = toml::from_str(&content).expect("generated TOML must be valid");
    let dirs = parsed["source_dirs"].as_array().unwrap();
    assert_eq!(dirs[0].as_str().unwrap(), r"src\main");
    assert_eq!(dirs[1].as_str().unwrap(), r"lib\\test");
}

// -- Failures / Exceptions: SpecreError::Io --

#[test]
fn init_shows_path_in_io_error() {
    let tmp = TempDir::new().unwrap();

    // Create a file that blocks create_dir_all from creating the specre directory
    tmp.child("blocker").touch().unwrap();

    specre()
        .args(["init", "--specre-dir", "blocker/specres"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Failed to access").and(predicate::str::contains("blocker")),
        );
}

// -- Scenario: glossary.toml is created by init --

#[test]
fn init_creates_glossary_toml() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created glossary.toml"));

    let glossary_path = tmp.path().join("glossary.toml");
    assert!(glossary_path.is_file());

    let content = fs::read_to_string(&glossary_path).unwrap();
    // Should be valid TOML with a terms array
    let parsed: toml::Value = toml::from_str(&content).expect("glossary.toml must be valid TOML");
    let terms = parsed["terms"].as_array().unwrap();
    assert!(!terms.is_empty(), "glossary should have sample terms");
}

// -- Scenario: glossary.toml already exists — preserved --

#[test]
fn init_preserves_existing_glossary() {
    let tmp = TempDir::new().unwrap();

    // Create existing glossary.toml with custom content
    let glossary_path = tmp.path().join("glossary.toml");
    fs::write(&glossary_path, "terms = [\"custom\"]\n").unwrap();

    specre()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Exists  glossary.toml"));

    // Content should be unchanged
    let content = fs::read_to_string(&glossary_path).unwrap();
    assert_eq!(content, "terms = [\"custom\"]\n");
}
