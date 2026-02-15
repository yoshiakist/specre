// @specre 01KHG0A2V4YXE918WMJCY7WFE8
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;

fn specre() -> assert_cmd::Command {
    cargo_bin_cmd!("specre")
}

/// Helper: create specre.toml with given specre_dir and source_dirs
fn write_config(dir: &std::path::Path, specre_dir: &str) {
    let content = format!("specre_dir = \"{specre_dir}\"\nsource_dirs = [\"src\"]\n");
    fs::write(dir.join("specre.toml"), content).unwrap();
}

/// Helper: create a specre .md file with front-matter
fn write_specre(
    dir: &std::path::Path,
    rel_path: &str,
    id: &str,
    name: &str,
    status: &str,
    last_verified: Option<&str>,
) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut fm = format!("---\nid: \"{id}\"\nname: \"{name}\"\nstatus: \"{status}\"\n");
    if let Some(lv) = last_verified {
        fm.push_str(&format!("last_verified: \"{lv}\"\n"));
    }
    fm.push_str("---\n\n## Related Files\n\n-\n");
    fs::write(path, fm).unwrap();
}

/// Helper: create a source file with @specre marker
fn write_source(dir: &std::path::Path, rel_path: &str, ulid: &str) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let content = format!("// @specre {ulid}\nfn main() {{}}\n");
    fs::write(path, content).unwrap();
}

/// Helper: create a source file without @specre marker
fn write_source_no_marker(dir: &std::path::Path, rel_path: &str) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "fn main() {}\n").unwrap();
}

/// Helper: parse stdout as JSON Value
fn parse_json(output: &assert_cmd::assert::Assert) -> Value {
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    serde_json::from_str(&stdout).expect("stdout should be valid JSON")
}

// ============================================================
// status --json
// ============================================================

#[test]
fn status_json_outputs_structured_json() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "draft",
        None,
    );
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "spec_b",
        "in-development",
        None,
    );
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_c.md",
        "01CCCCCCCCCCCCCCCCCCCCCCCC",
        "spec_c",
        "stable",
        Some("2026-02-15"),
    );
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_d.md",
        "01DDDDDDDDDDDDDDDDDDDDDD",
        "spec_d",
        "deprecated",
        None,
    );

    let assert = specre()
        .args(["status", "--json"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json = parse_json(&assert);
    let summary = &json["summary"];
    assert_eq!(summary["draft"], 1);
    assert_eq!(summary["in_development"], 1);
    assert_eq!(summary["stable"], 1);
    assert_eq!(summary["deprecated"], 1);
    assert_eq!(summary["total"], 4);
    assert!(json["stale"].is_array());
}

#[test]
fn status_json_includes_stale_specres() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_specre(
        tmp.path(),
        "docs/specres/auth/stale_spec.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "stale_spec",
        "stable",
        Some("2025-01-01"),
    );

    let assert = specre()
        .args(["status", "--json"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json = parse_json(&assert);
    let stale = json["stale"].as_array().unwrap();
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0]["name"], "stale_spec");
    assert!(stale[0]["path"].as_str().unwrap().contains("stale_spec.md"));
}

#[test]
fn status_without_json_flag_outputs_text() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "draft",
        None,
    );

    specre()
        .args(["status"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status Summary:"));
}

// ============================================================
// trace --json (ULID lookup)
// ============================================================

#[test]
fn trace_json_ulid_lookup() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
        Some("2026-02-15"),
    );
    write_source(tmp.path(), "src/main.rs", "01AAAAAAAAAAAAAAAAAAAAAAAA");

    let assert = specre()
        .args(["trace", "--json", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json = parse_json(&assert);
    assert!(json["specre"].as_str().unwrap().contains("spec_a.md"));
    let refs = json["source_refs"].as_array().unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0]["file"], "src/main.rs");
    assert_eq!(refs[0]["line"], 1);
}

#[test]
fn trace_json_file_lookup() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
        Some("2026-02-15"),
    );
    write_source(tmp.path(), "src/main.rs", "01AAAAAAAAAAAAAAAAAAAAAAAA");

    let assert = specre()
        .args(["trace", "--json", "src/main.rs"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json = parse_json(&assert);
    assert_eq!(json["file"], "src/main.rs");
    let specres = json["specres"].as_array().unwrap();
    assert_eq!(specres.len(), 1);
    assert_eq!(specres[0]["id"], "01AAAAAAAAAAAAAAAAAAAAAAAA");
    assert!(specres[0]["path"].as_str().unwrap().contains("spec_a.md"));
}

#[test]
fn trace_json_unknown_ulid() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    let assert = specre()
        .args(["trace", "--json", "01ZZZZZZZZZZZZZZZZZZZZZZZZ"])
        .current_dir(tmp.path())
        .assert()
        .failure();

    let json = parse_json(&assert);
    assert!(json["specre"].is_null());
    assert!(json["source_refs"].as_array().unwrap().is_empty());
}

// ============================================================
// orphans --json
// ============================================================

#[test]
fn orphans_json_with_orphans() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    // Orphan specre: no source marker
    write_specre(
        tmp.path(),
        "docs/specres/cli/orphan_spec.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "orphan_spec",
        "stable",
        Some("2026-02-15"),
    );
    // Dangling marker: marker with no matching specre
    write_source(tmp.path(), "src/main.rs", "01ZZZZZZZZZZZZZZZZZZZZZZZZ");

    let assert = specre()
        .args(["orphans", "--json"])
        .current_dir(tmp.path())
        .assert()
        .failure();

    let json = parse_json(&assert);
    let orphan_specres = json["orphan_specres"].as_array().unwrap();
    assert_eq!(orphan_specres.len(), 1);
    assert!(orphan_specres[0].as_str().unwrap().contains("orphan_spec.md"));

    let dangling = json["dangling_markers"].as_array().unwrap();
    assert_eq!(dangling.len(), 1);
    assert_eq!(dangling[0]["file"], "src/main.rs");
    assert_eq!(dangling[0]["id"], "01ZZZZZZZZZZZZZZZZZZZZZZZZ");
}

#[test]
fn orphans_json_no_orphans() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
        Some("2026-02-15"),
    );
    write_source(tmp.path(), "src/main.rs", "01AAAAAAAAAAAAAAAAAAAAAAAA");

    let assert = specre()
        .args(["orphans", "--json"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json = parse_json(&assert);
    assert!(json["orphan_specres"].as_array().unwrap().is_empty());
    assert!(json["dangling_markers"].as_array().unwrap().is_empty());
}

// ============================================================
// coverage --json
// ============================================================

#[test]
fn coverage_json_outputs_structured_json() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_source(tmp.path(), "src/tagged.rs", "01AAAAAAAAAAAAAAAAAAAAAAAA");
    write_source_no_marker(tmp.path(), "src/untagged.rs");

    let assert = specre()
        .args(["coverage", "--json"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json = parse_json(&assert);
    assert_eq!(json["total"], 2);
    assert_eq!(json["tagged"], 1);
    assert_eq!(json["coverage"], 0.5);
    let uncovered = json["uncovered"].as_array().unwrap();
    assert_eq!(uncovered.len(), 1);
    assert_eq!(uncovered[0], "src/untagged.rs");
}

#[test]
fn coverage_json_with_ext_filter() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_source(tmp.path(), "src/tagged.rs", "01AAAAAAAAAAAAAAAAAAAAAAAA");
    write_source_no_marker(tmp.path(), "src/other.ts");

    let assert = specre()
        .args(["coverage", "--json", "--ext", "rs"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json = parse_json(&assert);
    assert_eq!(json["total"], 1);
    assert_eq!(json["tagged"], 1);
}

// ============================================================
// init --json
// ============================================================

#[test]
fn init_json_outputs_structured_json() {
    let tmp = TempDir::new().unwrap();

    let assert = specre()
        .args(["init", "--json"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json = parse_json(&assert);
    assert_eq!(json["specre_dir"], "docs/specres");
    assert_eq!(json["config_file"], "specre.toml");

    // Verify files were actually created
    assert!(tmp.path().join("specre.toml").exists());
    assert!(tmp.path().join("docs/specres").exists());
}

// ============================================================
// new --json
// ============================================================

#[test]
fn new_json_outputs_structured_json() {
    let tmp = TempDir::new().unwrap();
    // new doesn't require specre.toml, but load_language() tries to read it
    fs::create_dir_all(tmp.path().join("docs/specres/cli")).unwrap();

    let assert = specre()
        .args(["new", "docs/specres/cli", "--name", "test_spec", "--json"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json = parse_json(&assert);
    assert!(json["id"].is_string());
    assert_eq!(json["id"].as_str().unwrap().len(), 26); // ULID length
    let path = json["path"].as_str().unwrap();
    assert!(path.contains("test_spec.md"));
}

// ============================================================
// tag --json
// ============================================================

#[test]
fn tag_json_outputs_structured_json() {
    let tmp = TempDir::new().unwrap();
    write_source_no_marker(tmp.path(), "src/main.rs");

    let assert = specre()
        .args([
            "tag",
            "--json",
            "01AAAAAAAAAAAAAAAAAAAAAAAA",
            "src/main.rs",
        ])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json = parse_json(&assert);
    assert_eq!(json["id"], "01AAAAAAAAAAAAAAAAAAAAAAAA");
    assert_eq!(json["file"], "src/main.rs");
    assert_eq!(json["line"], 1);
}

// ============================================================
// index --json
// ============================================================

#[test]
fn index_json_outputs_structured_json() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
        Some("2026-02-15"),
    );
    write_source(tmp.path(), "src/main.rs", "01AAAAAAAAAAAAAAAAAAAAAAAA");

    let assert = specre()
        .args(["index", "--json"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json = parse_json(&assert);
    assert_eq!(json["index_file"], "docs/specres/index.json");
    assert_eq!(json["specre_count"], 1);
    assert_eq!(json["source_ref_count"], 1);
    let md_files = json["index_md_files"].as_array().unwrap();
    assert_eq!(md_files.len(), 1);
    assert!(md_files[0].as_str().unwrap().contains("_INDEX.md"));
}

// ============================================================
// search --json (already JSON, flag accepted)
// ============================================================

#[test]
fn search_json_flag_accepted() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();

    let assert = specre()
        .args(["search", "--json"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json = parse_json(&assert);
    assert!(json["results"].is_array());
    assert!(json["total"].is_number());
}

// ============================================================
// health-check --json (already JSON, flag accepted)
// ============================================================

#[test]
fn health_check_json_flag_accepted() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    // health-check will likely fail (no index.json) but we just want to confirm
    // the --json flag is accepted without error
    specre()
        .args(["health-check", "--json"])
        .current_dir(tmp.path())
        .assert()
        .stderr(predicate::str::contains("unrecognized").not());
}

// ============================================================
// --json flag placement (global flag)
// ============================================================

#[test]
fn json_flag_before_subcommand() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();

    let assert = specre()
        .args(["--json", "status"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json = parse_json(&assert);
    assert!(json["summary"].is_object());
}

// ============================================================
// Error output goes to stderr regardless of --json
// ============================================================

#[test]
fn json_error_goes_to_stderr_as_plain_text() {
    let tmp = TempDir::new().unwrap();
    // No specre.toml

    specre()
        .args(["status", "--json"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "specre.toml not found. Run 'specre init' first.",
        ));
}
