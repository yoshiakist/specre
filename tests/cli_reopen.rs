// @specre 01KNK6TW0DJQ8TE4FF2ZBC6V67
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

fn write_specre_card(
    dir: &std::path::Path,
    rel_path: &str,
    id: &str,
    name: &str,
    status: &str,
    last_verified: Option<&str>,
) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let verified_line =
        last_verified.map_or_else(String::new, |date| format!("last_verified: \"{date}\"\n"));
    let content = format!(
        "---\nid: \"{id}\"\nname: \"{name}\"\nstatus: \"{status}\"\n{verified_line}---\n\n## Related Files\n\n\n\n## Functional Overview\n\nTest card.\n\n## Scenarios\n\n### Test\n\n1. Test step\n"
    );
    fs::write(path, content).unwrap();
}

fn read_frontmatter_field(dir: &std::path::Path, rel_path: &str, field: &str) -> Option<String> {
    let content = fs::read_to_string(dir.join(rel_path)).unwrap();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&format!("{field}:")) {
            let val = rest.trim().trim_matches('"');
            return Some(val.to_string());
        }
    }
    None
}

fn parse_json(stdout: &[u8]) -> serde_json::Value {
    serde_json::from_slice(stdout).expect("output should be valid JSON")
}

// -- Scenario: Reopen a stable specre by ULID --

#[test]
fn reopen_stable_specre() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
    );

    let output = specre()
        .args(["reopen", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "should exit 0");

    let status = read_frontmatter_field(tmp.path(), "docs/specres/cli/card_a.md", "status");
    assert_eq!(status.as_deref(), Some("in-development"));
}

// -- Scenario: last_verified is preserved after reopen --

#[test]
fn reopen_preserves_last_verified() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
    );

    specre()
        .args(["reopen", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let verified =
        read_frontmatter_field(tmp.path(), "docs/specres/cli/card_a.md", "last_verified");
    assert_eq!(
        verified.as_deref(),
        Some("2026-03-01"),
        "last_verified should be preserved"
    );
}

// -- Scenario: JSON output --

#[test]
fn reopen_json_output() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
    );

    let output = specre()
        .args(["reopen", "01AAAAAAAAAAAAAAAAAAAAAAAA", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let json = parse_json(&output.stdout);
    let reopened = &json["reopened"];
    assert_eq!(reopened["id"], "01AAAAAAAAAAAAAAAAAAAAAAAA");
    assert_eq!(reopened["name"], "card_a");
    assert_eq!(reopened["previous_status"], "stable");
    assert_eq!(reopened["new_status"], "in-development");
    assert_eq!(reopened["last_verified"], "2026-03-01");
    assert!(reopened["path"].as_str().unwrap().contains("card_a.md"));
}

// -- Scenario: Human-readable output --

#[test]
fn reopen_human_readable_output() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
    );

    let output = specre()
        .args(["reopen", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Reopened"), "should contain 'Reopened'");
    assert!(
        stdout.contains("01AAAAAAAAAAAAAAAAAAAAAAAA"),
        "should contain ULID"
    );
    assert!(stdout.contains("card_a"), "should contain name");
    assert!(stdout.contains("in-development"), "should show new status");
}

// -- Scenario: Reopen a non-stable specre returns an error --

#[test]
fn reopen_draft_specre_errors() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "draft",
        None,
    );

    specre()
        .args(["reopen", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("draft"))
        .stderr(predicate::str::contains("stable"));
}

#[test]
fn reopen_in_development_specre_errors() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "in-development",
        None,
    );

    specre()
        .args(["reopen", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("in-development"))
        .stderr(predicate::str::contains("stable"));
}

#[test]
fn reopen_deprecated_specre_errors() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "deprecated",
        None,
    );

    specre()
        .args(["reopen", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("deprecated"))
        .stderr(predicate::str::contains("stable"));
}

// -- Scenario: ULID not found --

#[test]
fn reopen_ulid_not_found() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();

    specre()
        .args(["reopen", "01ZZZZZZZZZZZZZZZZZZZZZZZZ"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("01ZZZZZZZZZZZZZZZZZZZZZZZZ"));
}

// -- Scenario: No specre.toml present --

#[test]
fn reopen_errors_without_config() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["reopen", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "specre.toml not found. Run 'specre init' first.",
        ));
}

// -- Scenario: Reopen preserves card body content --

#[test]
fn reopen_preserves_card_body() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_specre_card(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
    );

    let original_content =
        fs::read_to_string(tmp.path().join("docs/specres/cli/card_a.md")).unwrap();
    let body_start = original_content.find("\n---\n").unwrap() + 5;
    let original_body = &original_content[body_start..];

    specre()
        .args(["reopen", "01AAAAAAAAAAAAAAAAAAAAAAAA"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let updated_content =
        fs::read_to_string(tmp.path().join("docs/specres/cli/card_a.md")).unwrap();
    let updated_body_start = updated_content.find("\n---\n").unwrap() + 5;
    let updated_body = &updated_content[updated_body_start..];

    assert_eq!(original_body, updated_body, "card body should be preserved");
}
