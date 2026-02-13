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

fn write_specre(dir: &std::path::Path, rel_path: &str, id: &str, name: &str, status: &str) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let fm = format!("---\nid: \"{id}\"\nname: \"{name}\"\nstatus: \"{status}\"\n---\n\n## Related Files\n\n-\n");
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

// -- Scenario: Invalid ULID format --

#[test]
fn trace_invalid_ulid_too_short() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    specre()
        .args(["trace", "abc123"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid ULID format. Expected 26 uppercase alphanumeric characters.",
        ));
}

#[test]
fn trace_invalid_ulid_lowercase() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    specre()
        .args(["trace", "01aaaaaaaaaaaaaaaaaaaaaaaa"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid ULID format. Expected 26 uppercase alphanumeric characters.",
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
        .stdout(
            predicate::str::contains("src/a.rs:2")
                .and(predicate::str::contains("src/b.rs:3")),
        );
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
