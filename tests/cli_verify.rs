// @specre 01KNK4YMXKJ2B395E8NPQ4DV2Q
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

fn write_source(dir: &std::path::Path, rel_path: &str, content: &str) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn write_specre_card(
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

fn read_last_verified(dir: &std::path::Path, rel_path: &str) -> Option<String> {
    let content = fs::read_to_string(dir.join(rel_path)).unwrap();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("last_verified:") {
            let val = rest.trim().trim_matches('"');
            return Some(val.to_string());
        }
    }
    None
}

fn parse_json(stdout: &[u8]) -> serde_json::Value {
    serde_json::from_slice(stdout).expect("output should be valid JSON")
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

// -- Scenario: Verify single specre by ULID --

#[test]
fn verify_single_ulid() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &["src/a.rs"],
    );

    let output = specre()
        .args(["verify", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "should exit 0");

    let verified = read_last_verified(tmp.path(), "docs/specres/cli/card_a.md");
    assert_eq!(verified.as_deref(), Some(today().as_str()));
}

// -- Scenario: Verify multiple specres by ULID --

#[test]
fn verify_multiple_ulids() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &[],
    );
    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_b",
        "stable",
        Some("2026-03-01"),
        &[],
    );

    let output = specre()
        .args([
            "verify",
            "01AAAAAAAAAAAAAAAAAAAAAAAA",
            "01BBBBBBBBBBBBBBBBBBBBBBBB",
        ])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "should exit 0");

    let today = today();
    assert_eq!(
        read_last_verified(tmp.path(), "docs/specres/cli/card_a.md").as_deref(),
        Some(today.as_str())
    );
    assert_eq!(
        read_last_verified(tmp.path(), "docs/specres/cli/card_b.md").as_deref(),
        Some(today.as_str())
    );
}

// -- Scenario: Verify by domain --

#[test]
fn verify_by_domain() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &[],
    );
    write_specre_card(
        tmp.path(),
        "docs/specres/mcp/card_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_b",
        "stable",
        Some("2026-03-01"),
        &[],
    );

    let output = specre()
        .args(["verify", "--domain", "cli"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "should exit 0");

    let today = today();
    assert_eq!(
        read_last_verified(tmp.path(), "docs/specres/cli/card_a.md").as_deref(),
        Some(today.as_str())
    );
    // mcp domain card should NOT be updated
    assert_eq!(
        read_last_verified(tmp.path(), "docs/specres/mcp/card_b.md").as_deref(),
        Some("2026-03-01")
    );
}

// -- Scenario: Verify by file (aggregation file problem) --

#[test]
fn verify_by_file() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    // Source file with two @specre markers
    write_source(
        tmp.path(),
        "src/mod.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n// @specre 01BBBBBBBBBBBBBBBBBBBBBBBB\npub mod a;\n",
    );

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &["src/mod.rs"],
    );
    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_b",
        "stable",
        Some("2026-03-01"),
        &["src/mod.rs"],
    );
    // A third card NOT linked to the file
    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_c.md",
        "01CCCCCCCCCCCCCCCCCCCCCCCC",
        "card_c",
        "stable",
        Some("2026-03-01"),
        &["src/other.rs"],
    );

    let output = specre()
        .args(["verify", "--file", "src/mod.rs"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "should exit 0");

    let today = today();
    assert_eq!(
        read_last_verified(tmp.path(), "docs/specres/cli/card_a.md").as_deref(),
        Some(today.as_str())
    );
    assert_eq!(
        read_last_verified(tmp.path(), "docs/specres/cli/card_b.md").as_deref(),
        Some(today.as_str())
    );
    // card_c should NOT be updated
    assert_eq!(
        read_last_verified(tmp.path(), "docs/specres/cli/card_c.md").as_deref(),
        Some("2026-03-01")
    );
}

// -- Scenario: Verify by file using Related Files reference --

#[test]
fn verify_by_file_related_files_section() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    // Source file WITHOUT @specre marker
    write_source(tmp.path(), "src/foo.rs", "fn foo() {}\n");

    // Card references the file in Related Files
    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &["src/foo.rs"],
    );

    let output = specre()
        .args(["verify", "--file", "src/foo.rs"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "should exit 0");

    assert_eq!(
        read_last_verified(tmp.path(), "docs/specres/cli/card_a.md").as_deref(),
        Some(today().as_str())
    );
}

// -- Scenario: JSON output --

#[test]
fn verify_json_output() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &[],
    );

    let output = specre()
        .args(["verify", "01AAAAAAAAAAAAAAAAAAAAAAAA", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let json = parse_json(&output.stdout);
    assert!(json["verified"].is_array());
    assert_eq!(json["count"], 1);

    let verified = &json["verified"][0];
    assert_eq!(verified["id"], "01AAAAAAAAAAAAAAAAAAAAAAAA");
    assert_eq!(verified["name"], "card_a");
    assert_eq!(verified["last_verified"], today());
    assert!(verified["path"].as_str().unwrap().contains("card_a.md"));
}

// -- Scenario: Human-readable output --

#[test]
fn verify_human_readable_output() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &[],
    );

    let output = specre()
        .args(["verify", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Verified"), "should contain 'Verified'");
    assert!(
        stdout.contains("01AAAAAAAAAAAAAAAAAAAAAAAA"),
        "should contain ULID"
    );
    assert!(stdout.contains("card_a"), "should contain name");
}

// -- Scenario: No matching specres found --

#[test]
fn verify_ulid_not_found() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    // No specre cards exist
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();

    specre()
        .args(["verify", "01ZZZZZZZZZZZZZZZZZZZZZZZZ"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("01ZZZZZZZZZZZZZZZZZZZZZZZZ"));
}

// -- Scenario: No arguments provided --

#[test]
fn verify_no_arguments() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    specre()
        .args(["verify"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("specify"));
}

// -- Scenario: ULID not found among multiple --

#[test]
fn verify_partial_failure() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &[],
    );

    let output = specre()
        .args([
            "verify",
            "01AAAAAAAAAAAAAAAAAAAAAAAA",
            "01ZZZZZZZZZZZZZZZZZZZZZZZZ",
        ])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "should exit 1 for partial failure"
    );

    // But card_a should still be updated
    assert_eq!(
        read_last_verified(tmp.path(), "docs/specres/cli/card_a.md").as_deref(),
        Some(today().as_str())
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("01ZZZZZZZZZZZZZZZZZZZZZZZZ"));
}

// -- Scenario: Specre without last_verified field --

#[test]
fn verify_adds_last_verified_when_missing() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        None, // No last_verified
        &[],
    );

    let output = specre()
        .args(["verify", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "should exit 0");

    let verified = read_last_verified(tmp.path(), "docs/specres/cli/card_a.md");
    assert_eq!(verified.as_deref(), Some(today().as_str()));
}

// -- Scenario: No specre.toml present --

#[test]
fn verify_errors_without_config() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["verify", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "specre.toml not found. Run 'specre init' first.",
        ));
}

// -- Scenario: Mutual exclusivity of --domain and --file --

#[test]
fn verify_domain_and_file_conflict() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    specre()
        .args(["verify", "--domain", "cli", "--file", "src/a.rs"])
        .current_dir(tmp.path())
        .assert()
        .failure();
}

// -- Scenario: ULIDs and --domain conflict --

#[test]
fn verify_ulids_and_domain_conflict() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    specre()
        .args(["verify", "01AAAAAAAAAAAAAAAAAAAAAAAA", "--domain", "cli"])
        .current_dir(tmp.path())
        .assert()
        .failure();
}

// -- Scenario: ULIDs and --file conflict --

#[test]
fn verify_ulids_and_file_conflict() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    specre()
        .args(["verify", "01AAAAAAAAAAAAAAAAAAAAAAAA", "--file", "src/a.rs"])
        .current_dir(tmp.path())
        .assert()
        .failure();
}

// -- Scenario: Verify preserves card content beyond front-matter --

#[test]
fn verify_preserves_card_body() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &["src/a.rs"],
    );

    let original_content =
        fs::read_to_string(tmp.path().join("docs/specres/cli/card_a.md")).unwrap();
    let body_start = original_content.find("\n---\n").unwrap() + 5; // after closing ---
    let original_body = &original_content[body_start..];

    specre()
        .args(["verify", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let updated_content =
        fs::read_to_string(tmp.path().join("docs/specres/cli/card_a.md")).unwrap();
    let updated_body_start = updated_content.find("\n---\n").unwrap() + 5;
    let updated_body = &updated_content[updated_body_start..];

    assert_eq!(original_body, updated_body, "card body should be preserved");
}
