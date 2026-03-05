// @specre 01KHB48EES4FR5TFV6ZP2W3MGT
mod common;
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use common::write_config_with_exclude;
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
    let ext_toml: Vec<String> = target_extensions
        .iter()
        .map(|s| format!("\"{s}\""))
        .collect();
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

// -- Scenario: No orphans or dangling markers --

#[test]
fn orphans_clean_project() {
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
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No orphans or dangling markers found.",
        ));
}

// -- Scenario: Orphan specres detected --

#[test]
fn orphans_detects_orphan_specres() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/auth/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
    );
    write_specre(
        tmp.path(),
        "docs/specres/cart/spec_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "spec_b",
        "draft",
    );
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("Orphan specres (no source markers):")
                .and(predicate::str::contains("docs/specres/auth/spec_a.md"))
                .and(predicate::str::contains("docs/specres/cart/spec_b.md")),
        );
}

// -- Scenario: Dangling markers detected --

#[test]
fn orphans_detects_dangling_markers() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();
    write_source(
        tmp.path(),
        "src/example.rs",
        "// @specre 01CCCCCCCCCCCCCCCCCCCCCCCC\nfn main() {}\n",
    );
    write_source(
        tmp.path(),
        "src/other.rs",
        "// @specre 01DDDDDDDDDDDDDDDDDDDDDDDD\n",
    );

    specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("Dangling markers (no matching specre):")
                .and(predicate::str::contains("src/example.rs:1"))
                .and(predicate::str::contains("01CCCCCCCCCCCCCCCCCCCCCCCC"))
                .and(predicate::str::contains("src/other.rs:1"))
                .and(predicate::str::contains("01DDDDDDDDDDDDDDDDDDDDDDDD")),
        );
}

// -- Scenario: Both orphans and dangling markers --

#[test]
fn orphans_detects_both() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/auth/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
    );
    write_source(
        tmp.path(),
        "src/example.rs",
        "// @specre 01CCCCCCCCCCCCCCCCCCCCCCCC\n",
    );

    specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("Orphan specres (no source markers):")
                .and(predicate::str::contains("docs/specres/auth/spec_a.md"))
                .and(predicate::str::contains(
                    "Dangling markers (no matching specre):",
                ))
                .and(predicate::str::contains("src/example.rs:1"))
                .and(predicate::str::contains("01CCCCCCCCCCCCCCCCCCCCCCCC")),
        );
}

// -- Scenario: Deprecated specres are excluded from orphan detection --

#[test]
fn orphans_excludes_deprecated_specres() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/old_spec.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "old_spec",
        "deprecated",
    );
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No orphans or dangling markers found.",
        ));
}

// -- Scenario: specre.toml does not exist --

#[test]
fn orphans_errors_without_config() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "specre.toml not found. Run 'specre init' first.",
        ));
}

// -- Scenario: Empty project --

#[test]
fn orphans_empty_project() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No orphans or dangling markers found.",
        ));
}

// -- Scenario: Paths use forward slashes --

#[test]
fn orphans_paths_use_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/auth/signup/spec_x.md",
        "01XXXXXXXXXXXXXXXXXXXXXXXXXX",
        "spec_x",
        "stable",
    );
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    let output = specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains('\\'), "Paths should use forward slashes");
}

// -- Scenario: Specre with marker is not orphan --

#[test]
fn orphans_linked_specre_is_not_orphan() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/linked.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "linked",
        "stable",
    );
    write_specre(
        tmp.path(),
        "docs/specres/cli/orphan.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "orphan",
        "draft",
    );
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n",
    );

    specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("Orphan specres (no source markers):")
                .and(predicate::str::contains("docs/specres/cli/orphan.md"))
                .and(predicate::str::contains("docs/specres/cli/linked.md").not()),
        );
}

// -- Scenario: Markers inside string literals are ignored --

#[test]
fn orphans_ignores_markers_inside_string_literals() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src", "tests"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
    );
    // Only a fake marker inside a string literal — should NOT count as a real marker
    write_source(
        tmp.path(),
        "tests/test.rs",
        r#"    write_source(tmp.path(), "src/a.rs", "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n");"#,
    );
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    // spec_a should be an orphan because the only "marker" is inside a string literal
    specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("Orphan specres (no source markers):")
                .and(predicate::str::contains("docs/specres/cli/spec_a.md")),
        );
}

#[test]
fn orphans_string_literal_marker_not_dangling() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src", "tests"]);
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();
    // A fake marker inside a string literal referencing a nonexistent specre
    write_source(
        tmp.path(),
        "tests/test.rs",
        r#"    write_source(tmp.path(), "src/a.rs", "// @specre 01ZZZZZZZZZZZZZZZZZZZZZZZZ\n");"#,
    );
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    // Should NOT report the fake marker as dangling
    specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No orphans or dangling markers found.",
        ));
}

// -- Scenario: target_extensions filters source scanning --

#[test]
fn orphans_target_extensions_filters_source_scanning() {
    let tmp = TempDir::new().unwrap();
    write_config_with_ext(tmp.path(), "docs/specres", &["src"], &["rs"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
    );
    // Marker in .py file — should be ignored due to target_extensions
    write_source(
        tmp.path(),
        "src/helper.py",
        "# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\ndef helper(): pass\n",
    );
    // No .rs files with the marker
    write_source(tmp.path(), "src/main.rs", "fn main() {}\n");

    // spec_a should be orphan because .py is not in target_extensions
    specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("Orphan specres (no source markers):")
                .and(predicate::str::contains("docs/specres/cli/spec_a.md")),
        );
}

#[test]
fn orphans_target_extensions_hides_dangling_in_non_target_files() {
    let tmp = TempDir::new().unwrap();
    write_config_with_ext(tmp.path(), "docs/specres", &["src"], &["rs"]);
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();
    // Dangling marker in .py — should NOT be reported since .py is not a target
    write_source(
        tmp.path(),
        "src/helper.py",
        "# @specre 01ZZZZZZZZZZZZZZZZZZZZZZZZ\n",
    );
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No orphans or dangling markers found.",
        ));
}

// -- Scenario: Binary files in source directories are skipped --

#[test]
fn orphans_skips_binary_files_silently() {
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
        "src/main.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n",
    );
    // Write a binary file (invalid UTF-8) into src/
    let binary_path = tmp.path().join("src/logo.png");
    fs::write(&binary_path, b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00").unwrap();

    let output = specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Binary file should not produce any warning on stderr
    assert!(
        !stderr.contains("Warning"),
        "Binary file should not produce a warning, got: {stderr}"
    );
    // Orphan detection should still work correctly
    assert!(
        stdout.contains("No orphans or dangling markers found."),
        "Expected clean orphans output, got: {stdout}"
    );
    assert!(output.status.success());
}

// -- Failures / Exceptions: IO error warns and skips --

#[cfg(unix)]
#[test]
fn orphans_warns_on_unreadable_source_file() {
    use std::os::unix::fs::PermissionsExt;
    if common::is_root() {
        return; // root bypasses file-permission checks
    }

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
        "src/good.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n",
    );

    let bad_file = tmp.path().join("src/bad.rs");
    fs::write(&bad_file, "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n").unwrap();
    fs::set_permissions(&bad_file, fs::Permissions::from_mode(0o000)).unwrap();

    let output = specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning: failed to read"),
        "Expected warning about unreadable source file in stderr, got: {stderr}"
    );

    fs::set_permissions(&bad_file, fs::Permissions::from_mode(0o644)).unwrap();
}

// -- Scenario: exclude_patterns hides dangling markers in excluded files --

#[test]
fn orphans_exclude_patterns_hides_dangling_in_excluded_files() {
    let tmp = TempDir::new().unwrap();
    write_config_with_exclude(tmp.path(), "docs/specres", &["src"], &["*.test.ts"]);
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();
    // Dangling marker in excluded test file — should NOT be reported
    write_source(
        tmp.path(),
        "src/app.test.ts",
        "// @specre 01ZZZZZZZZZZZZZZZZZZZZZZZZ\n",
    );
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    specre()
        .args(["orphans"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No orphans or dangling markers found.",
        ));
}
