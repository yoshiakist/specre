// @specre 01KHB48DYZDN8GHXPX7MSYJ1NZ
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::fs;

fn specre() -> assert_cmd::Command {
    cargo_bin_cmd!("specre")
}

fn write_config(dir: &std::path::Path, specre_dir: &str, source_dirs: &[&str]) {
    let dirs_toml: Vec<String> = source_dirs.iter().map(|s| format!("\"{s}\"")).collect();
    let content = format!(
        "specre_dir = \"{specre_dir}\"\nsource_dirs = [{}]\n",
        dirs_toml.join(", ")
    );
    fs::write(dir.join("specre.toml"), content).unwrap();
}

fn write_config_with_ext(
    dir: &std::path::Path,
    specre_dir: &str,
    source_dirs: &[&str],
    target_extensions: &[&str],
) {
    let dirs_toml: Vec<String> = source_dirs.iter().map(|s| format!("\"{s}\"")).collect();
    let ext_toml: Vec<String> = target_extensions.iter().map(|s| format!("\"{s}\"")).collect();
    let content = format!(
        "specre_dir = \"{specre_dir}\"\nsource_dirs = [{}]\ntarget_extensions = [{}]\n",
        dirs_toml.join(", "),
        ext_toml.join(", ")
    );
    fs::write(dir.join("specre.toml"), content).unwrap();
}

fn write_specre(dir: &std::path::Path, rel_path: &str, id: &str, name: &str, status: &str) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let fm = format!(
        "---\nid: \"{id}\"\nname: \"{name}\"\nstatus: \"{status}\"\n---\n\n## Related Files\n\n-\n"
    );
    fs::write(path, fm).unwrap();
}

fn write_source(dir: &std::path::Path, rel_path: &str, content: &str) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

// -- Scenario: Basic invocation shows specre and source references --

#[test]
fn trace_shows_specre_and_source_references() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
    );
    write_source(
        tmp.path(),
        "src/example.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n",
    );
    write_source(
        tmp.path(),
        "src/other.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn other() {}\n",
    );

    specre()
        .args(["trace", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Specre:")
                .and(predicate::str::contains("docs/specres/cli/spec_a.md"))
                .and(predicate::str::contains("Source references:"))
                .and(predicate::str::contains("src/example.rs:1"))
                .and(predicate::str::contains("src/other.rs:1")),
        );
}

// -- Scenario: ULID found in specre but no source references --

#[test]
fn trace_specre_found_no_source_refs() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/auth/spec_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "spec_b",
        "draft",
    );
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    specre()
        .args(["trace", "01BBBBBBBBBBBBBBBBBBBBBBBB"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Specre:")
                .and(predicate::str::contains("docs/specres/auth/spec_b.md"))
                .and(predicate::str::contains("Source references:"))
                .and(predicate::str::contains("(none)")),
        );
}

// -- Scenario: ULID found in source but no matching specre --

#[test]
fn trace_source_refs_found_no_specre() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();
    write_source(
        tmp.path(),
        "src/example.rs",
        "// @specre 01CCCCCCCCCCCCCCCCCCCCCCCC\nfn main() {}\n",
    );

    specre()
        .args(["trace", "01CCCCCCCCCCCCCCCCCCCCCCCC"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Specre:")
                .and(predicate::str::contains("(not found)"))
                .and(predicate::str::contains("Source references:"))
                .and(predicate::str::contains("src/example.rs:1")),
        );
}

// -- Scenario: ULID not found anywhere --

#[test]
fn trace_ulid_not_found_exits_with_error() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    specre()
        .args(["trace", "01ZZZZZZZZZZZZZZZZZZZZZZZZ"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("Specre:")
                .and(predicate::str::contains("(not found)"))
                .and(predicate::str::contains("Source references:"))
                .and(predicate::str::contains("(none)")),
        );
}

// -- Scenario: Non-ULID strings are treated as file paths --

#[test]
fn trace_short_string_treated_as_file_path() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    specre()
        .args(["trace", "abc123"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("file not found: abc123"));
}

#[test]
fn trace_lowercase_ulid_treated_as_file_path() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    specre()
        .args(["trace", "01aaaaaaaaaaaaaaaaaaaaaaaa"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "file not found: 01aaaaaaaaaaaaaaaaaaaaaaaa",
        ));
}

// -- Scenario: specre.toml does not exist --

#[test]
fn trace_errors_without_config() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["trace", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "specre.toml not found. Run 'specre init' first.",
        ));
}

// -- Scenario: Paths use forward slashes --

#[test]
fn trace_paths_use_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/auth/signup/spec_x.md",
        "01XXXXXXXXXXXXXXXXXXXXXXXXXX",
        "spec_x",
        "stable",
    );
    write_source(
        tmp.path(),
        "src/auth/signup.rs",
        "// @specre 01XXXXXXXXXXXXXXXXXXXXXXXXXX\n",
    );

    let output = specre()
        .args(["trace", "01XXXXXXXXXXXXXXXXXXXXXXXXXX"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains('\\'), "Paths should use forward slashes");
}

// -- Scenario: Backslash paths are normalized to forward slashes --

#[test]
fn trace_file_path_backslash_is_normalized() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
    );
    write_source(
        tmp.path(),
        "src/nested/file.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n",
    );

    specre()
        .args(["trace", "src\\nested\\file.rs"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("File: src/nested/file.rs")
                .and(predicate::str::contains("01AAAAAAAAAAAAAAAAAAAAAAAA"))
                .and(predicate::str::contains("docs/specres/cli/spec_a.md")),
        );
}

// -- Scenario: Multiple source refs across different files --

#[test]
fn trace_multiple_source_refs_with_line_numbers() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_m.md",
        "01MMMMMMMMMMMMMMMMMMMMMMMM",
        "spec_m",
        "stable",
    );
    write_source(
        tmp.path(),
        "src/a.rs",
        "fn a() {}\n// @specre 01MMMMMMMMMMMMMMMMMMMMMMMM\n",
    );
    write_source(
        tmp.path(),
        "src/b.rs",
        "fn b() {}\nfn c() {}\n// @specre 01MMMMMMMMMMMMMMMMMMMMMMMM\n",
    );

    specre()
        .args(["trace", "01MMMMMMMMMMMMMMMMMMMMMMMM"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("src/a.rs:2").and(predicate::str::contains("src/b.rs:3")));
}

// -- Scenario: Markers inside string literals are ignored --

#[test]
fn trace_ignores_markers_inside_string_literals() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src", "tests"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
    );
    // Real marker
    write_source(
        tmp.path(),
        "src/real.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn real() {}\n",
    );
    // Fake marker inside a string literal in test code
    write_source(
        tmp.path(),
        "tests/test.rs",
        &format!(
            "{}\n{}",
            r#"    write_source(tmp.path(), "src/a.rs", "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n");"#,
            "",
        ),
    );

    specre()
        .args(["trace", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("src/real.rs:1")
                .and(predicate::str::contains("tests/test.rs").not()),
        );
}

// -- Scenario: File path invocation shows linked specres --

#[test]
fn trace_file_path_shows_linked_specres() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
    );
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "spec_b",
        "draft",
    );
    write_source(
        tmp.path(),
        "src/config.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n// @specre 01BBBBBBBBBBBBBBBBBBBBBBBB\nfn config() {}\n",
    );

    specre()
        .args(["trace", "src/config.rs"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("File: src/config.rs")
                .and(predicate::str::contains("Specres:"))
                .and(predicate::str::contains("01AAAAAAAAAAAAAAAAAAAAAAAA"))
                .and(predicate::str::contains("docs/specres/cli/spec_a.md"))
                .and(predicate::str::contains("01BBBBBBBBBBBBBBBBBBBBBBBB"))
                .and(predicate::str::contains("docs/specres/cli/spec_b.md")),
        );
}

// -- Scenario: File path with a ULID that has no matching specre --

#[test]
fn trace_file_path_unresolved_ulid_shows_not_found() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();
    write_source(
        tmp.path(),
        "src/example.rs",
        "// @specre 01ZZZZZZZZZZZZZZZZZZZZZZZZ\nfn main() {}\n",
    );

    specre()
        .args(["trace", "src/example.rs"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("File: src/example.rs")
                .and(predicate::str::contains("01ZZZZZZZZZZZZZZZZZZZZZZZZ"))
                .and(predicate::str::contains("(not found)")),
        );
}

// -- Scenario: File path with no markers --

#[test]
fn trace_file_path_no_markers_exits_with_error() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();
    write_source(tmp.path(), "src/utils.rs", "fn helper() {}\n");

    specre()
        .args(["trace", "src/utils.rs"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("File: src/utils.rs")
                .and(predicate::str::contains("Specres:"))
                .and(predicate::str::contains("(none)")),
        );
}

// -- Scenario: File does not exist --

#[test]
fn trace_file_path_not_found() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    specre()
        .args(["trace", "src/nonexistent.rs"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "file not found: src/nonexistent.rs",
        ));
}

// -- Scenario: File path output uses forward slashes --

#[test]
fn trace_file_path_output_uses_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
    );
    write_source(
        tmp.path(),
        "src/nested/deep/file.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n",
    );

    let output = specre()
        .args(["trace", "src/nested/deep/file.rs"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains('\\'), "Paths should use forward slashes");
}

// -- Scenario: File path ignores markers inside string literals --

#[test]
fn trace_file_path_ignores_string_literal_markers() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
    );
    write_source(
        tmp.path(),
        "src/test_helper.rs",
        &format!(
            "{}\n{}\n",
            "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA",
            r#"    let s = "// @specre 01BBBBBBBBBBBBBBBBBBBBBBBB";"#,
        ),
    );

    specre()
        .args(["trace", "src/test_helper.rs"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("01AAAAAAAAAAAAAAAAAAAAAAAA")
                .and(predicate::str::contains("01BBBBBBBBBBBBBBBBBBBBBBBB").not()),
        );
}

// -- Scenario: target_extensions filters source files in ULID mode --

#[test]
fn trace_ulid_target_extensions_filters_source_files() {
    let tmp = TempDir::new().unwrap();
    write_config_with_ext(tmp.path(), "docs/specres", &["src"], &["rs"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
    );
    // .rs file — should be found
    write_source(
        tmp.path(),
        "src/main.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n",
    );
    // .py file — should be skipped due to target_extensions
    write_source(
        tmp.path(),
        "src/helper.py",
        "# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\ndef helper(): pass\n",
    );

    specre()
        .args(["trace", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("src/main.rs:1")
                .and(predicate::str::contains("helper.py").not()),
        );
}

// -- Scenario: trace file-path mode is unaffected by target_extensions --

#[test]
fn trace_file_path_ignores_target_extensions() {
    let tmp = TempDir::new().unwrap();
    write_config_with_ext(tmp.path(), "docs/specres", &["src"], &["rs"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
    );
    // .py file — not in target_extensions, but trace by file path should still work
    write_source(
        tmp.path(),
        "src/helper.py",
        "# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\ndef helper(): pass\n",
    );

    specre()
        .args(["trace", "src/helper.py"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("File: src/helper.py")
                .and(predicate::str::contains("01AAAAAAAAAAAAAAAAAAAAAAAA"))
                .and(predicate::str::contains("docs/specres/cli/spec_a.md")),
        );
}

// -- Failures / Exceptions: IO error warns and skips --

#[cfg(unix)]
#[test]
fn trace_ulid_warns_on_unreadable_specre_file() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    // Create a readable specre card
    write_specre(tmp.path(), "docs/specres/cli/good.md", "01AAAAAAAAAAAAAAAAAAAAAAAA", "good");

    // Create an unreadable specre card
    let bad_card = tmp.path().join("docs/specres/cli/bad.md");
    fs::create_dir_all(bad_card.parent().unwrap()).unwrap();
    fs::write(
        &bad_card,
        "---\nid: \"01BBBBBBBBBBBBBBBBBBBBBBBB\"\nname: \"bad\"\nstatus: \"draft\"\n---\n",
    )
    .unwrap();
    fs::set_permissions(&bad_card, fs::Permissions::from_mode(0o000)).unwrap();

    let output = specre()
        .args(["trace", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning: failed to read"),
        "Expected warning about unreadable file in stderr, got: {stderr}"
    );

    fs::set_permissions(&bad_card, fs::Permissions::from_mode(0o644)).unwrap();
}

#[cfg(unix)]
#[test]
fn trace_ulid_warns_on_unreadable_source_file() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre(tmp.path(), "docs/specres/cli/spec_a.md", "01AAAAAAAAAAAAAAAAAAAAAAAA", "spec_a");
    write_source(tmp.path(), "src/good.rs", "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n");

    let bad_file = tmp.path().join("src/bad.rs");
    fs::write(&bad_file, "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n").unwrap();
    fs::set_permissions(&bad_file, fs::Permissions::from_mode(0o000)).unwrap();

    let output = specre()
        .args(["trace", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning: failed to read"),
        "Expected warning about unreadable source file in stderr, got: {stderr}"
    );

    // The good source ref should still be found
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("src/good.rs"));

    fs::set_permissions(&bad_file, fs::Permissions::from_mode(0o644)).unwrap();
}
