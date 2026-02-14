// @specre 01KHB48EES4FR5TFV6ZP2W3MGT
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
