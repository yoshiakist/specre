// @specre 01KHFEA9QVV4A127VCRJY97A68
mod common;
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

fn write_source(dir: &std::path::Path, rel_path: &str, content: &str) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

// -- Scenario: Basic coverage report --

#[test]
fn coverage_basic_report() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );
    write_source(
        tmp.path(),
        "src/b.rs",
        "// @specre 01BBBBBBBBBBBBBBBBBBBBBBBB\nfn b() {}\n",
    );
    write_source(
        tmp.path(),
        "src/c.rs",
        "// @specre 01CCCCCCCCCCCCCCCCCCCCCCCC\nfn c() {}\n",
    );
    write_source(tmp.path(), "src/d.rs", "fn d() {}\n");

    specre()
        .args(["coverage"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Coverage: 3/4 files (75.0%)"));
}

// -- Scenario: Full coverage --

#[test]
fn coverage_full() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );
    write_source(
        tmp.path(),
        "src/b.rs",
        "// @specre 01BBBBBBBBBBBBBBBBBBBBBBBB\nfn b() {}\n",
    );

    specre()
        .args(["coverage"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Coverage: 2/2 files (100.0%)"));
}

// -- Scenario: No source files found --

#[test]
fn coverage_no_source_files() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    specre()
        .args(["coverage"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Coverage: 0/0 files (N/A)"));
}

// -- Scenario: Zero coverage --

#[test]
fn coverage_zero() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_source(tmp.path(), "src/a.rs", "fn a() {}\n");
    write_source(tmp.path(), "src/b.rs", "fn b() {}\n");
    write_source(tmp.path(), "src/c.rs", "fn c() {}\n");

    specre()
        .args(["coverage"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Coverage: 0/3 files (0.0%)"));
}

// -- Scenario: Extension filtering via --ext flag --

#[test]
fn coverage_ext_flag_filters() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );
    write_source(tmp.path(), "src/b.ts", "// no marker\n");

    // Only count .rs files
    specre()
        .args(["coverage", "--ext", "rs"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Coverage: 1/1 files (100.0%)"));
}

// -- Scenario: Extension filtering via specre.toml target_extensions --

#[test]
fn coverage_target_extensions_from_config() {
    let tmp = TempDir::new().unwrap();
    write_config_with_ext(tmp.path(), "docs/specres", &["src"], &["rs"]);
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );
    write_source(tmp.path(), "src/b.ts", "// no marker\n");

    specre()
        .args(["coverage"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Coverage: 1/1 files (100.0%)"));
}

// -- Scenario: --ext flag overrides specre.toml target_extensions --

#[test]
fn coverage_ext_flag_overrides_config() {
    let tmp = TempDir::new().unwrap();
    write_config_with_ext(tmp.path(), "docs/specres", &["src"], &["rs"]);
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );
    write_source(tmp.path(), "src/b.ts", "// no marker\n");

    // --ext ts overrides target_extensions=["rs"] from config
    specre()
        .args(["coverage", "--ext", "ts"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Coverage: 0/1 files (0.0%)"));
}

// -- Scenario: Uncovered files are listed --

#[test]
fn coverage_lists_uncovered_files() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_source(
        tmp.path(),
        "src/foo.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn foo() {}\n",
    );
    write_source(tmp.path(), "src/bar.rs", "fn bar() {}\n");
    write_source(tmp.path(), "src/baz.rs", "fn baz() {}\n");

    specre()
        .args(["coverage"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Coverage: 1/3 files (33.3%)")
                .and(predicate::str::contains("Uncovered files:"))
                .and(predicate::str::contains("src/bar.rs"))
                .and(predicate::str::contains("src/baz.rs")),
        );
}

// -- Scenario: Uncovered files are sorted alphabetically --

#[test]
fn coverage_uncovered_files_sorted() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_source(tmp.path(), "src/zebra.rs", "fn z() {}\n");
    write_source(tmp.path(), "src/alpha.rs", "fn a() {}\n");

    let output = specre()
        .args(["coverage"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let alpha_pos = stdout.find("src/alpha.rs").unwrap();
    let zebra_pos = stdout.find("src/zebra.rs").unwrap();
    assert!(
        alpha_pos < zebra_pos,
        "Uncovered files should be sorted alphabetically"
    );
}

// -- Scenario: Paths use forward slashes --

#[test]
fn coverage_paths_use_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_source(tmp.path(), "src/sub/deep.rs", "fn deep() {}\n");

    let output = specre()
        .args(["coverage"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("src/sub/deep.rs"),
        "Paths should use forward slashes"
    );
    assert!(
        !stdout.contains('\\'),
        "Paths should not contain backslashes"
    );
}

// -- Scenario: specre.toml does not exist --

#[test]
fn coverage_errors_without_config() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["coverage"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "specre.toml not found. Run 'specre init' first.",
        ));
}

// -- Scenario: source_dirs directory does not exist --

#[test]
fn coverage_skips_nonexistent_source_dirs() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src", "nonexistent"]);
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    specre()
        .args(["coverage"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Coverage: 1/1 files (100.0%)"));
}

// -- Full coverage does not show uncovered section --

#[test]
fn coverage_full_no_uncovered_section() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    specre()
        .args(["coverage"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Coverage: 1/1 files (100.0%)")
                .and(predicate::str::contains("Uncovered files:").not()),
        );
}

// -- Failures / Exceptions: IO error warns and counts as uncovered --

#[cfg(unix)]
#[test]
fn coverage_warns_on_unreadable_source_file() {
    use std::os::unix::fs::PermissionsExt;
    if common::is_root() {
        return; // root bypasses file-permission checks
    }

    let tmp = TempDir::new().unwrap();
    write_config_with_ext(tmp.path(), "docs/specres", &["src"], &["rs"]);

    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("good.rs"),
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n",
    )
    .unwrap();

    let bad_file = src_dir.join("bad.rs");
    fs::write(&bad_file, "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n").unwrap();
    fs::set_permissions(&bad_file, fs::Permissions::from_mode(0o000)).unwrap();

    let output = specre()
        .args(["coverage"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning: failed to read"),
        "Expected warning about unreadable file in stderr, got: {stderr}"
    );

    // The unreadable file should be counted as uncovered
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Coverage: 1/2 files"));

    fs::set_permissions(&bad_file, fs::Permissions::from_mode(0o644)).unwrap();
}
