// @specre 01KJ4NJW28F072X64SS68P3N08
// @specre 01KHFGVXWP100JXYBZTRJGMB9H
mod common;
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::fs;
use std::process::Command;

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

fn write_config_with_health_check(
    dir: &std::path::Path,
    specre_dir: &str,
    source_dirs: &[&str],
    coverage: f64,
    orphans: usize,
    index_age_hours: f64,
) {
    let dirs_toml: Vec<String> = source_dirs.iter().map(|s| format!("\"{s}\"")).collect();
    let content = format!(
        "specre_dir = \"{specre_dir}\"\nsource_dirs = [{}]\n\n[health_check]\ncoverage = {coverage}\norphans = {orphans}\nindex_age_hours = {index_age_hours}\n",
        dirs_toml.join(", ")
    );
    fs::write(dir.join("specre.toml"), content).unwrap();
}

fn write_config_with_drifts_threshold(
    dir: &std::path::Path,
    specre_dir: &str,
    source_dirs: &[&str],
    drifts: usize,
) {
    let dirs_toml: Vec<String> = source_dirs.iter().map(|s| format!("\"{s}\"")).collect();
    let content = format!(
        "specre_dir = \"{specre_dir}\"\nsource_dirs = [{}]\n\n[health_check]\ncoverage = 0.0\norphans = 100\nindex_age_hours = 1000.0\ndrifts = {drifts}\n",
        dirs_toml.join(", ")
    );
    fs::write(dir.join("specre.toml"), content).unwrap();
}

fn write_config_with_exclude_and_health_check(
    dir: &std::path::Path,
    specre_dir: &str,
    source_dirs: &[&str],
    exclude_patterns: &[&str],
    coverage: f64,
    orphans: usize,
    index_age_hours: f64,
) {
    let dirs_toml: Vec<String> = source_dirs.iter().map(|s| format!("\"{s}\"")).collect();
    let pats_toml: Vec<String> = exclude_patterns
        .iter()
        .map(|s| format!("\"{s}\""))
        .collect();
    let content = format!(
        "specre_dir = \"{specre_dir}\"\nsource_dirs = [{}]\nexclude_patterns = [{}]\n\n[health_check]\ncoverage = {coverage}\norphans = {orphans}\nindex_age_hours = {index_age_hours}\n",
        dirs_toml.join(", "),
        pats_toml.join(", ")
    );
    fs::write(dir.join("specre.toml"), content).unwrap();
}

fn write_source(dir: &std::path::Path, rel_path: &str, content: &str) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn write_specre_card(dir: &std::path::Path, rel_path: &str, id: &str, name: &str, status: &str) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let content = format!(
        "---\nid: \"{id}\"\nname: \"{name}\"\nstatus: \"{status}\"\n---\n\n## Related Files\n\n## Functional Overview\n\n## Scenarios\n"
    );
    fs::write(path, content).unwrap();
}

fn write_index_json(dir: &std::path::Path, generated_at: &str) {
    let content =
        format!(r#"{{"version":1,"generated_at":"{generated_at}","specres":[],"source_refs":[]}}"#);
    let path = dir.join("docs/specres/index.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn recent_timestamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn old_timestamp(hours_ago: i64) -> String {
    (chrono::Utc::now() - chrono::Duration::hours(hours_ago))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn parse_json(stdout: &[u8]) -> serde_json::Value {
    serde_json::from_slice(stdout).expect("output should be valid JSON")
}

// -- Scenario: Healthy ecosystem — all metrics within thresholds --

#[test]
fn health_check_healthy() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    // 2 source files, both tagged -> coverage 1.0
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

    // Matching specre cards (not orphans)
    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );
    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_b",
        "stable",
    );

    // Recent index.json
    write_index_json(tmp.path(), &recent_timestamp());

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "should exit 0 when healthy");

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], true);
    assert_eq!(json["coverage"], 1.0);
    assert_eq!(json["orphans"], 0);
    assert!(json["index_age_hours"].as_f64().unwrap() < 1.0);
    assert!(
        json["drifts"].is_null(),
        "drifts should be null without git"
    );
    assert_eq!(json["thresholds"]["coverage"], 0.9);
    assert_eq!(json["thresholds"]["orphans"], 5);
    assert_eq!(json["thresholds"]["drifts"], 0);
    assert_eq!(json["thresholds"]["index_age_hours"], 24.0);
}

// -- Scenario: Unhealthy ecosystem — coverage below threshold --

#[test]
fn health_check_unhealthy_low_coverage() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    // 2 source files, 1 tagged -> coverage 0.5
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );
    write_source(tmp.path(), "src/b.rs", "fn b() {}\n");

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    write_index_json(tmp.path(), &recent_timestamp());

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(!output.status.success(), "should exit 1 when unhealthy");

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], false);
    assert_eq!(json["coverage"], 0.5);
}

// -- Scenario: Unhealthy ecosystem — orphans above threshold --

#[test]
fn health_check_unhealthy_many_orphans() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    // 1 source file tagged -> coverage 1.0
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    // 6 orphan specre cards (no matching source markers)
    for i in 1..=6 {
        // ULIDs are 26 uppercase alphanumeric chars
        let id = format!("01ORPHAN{:0>18}", format!("{i}AAAAAAAAAAAAAAA"));
        write_specre_card(
            tmp.path(),
            &format!("docs/specres/cli/orphan_{i}.md"),
            &id[..26],
            &format!("orphan_{i}"),
            "draft",
        );
    }

    write_index_json(tmp.path(), &recent_timestamp());

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "should exit 1 when too many orphans"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], false);
    assert!(json["orphans"].as_u64().unwrap() > 5);
}

// -- Scenario: Unhealthy ecosystem — index.json missing --

#[test]
fn health_check_unhealthy_no_index() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    // No index.json

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "should exit 1 when index.json missing"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], false);
    assert!(json["index_age_hours"].is_null());
}

// -- Scenario: Unhealthy ecosystem — index.json stale --

#[test]
fn health_check_unhealthy_stale_index() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    // index.json generated 48 hours ago
    write_index_json(tmp.path(), &old_timestamp(48));

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(!output.status.success(), "should exit 1 when index stale");

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], false);
    assert!(json["index_age_hours"].as_f64().unwrap() > 24.0);
}

// -- Scenario: Stale timestamp but content identical — treated as healthy --

#[test]
fn health_check_stale_index_content_identical() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    // 1 source file, tagged -> coverage 1.0
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    // Matching specre card
    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    // Generate real index.json with correct content
    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // Make the timestamp old (48 hours ago) but keep content identical
    let index_path = tmp.path().join("docs/specres/index.json");
    let content = fs::read_to_string(&index_path).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&content).unwrap();
    json["generated_at"] = serde_json::Value::String(old_timestamp(48));
    fs::write(&index_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "should be healthy when index content matches despite old timestamp"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], true);
    assert!(json["index_age_hours"].as_f64().unwrap() > 24.0);
}

// -- Scenario: Unhealthy ecosystem — index.json stale with content drift --

#[test]
fn health_check_stale_index_content_drift() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    // 1 source file, tagged
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    // Generate real index.json
    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // Make the timestamp old
    let index_path = tmp.path().join("docs/specres/index.json");
    let content = fs::read_to_string(&index_path).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&content).unwrap();
    json["generated_at"] = serde_json::Value::String(old_timestamp(48));
    fs::write(&index_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

    // Now add a new source file and specre card (content drift)
    write_source(
        tmp.path(),
        "src/b.rs",
        "// @specre 01BBBBBBBBBBBBBBBBBBBBBBBB\nfn b() {}\n",
    );

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_b",
        "stable",
    );

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "should be unhealthy when index content has drifted"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], false);
    assert!(json["index_age_hours"].as_f64().unwrap() > 24.0);
}

// -- Scenario: Custom thresholds via specre.toml --

#[test]
fn health_check_custom_thresholds() {
    let tmp = TempDir::new().unwrap();
    // Set permissive thresholds: coverage 0.30, orphans 20, index_age_hours 100
    write_config_with_health_check(tmp.path(), "docs/specres", &["src"], 0.30, 20, 100.0);

    // 2 source files, 1 tagged -> coverage 0.5 (above custom 0.30 threshold)
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );
    write_source(tmp.path(), "src/b.rs", "fn b() {}\n");

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    // index.json 50 hours old (within custom 100h threshold)
    write_index_json(tmp.path(), &old_timestamp(50));

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "should be healthy with permissive thresholds"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], true);
    assert_eq!(json["thresholds"]["coverage"], 0.3);
    assert_eq!(json["thresholds"]["orphans"], 20);
    assert_eq!(json["thresholds"]["drifts"], 0);
    assert_eq!(json["thresholds"]["index_age_hours"], 100.0);
}

// -- Scenario: No source files — coverage is 0.0 --

#[test]
fn health_check_no_source_files() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();

    write_index_json(tmp.path(), &recent_timestamp());

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(!output.status.success(), "should exit 1 with 0 coverage");

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], false);
    assert_eq!(json["coverage"], 0.0);
}

// -- Scenario: Multiple metrics failing simultaneously --

#[test]
fn health_check_multiple_failures() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    // Low coverage: 0 of 1
    write_source(tmp.path(), "src/a.rs", "fn a() {}\n");

    // No index.json -> null index_age_hours

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(!output.status.success());

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], false);
    assert_eq!(json["coverage"], 0.0);
    assert!(json["index_age_hours"].is_null());
}

// -- Scenario: specre.toml does not exist --

#[test]
fn health_check_errors_without_config() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "specre.toml not found. Run 'specre init' first.",
        ));
}

// -- Scenario: JSON output is machine-parseable --

#[test]
fn health_check_json_output_is_valid() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    write_index_json(tmp.path(), &recent_timestamp());

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let json = parse_json(&output.stdout);

    // Verify field types
    assert!(json["healthy"].is_boolean());
    assert!(json["coverage"].is_f64());
    assert!(json["orphans"].is_u64());
    // drifts is null (no git) or u64
    assert!(
        json["drifts"].is_null() || json["drifts"].is_u64(),
        "drifts should be null or integer"
    );
    assert!(
        json["index_age_hours"].is_f64(),
        "index_age_hours should be a float"
    );
    assert!(json["thresholds"].is_object());
    assert!(json["thresholds"]["coverage"].is_f64());
    assert!(json["thresholds"]["orphans"].is_u64());
    assert!(json["thresholds"]["drifts"].is_u64());
    assert!(json["thresholds"]["index_age_hours"].is_f64());
}

// -- Scenario: index.json malformed --

#[test]
fn health_check_malformed_index_json() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    // Malformed index.json
    fs::write(tmp.path().join("docs/specres/index.json"), "not valid json").unwrap();

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(!output.status.success());

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], false);
    assert!(json["index_age_hours"].is_null());

    // Should warn about malformed JSON
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning:"),
        "Expected warning about malformed index.json in stderr, got: {stderr}"
    );
}

// -- Failures / Exceptions: IO error on index.json warns --

#[cfg(unix)]
#[test]
fn health_check_warns_on_unreadable_index_json() {
    use std::os::unix::fs::PermissionsExt;
    if common::is_root() {
        return; // root bypasses file-permission checks
    }

    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );
    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    // Create index.json but make it unreadable
    write_index_json(tmp.path(), &recent_timestamp());
    let index_path = tmp.path().join("docs/specres/index.json");
    write_index_json(tmp.path(), &recent_timestamp());
    fs::set_permissions(&index_path, fs::Permissions::from_mode(0o000)).unwrap();

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning:"),
        "Expected warning about unreadable index.json in stderr, got: {stderr}"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], false);
    assert!(json["index_age_hours"].is_null());

    fs::set_permissions(&index_path, fs::Permissions::from_mode(0o644)).unwrap();
}

// -- Scenario: exclude_patterns makes coverage healthy --

#[test]
fn health_check_exclude_patterns_improves_coverage() {
    let tmp = TempDir::new().unwrap();
    // Without exclude, coverage would be 1/3 = 33% (below 0.90 threshold)
    // With exclude, coverage becomes 1/1 = 100% (above threshold)
    write_config_with_exclude_and_health_check(
        tmp.path(),
        "docs/specres",
        &["src"],
        &["*.test.ts", "*/_generated/*"],
        0.90,
        5,
        24.0,
    );

    write_source(
        tmp.path(),
        "src/main.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n",
    );
    // These should be excluded
    write_source(tmp.path(), "src/app.test.ts", "// no marker\n");
    write_source(tmp.path(), "src/_generated/types.rs", "type T = i32;\n");

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    write_index_json(tmp.path(), &recent_timestamp());

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "should be healthy after excluding test/generated files"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], true);
    assert_eq!(json["coverage"], 1.0);
}

// ---------------------------------------------------------------------------
// Git helpers for drift-related health-check tests
// ---------------------------------------------------------------------------

fn write_specre_card_with_verified(
    dir: &std::path::Path,
    rel_path: &str,
    id: &str,
    name: &str,
    status: &str,
    last_verified: Option<&str>,
    related_files: &[&str],
) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let verified_line =
        last_verified.map_or_else(String::new, |date| format!("last_verified: \"{date}\"\n"));
    let related = if related_files.is_empty() {
        String::new()
    } else {
        related_files
            .iter()
            .map(|f| format!("- `{f}`"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let content = format!(
        "---\nid: \"{id}\"\nname: \"{name}\"\nstatus: \"{status}\"\n{verified_line}---\n\n## Related Files\n\n{related}\n\n## Functional Overview\n\nTest card.\n\n## Scenarios\n\n### Test\n\n1. Test step\n"
    );
    fs::write(path, content).unwrap();
}

fn git_init_and_commit_with_date(dir: &std::path::Path, date: &str) {
    Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["add", "-A"])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .env("GIT_AUTHOR_DATE", date)
        .env("GIT_COMMITTER_DATE", date)
        .current_dir(dir)
        .output()
        .unwrap();
}

fn git_commit_file_with_date(dir: &std::path::Path, file: &str, content: &str, date: &str) {
    fs::write(dir.join(file), content).unwrap();
    Command::new("git")
        .args(["add", file])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", &format!("update {file}")])
        .env("GIT_AUTHOR_DATE", date)
        .env("GIT_COMMITTER_DATE", date)
        .current_dir(dir)
        .output()
        .unwrap();
}

// -- Scenario: Git not available — drift check skipped --

#[test]
fn health_check_no_git_drifts_null() {
    let tmp = TempDir::new().unwrap();
    // Permissive thresholds so only drift matters
    write_config_with_drifts_threshold(tmp.path(), "docs/specres", &["src"], 0);

    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );
    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );
    write_index_json(tmp.path(), &recent_timestamp());

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "should be healthy when git is unavailable (drifts excluded from verdict)"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], true);
    assert!(
        json["drifts"].is_null(),
        "drifts should be null without git"
    );
    assert_eq!(json["thresholds"]["drifts"], 0);
}

// -- Scenario: Unhealthy ecosystem — drifts above threshold --

#[test]
fn health_check_unhealthy_drifts_above_threshold() {
    let tmp = TempDir::new().unwrap();
    // Permissive thresholds for everything except drifts (default 0)
    write_config_with_drifts_threshold(tmp.path(), "docs/specres", &["src"], 0);

    // Source file with marker, committed at old date
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );
    // Specre card verified at old date, with related file
    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-01-01"),
        &["src/a.rs"],
    );
    write_index_json(tmp.path(), &recent_timestamp());

    // Initialize git and commit at old date
    git_init_and_commit_with_date(tmp.path(), "2026-01-01T00:00:00+00:00");

    // Modify the source file and commit at a later date (causes drift)
    git_commit_file_with_date(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() { /* changed */ }\n",
        "2026-03-15T00:00:00+00:00",
    );

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "should exit 1 when drifts exceed threshold"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], false);
    assert_eq!(json["drifts"], 1);
}

// -- Scenario: Healthy with drifts within custom threshold --

#[test]
fn health_check_healthy_drifts_within_threshold() {
    let tmp = TempDir::new().unwrap();
    // Allow up to 5 drifts
    write_config_with_drifts_threshold(tmp.path(), "docs/specres", &["src"], 5);

    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );
    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-01-01"),
        &["src/a.rs"],
    );
    write_index_json(tmp.path(), &recent_timestamp());

    git_init_and_commit_with_date(tmp.path(), "2026-01-01T00:00:00+00:00");
    git_commit_file_with_date(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() { /* changed */ }\n",
        "2026-03-15T00:00:00+00:00",
    );

    let output = specre()
        .args(["health-check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "should be healthy when drifts (1) <= threshold (5)"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["healthy"], true);
    assert_eq!(json["drifts"], 1);
    assert_eq!(json["thresholds"]["drifts"], 5);
}
