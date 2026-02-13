// @specre 01KHB48EYB9686YYQMYFYQ5R1Z
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::fs;

fn specre() -> assert_cmd::Command {
    cargo_bin_cmd!("specre")
}

fn write_source(dir: &std::path::Path, rel_path: &str, content: &str) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

// -- Scenario: Basic invocation inserts marker at line 1 --

#[test]
fn tag_inserts_marker_for_rust_file() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/example.rs", "fn main() {}\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/example.rs"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Tagged src/example.rs with 01AAAAAAAAAAAAAAAAAAAAAAAA",
        ));

    let content = fs::read_to_string(tmp.path().join("src/example.rs")).unwrap();
    assert_eq!(
        content,
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n"
    );
}

// -- Scenario: Comment syntax varies by language --

#[test]
fn tag_uses_hash_comment_for_python() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/app.py", "print('hello')\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/app.py"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/app.py")).unwrap();
    assert!(content.starts_with("# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

#[test]
fn tag_uses_hash_comment_for_ruby() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/app.rb", "puts 'hello'\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/app.rb"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/app.rb")).unwrap();
    assert!(content.starts_with("# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

#[test]
fn tag_uses_block_comment_for_css() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/style.css", "body { color: red; }\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/style.css"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/style.css")).unwrap();
    assert!(content.starts_with("/* @specre 01AAAAAAAAAAAAAAAAAAAAAAAA */\n"));
}

#[test]
fn tag_uses_html_comment_for_html() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/page.html", "<html></html>\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/page.html"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/page.html")).unwrap();
    assert!(content.starts_with("<!-- @specre 01AAAAAAAAAAAAAAAAAAAAAAAA -->\n"));
}

#[test]
fn tag_uses_dash_dash_comment_for_sql() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/query.sql", "SELECT 1;\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/query.sql"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/query.sql")).unwrap();
    assert!(content.starts_with("-- @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

#[test]
fn tag_uses_slash_slash_for_unknown_extension() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/file.xyz", "data\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/file.xyz"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/file.xyz")).unwrap();
    assert!(content.starts_with("// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: Marker already exists in file --

#[test]
fn tag_already_exists_does_not_modify() {
    let tmp = TempDir::new().unwrap();
    let original = "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n";
    write_source(tmp.path(), "src/example.rs", original);

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/example.rs"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Marker already exists in src/example.rs",
        ));

    let content = fs::read_to_string(tmp.path().join("src/example.rs")).unwrap();
    assert_eq!(content, original);
}

// -- Scenario: File does not exist --

#[test]
fn tag_file_not_found() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/nonexistent.rs"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "file not found: src/nonexistent.rs",
        ));
}

// -- Scenario: Invalid ULID format --

#[test]
fn tag_invalid_ulid() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/example.rs", "fn main() {}\n");

    specre()
        .args(["tag", "abc123", "src/example.rs"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid ULID format. Expected 26 uppercase alphanumeric characters.",
        ));
}

// -- Scenario: Target is a directory --

#[test]
fn tag_target_is_directory() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("is a directory, not a file"));
}

// -- Scenario: Preserves existing file content --

#[test]
fn tag_preserves_existing_content() {
    let tmp = TempDir::new().unwrap();
    let original = "line1\nline2\nline3\n";
    write_source(tmp.path(), "src/example.js", original);

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/example.js"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/example.js")).unwrap();
    assert_eq!(
        content,
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nline1\nline2\nline3\n"
    );
}

// -- Scenario: Paths use forward slashes in output --

#[test]
fn tag_output_uses_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/nested/deep/example.rs", "fn main() {}\n");

    let output = specre()
        .args([
            "tag",
            "01AAAAAAAAAAAAAAAAAAAAAAAA",
            "src/nested/deep/example.rs",
        ])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("src/nested/deep/example.rs"),
        "Output should contain forward-slash paths"
    );
    assert!(!stdout.contains('\\'), "Output should not contain backslashes");
}

// -- Scenario: Hash comment for shell files --

#[test]
fn tag_uses_hash_comment_for_shell() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "scripts/deploy.sh", "#!/bin/bash\necho hi\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "scripts/deploy.sh"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("scripts/deploy.sh")).unwrap();
    assert!(content.starts_with("# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: Hash comment for YAML files --

#[test]
fn tag_uses_hash_comment_for_yaml() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "config/settings.yml", "key: value\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "config/settings.yml"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("config/settings.yml")).unwrap();
    assert!(content.starts_with("# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}
