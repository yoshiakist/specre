// @specre 01KJYMAV7G01B743W72WAG9RGN
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

fn specre() -> assert_cmd::Command {
    cargo_bin_cmd!("specre")
}

fn setup_project(tmp: &TempDir) {
    // Create specre.toml
    tmp.child("specre.toml")
        .write_str(
            r#"specre_dir = "docs/specres"
source_dirs = ["src"]
"#,
        )
        .unwrap();

    // Create glossary.toml
    tmp.child("glossary.toml")
        .write_str("terms = [\"user\"]\n")
        .unwrap();

    // Create specre cards directory
    let specres_dir = tmp.path().join("docs/specres");
    fs::create_dir_all(&specres_dir).unwrap();
    fs::write(
        specres_dir.join("some_card.md"),
        "---\nid: \"01AAAAAAAAAAAAAAAAAAAAAAAA\"\nname: \"test\"\nstatus: \"stable\"\n---\n",
    )
    .unwrap();

    // Create source files with markers
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("main.rs"),
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n",
    )
    .unwrap();
}

// --- Scenario 1: Happy path — keep specre cards directory ---

#[test]
fn destroy_removes_markers_and_config_keeps_specre_dir() {
    let tmp = TempDir::new().unwrap();
    setup_project(&tmp);

    specre()
        .args(["destroy"])
        .current_dir(tmp.path())
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Done. To remove the specre binary, run: cargo uninstall specre",
        ));

    // specre.toml should be deleted
    assert!(!tmp.path().join("specre.toml").exists());

    // glossary.toml should be deleted
    assert!(!tmp.path().join("glossary.toml").exists());

    // specre cards directory should still exist
    assert!(tmp.path().join("docs/specres").is_dir());

    // @specre marker should be removed from source file
    let content = fs::read_to_string(tmp.path().join("src/main.rs")).unwrap();
    assert!(!content.contains("@specre"));
    assert!(content.contains("fn main() {}"));
}

#[test]
fn destroy_default_answer_keeps_specre_dir() {
    let tmp = TempDir::new().unwrap();
    setup_project(&tmp);

    // Press Enter (empty input = default N)
    specre()
        .args(["destroy"])
        .current_dir(tmp.path())
        .write_stdin("\n")
        .assert()
        .success();

    // specre cards directory should still exist
    assert!(tmp.path().join("docs/specres").is_dir());
}

// --- Scenario 2: Happy path — also delete specre cards directory ---

#[test]
fn destroy_with_yes_deletes_specre_dir() {
    let tmp = TempDir::new().unwrap();
    setup_project(&tmp);

    specre()
        .args(["destroy"])
        .current_dir(tmp.path())
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Done. To remove the specre binary, run: cargo uninstall specre",
        ));

    // specre.toml should be deleted
    assert!(!tmp.path().join("specre.toml").exists());

    // specre cards directory should be deleted
    assert!(!tmp.path().join("docs/specres").exists());

    // marker removed from source
    let content = fs::read_to_string(tmp.path().join("src/main.rs")).unwrap();
    assert!(!content.contains("@specre"));
}

// --- Scenario 3: No specre.toml — error ---

#[test]
fn destroy_errors_when_no_config() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["destroy"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("specre.toml"));
}

// --- Scenario 4: Source file with no markers — left unchanged ---

#[test]
fn destroy_leaves_files_without_markers_unchanged() {
    let tmp = TempDir::new().unwrap();
    setup_project(&tmp);

    // Add a file with no markers
    let src_dir = tmp.path().join("src");
    fs::write(
        src_dir.join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    )
    .unwrap();

    specre()
        .args(["destroy"])
        .current_dir(tmp.path())
        .write_stdin("n\n")
        .assert()
        .success();

    // Untouched file should be unchanged
    let content = fs::read_to_string(tmp.path().join("src/lib.rs")).unwrap();
    assert_eq!(content, "pub fn add(a: i32, b: i32) -> i32 { a + b }\n");
}

// --- Scenario 5: Source file with multiple markers — all removed ---

#[test]
fn destroy_removes_multiple_markers_from_single_file() {
    let tmp = TempDir::new().unwrap();
    setup_project(&tmp);

    // Overwrite main.rs with two markers
    let src_dir = tmp.path().join("src");
    fs::write(
        src_dir.join("main.rs"),
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n// @specre 01BBBBBBBBBBBBBBBBBBBBBBBB\nfn main() {}\n",
    )
    .unwrap();

    specre()
        .args(["destroy"])
        .current_dir(tmp.path())
        .write_stdin("n\n")
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/main.rs")).unwrap();
    assert!(!content.contains("@specre"));
    assert!(content.contains("fn main() {}"));
}

// --- Scenario 6: glossary.toml does not exist — silently skipped ---

#[test]
fn destroy_succeeds_without_glossary_toml() {
    let tmp = TempDir::new().unwrap();
    // Setup without glossary.toml
    tmp.child("specre.toml")
        .write_str(
            r#"specre_dir = "docs/specres"
source_dirs = ["src"]
"#,
        )
        .unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}\n").unwrap();
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();

    specre()
        .args(["destroy"])
        .current_dir(tmp.path())
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Done. To remove the specre binary, run: cargo uninstall specre",
        ));
}

// --- Scenario 7: Lines containing @specre that are NOT markers — preserved ---

#[test]
fn destroy_preserves_indented_marker_lines() {
    let tmp = TempDir::new().unwrap();
    setup_project(&tmp);

    // Indented @specre line — condition 1 fails (has leading whitespace)
    let src_dir = tmp.path().join("src");
    let indented = "fn foo() {\n    // @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n}\n";
    fs::write(src_dir.join("lib.rs"), indented).unwrap();

    specre()
        .args(["destroy"])
        .current_dir(tmp.path())
        .write_stdin("n\n")
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/lib.rs")).unwrap();
    assert_eq!(
        content, indented,
        "indented @specre line must not be removed"
    );
}

#[test]
fn destroy_preserves_string_literal_containing_specre() {
    let tmp = TempDir::new().unwrap();
    setup_project(&tmp);

    // String literal — condition 2 fails (quote precedes @specre)
    let src_dir = tmp.path().join("src");
    let with_string = "let s = \"// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\";\n";
    fs::write(src_dir.join("lib.rs"), with_string).unwrap();

    specre()
        .args(["destroy"])
        .current_dir(tmp.path())
        .write_stdin("n\n")
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/lib.rs")).unwrap();
    assert_eq!(
        content, with_string,
        "string literal containing @specre must not be removed"
    );
}

#[test]
fn destroy_preserves_prose_comment_mentioning_specre() {
    let tmp = TempDir::new().unwrap();
    setup_project(&tmp);

    // Prose comment — condition 3 fails (prefix "# See " has embedded space)
    let src_dir = tmp.path().join("src");
    let prose = "# See @specre 01AAAAAAAAAAAAAAAAAAAAAAAA for details\n";
    fs::write(src_dir.join("lib.rs"), prose).unwrap();

    specre()
        .args(["destroy"])
        .current_dir(tmp.path())
        .write_stdin("n\n")
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/lib.rs")).unwrap();
    assert_eq!(
        content, prose,
        "prose comment mentioning @specre must not be removed"
    );
}

// --- Output: prompt text ---

#[test]
fn destroy_prints_warning_and_prompt() {
    let tmp = TempDir::new().unwrap();
    setup_project(&tmp);

    specre()
        .args(["destroy"])
        .current_dir(tmp.path())
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "This will remove all @specre markers from your source files and delete specre.toml.",
            )
            .and(predicate::str::contains("docs/specres")),
        );
}
