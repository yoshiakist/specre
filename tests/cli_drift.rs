// @specre 01KNJYEGK8KFVQK7EQGK9ZCZJR
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

fn write_config_with_drift(
    dir: &std::path::Path,
    specre_dir: &str,
    source_dirs: &[&str],
    grace_days: u64,
) {
    let dirs_toml: Vec<String> = source_dirs.iter().map(|s| format!("\"{s}\"")).collect();
    let content = format!(
        "specre_dir = \"{specre_dir}\"\nsource_dirs = [{}]\n\n[drift]\ngrace_days = {grace_days}\n",
        dirs_toml.join(", ")
    );
    fs::write(dir.join("specre.toml"), content).unwrap();
}

fn write_source(dir: &std::path::Path, rel_path: &str, content: &str) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

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

/// Initialize a git repo, add all files, and commit with a backdated date.
fn git_init_and_commit(dir: &std::path::Path) {
    git_init_and_commit_with_date(dir, "2026-01-01T00:00:00+00:00");
}

/// Initialize a git repo, add all files, and commit with a specific date.
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

/// Modify a file and commit with a backdated author date.
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

fn parse_json(stdout: &[u8]) -> serde_json::Value {
    serde_json::from_slice(stdout).expect("output should be valid JSON")
}

// -- Scenario: Project-wide drift check with no drift detected --

#[test]
fn drift_no_drift_all_clean() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    // Source file with marker, committed at 2026-03-01
    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    // Specre card verified after the source was last modified
    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-04-01"),
        &["src/a.rs"],
    );

    git_init_and_commit(tmp.path());

    let output = specre()
        .args(["drift", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "should exit 0 when no drift");

    let json = parse_json(&output.stdout);
    assert_eq!(json["drifted"].as_array().unwrap().len(), 0);
    assert_eq!(json["total"], json["clean"]);
    assert!(json["total"].as_u64().unwrap() > 0);
}

// -- Scenario: Project-wide drift check with drift detected --

#[test]
fn drift_detected_source_modified_after_verified() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    // last_verified is 2026-03-01
    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &["src/a.rs"],
    );

    git_init_and_commit(tmp.path());

    // Modify source file after last_verified
    git_commit_file_with_date(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() { /* changed */ }\n",
        "2026-04-05T12:00:00+00:00",
    );

    let output = specre()
        .args(["drift", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(!output.status.success(), "should exit 1 when drift found");

    let json = parse_json(&output.stdout);
    let drifted = json["drifted"].as_array().unwrap();
    assert_eq!(drifted.len(), 1);
    assert_eq!(drifted[0]["id"], "01AAAAAAAAAAAAAAAAAAAAAAAA");
    assert_eq!(drifted[0]["last_verified"], "2026-03-01");

    let changed = drifted[0]["changed_files"].as_array().unwrap();
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0]["file"], "src/a.rs");
    assert_eq!(changed[0]["last_modified"], "2026-04-05");
    assert!(changed[0]["diff_stat"].as_str().unwrap().contains('+'));
}

// -- Scenario: Single specre check by ULID --

#[test]
fn drift_single_ulid_check() {
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

    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &["src/a.rs"],
    );
    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/cli/card_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_b",
        "stable",
        Some("2026-03-01"),
        &["src/b.rs"],
    );

    git_init_and_commit(tmp.path());

    // Modify both files after last_verified
    git_commit_file_with_date(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() { /* changed */ }\n",
        "2026-04-05T12:00:00+00:00",
    );
    git_commit_file_with_date(
        tmp.path(),
        "src/b.rs",
        "// @specre 01BBBBBBBBBBBBBBBBBBBBBBBB\nfn b() { /* changed */ }\n",
        "2026-04-05T12:00:00+00:00",
    );

    // Only check card_a by ULID
    let output = specre()
        .args(["drift", "01AAAAAAAAAAAAAAAAAAAAAAAA", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let json = parse_json(&output.stdout);
    // total should be 1 (only the targeted specre)
    assert_eq!(json["total"], 1);
    let drifted = json["drifted"].as_array().unwrap();
    assert_eq!(drifted.len(), 1);
    assert_eq!(drifted[0]["id"], "01AAAAAAAAAAAAAAAAAAAAAAAA");
}

// -- Scenario: Path-based filtering --

#[test]
fn drift_path_filter() {
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

    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &["src/a.rs"],
    );
    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/mcp/card_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_b",
        "stable",
        Some("2026-03-01"),
        &["src/b.rs"],
    );

    git_init_and_commit(tmp.path());

    git_commit_file_with_date(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() { /* changed */ }\n",
        "2026-04-05T12:00:00+00:00",
    );
    git_commit_file_with_date(
        tmp.path(),
        "src/b.rs",
        "// @specre 01BBBBBBBBBBBBBBBBBBBBBBBB\nfn b() { /* changed */ }\n",
        "2026-04-05T12:00:00+00:00",
    );

    // Filter by path: only cli cards
    let output = specre()
        .args(["drift", "docs/specres/cli/", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
    let drifted = json["drifted"].as_array().unwrap();
    assert_eq!(drifted.len(), 1);
    assert_eq!(drifted[0]["id"], "01AAAAAAAAAAAAAAAAAAAAAAAA");
}

// -- Scenario: Domain filtering --

#[test]
fn drift_domain_filter() {
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

    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &["src/a.rs"],
    );
    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/mcp/card_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_b",
        "stable",
        Some("2026-03-01"),
        &["src/b.rs"],
    );

    git_init_and_commit(tmp.path());

    git_commit_file_with_date(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() { /* changed */ }\n",
        "2026-04-05T12:00:00+00:00",
    );
    git_commit_file_with_date(
        tmp.path(),
        "src/b.rs",
        "// @specre 01BBBBBBBBBBBBBBBBBBBBBBBB\nfn b() { /* changed */ }\n",
        "2026-04-05T12:00:00+00:00",
    );

    let output = specre()
        .args(["drift", "--domain", "mcp", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
    let drifted = json["drifted"].as_array().unwrap();
    assert_eq!(drifted.len(), 1);
    assert_eq!(drifted[0]["domain"], "mcp");
}

// -- Scenario: Status filtering (only stable by default) --

#[test]
fn drift_skips_draft_by_default() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    // Draft specre — should be skipped
    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "draft",
        Some("2026-03-01"),
        &["src/a.rs"],
    );

    git_init_and_commit(tmp.path());

    git_commit_file_with_date(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() { /* changed */ }\n",
        "2026-04-05T12:00:00+00:00",
    );

    let output = specre()
        .args(["drift", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "should exit 0 when only draft specres exist"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 0);
    assert_eq!(json["drifted"].as_array().unwrap().len(), 0);
}

// -- Scenario: Status filtering with --status override --

#[test]
fn drift_status_override() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

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
        "in-development",
        Some("2026-03-01"),
        &["src/a.rs"],
    );

    git_init_and_commit(tmp.path());

    git_commit_file_with_date(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() { /* changed */ }\n",
        "2026-04-05T12:00:00+00:00",
    );

    let output = specre()
        .args(["drift", "--status", "in-development", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
    assert_eq!(json["drifted"].as_array().unwrap().len(), 1);
}

// -- Scenario: Grace period from specre.toml --

#[test]
fn drift_grace_period_hides_recent_changes() {
    let tmp = TempDir::new().unwrap();
    write_config_with_drift(tmp.path(), "docs/specres", &["src"], 7);

    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    // last_verified: 2026-04-01, source changed 2026-04-05 (4 days later, within 7-day grace)
    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-04-01"),
        &["src/a.rs"],
    );

    git_init_and_commit(tmp.path());

    git_commit_file_with_date(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() { /* changed */ }\n",
        "2026-04-05T12:00:00+00:00",
    );

    let output = specre()
        .args(["drift", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "should exit 0 when change is within grace period"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["drifted"].as_array().unwrap().len(), 0);
    assert_eq!(json["grace_days"], 7);
}

// -- Scenario: Grace period CLI override --

#[test]
fn drift_grace_cli_override() {
    let tmp = TempDir::new().unwrap();
    // specre.toml has grace_days = 7
    write_config_with_drift(tmp.path(), "docs/specres", &["src"], 7);

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
        Some("2026-04-01"),
        &["src/a.rs"],
    );

    git_init_and_commit(tmp.path());

    git_commit_file_with_date(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() { /* changed */ }\n",
        "2026-04-05T12:00:00+00:00",
    );

    // Override grace to 0d — should now detect drift
    let output = specre()
        .args(["drift", "--grace", "0d", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "should detect drift with 0d grace override"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["drifted"].as_array().unwrap().len(), 1);
    assert_eq!(json["grace_days"], 0);
}

// -- Scenario: Specre with no last_verified date --

#[test]
fn drift_no_last_verified_always_drifted() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    write_source(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n",
    );

    // No last_verified
    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        None,
        &["src/a.rs"],
    );

    git_init_and_commit(tmp.path());

    let output = specre()
        .args(["drift", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "should exit 1 when specre has no last_verified"
    );

    let json = parse_json(&output.stdout);
    let drifted = json["drifted"].as_array().unwrap();
    assert_eq!(drifted.len(), 1);
    assert!(drifted[0]["last_verified"].is_null());
}

// -- Scenario: Specre with no related source files --

#[test]
fn drift_no_related_files_is_clean() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    // Source file exists but does NOT reference this specre
    write_source(tmp.path(), "src/a.rs", "fn a() {}\n");

    // Specre has no related files and no markers point to it
    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &[],
    );

    git_init_and_commit(tmp.path());

    let output = specre()
        .args(["drift", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "should exit 0 when specre has no related files"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["drifted"].as_array().unwrap().len(), 0);
    assert_eq!(json["clean"], 1);
}

// -- Scenario: Related files from both Related Files section and @specre markers --

#[test]
fn drift_resolves_from_both_related_files_and_markers() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    // foo.rs is listed in Related Files, bar.rs has a @specre marker
    write_source(tmp.path(), "src/foo.rs", "fn foo() {}\n");
    write_source(
        tmp.path(),
        "src/bar.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn bar() {}\n",
    );

    write_specre_card_with_verified(
        tmp.path(),
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        Some("2026-03-01"),
        &["src/foo.rs"],
    );

    git_init_and_commit(tmp.path());

    // Modify bar.rs (linked via marker only)
    git_commit_file_with_date(
        tmp.path(),
        "src/bar.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn bar() { /* changed */ }\n",
        "2026-04-05T12:00:00+00:00",
    );

    let output = specre()
        .args(["drift", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let json = parse_json(&output.stdout);
    let drifted = json["drifted"].as_array().unwrap();
    assert_eq!(drifted.len(), 1);
    let changed = drifted[0]["changed_files"].as_array().unwrap();
    assert!(
        changed.iter().any(|f| f["file"] == "src/bar.rs"),
        "should include bar.rs from @specre marker"
    );
}

// -- Scenario: JSON output schema --

#[test]
fn drift_json_output_schema() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

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
        Some("2026-04-01"),
        &["src/a.rs"],
    );

    git_init_and_commit(tmp.path());

    let output = specre()
        .args(["drift", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let json = parse_json(&output.stdout);

    // Verify top-level fields
    assert!(json["drifted"].is_array());
    assert!(json["clean"].is_u64());
    assert!(json["total"].is_u64());
    assert!(json["grace_days"].is_u64());
}

// -- Scenario: Human-readable output --

#[test]
fn drift_human_readable_output() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

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
        Some("2026-03-01"),
        &["src/a.rs"],
    );

    git_init_and_commit(tmp.path());

    git_commit_file_with_date(
        tmp.path(),
        "src/a.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() { /* changed */ }\n",
        "2026-04-05T12:00:00+00:00",
    );

    let output = specre()
        .args(["drift"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("drifted"),
        "should contain 'drifted' summary"
    );
    assert!(
        stdout.contains("01AAAAAAAAAAAAAAAAAAAAAAAA"),
        "should contain ULID"
    );
    assert!(stdout.contains("src/a.rs"), "should contain changed file");
}

// -- Scenario: No specre.toml present --

#[test]
fn drift_errors_without_config() {
    let tmp = TempDir::new().unwrap();

    // Initialize git but no specre.toml
    git_init_and_commit(tmp.path());

    specre()
        .args(["drift"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "specre.toml not found. Run 'specre init' first.",
        ));
}

// -- Scenario: Non-git repository --

#[test]
fn drift_errors_without_git() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

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
        Some("2026-03-01"),
        &["src/a.rs"],
    );

    // No git init

    specre()
        .args(["drift"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("git"));
}

// -- Scenario: Invalid grace format --

#[test]
fn drift_invalid_grace_format() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    git_init_and_commit(tmp.path());

    specre()
        .args(["drift", "--grace", "abc"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("grace"));
}
