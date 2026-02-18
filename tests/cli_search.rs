// @specre 01KHFTCYJN8YJMW2RNHJTAQV49
// @specre 01KHQBKWZY2D77XP7A50HGTZQ8
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

fn write_config_with_search(
    dir: &std::path::Path,
    specre_dir: &str,
    source_dirs: &[&str],
    max_results: usize,
) {
    let dirs_toml: Vec<String> = source_dirs.iter().map(|s| format!("\"{s}\"")).collect();
    let content = format!(
        "specre_dir = \"{specre_dir}\"\nsource_dirs = [{}]\n\n[search]\nmax_results = {max_results}\n",
        dirs_toml.join(", ")
    );
    fs::write(dir.join("specre.toml"), content).unwrap();
}

fn write_glossary(dir: &std::path::Path, terms: &[&str]) {
    let terms_toml: Vec<String> = terms.iter().map(|s| format!("  \"{s}\"")).collect();
    let content = format!("terms = [\n{},\n]\n", terms_toml.join(",\n"));
    fs::write(dir.join("glossary.toml"), content).unwrap();
}

fn write_specre_card_full(
    dir: &std::path::Path,
    rel_path: &str,
    id: &str,
    name: &str,
    status: &str,
    last_verified: Option<&str>,
    body: &str,
) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let lv = last_verified.map_or_else(String::new, |d| format!("last_verified: \"{d}\"\n"));
    let content = format!(
        "---\nid: \"{id}\"\nname: \"{name}\"\nstatus: \"{status}\"\n{lv}---\n\n{body}"
    );
    fs::write(path, content).unwrap();
}

fn write_specre_card(dir: &std::path::Path, rel_path: &str, id: &str, name: &str, status: &str) {
    write_specre_card_full(
        dir,
        rel_path,
        id,
        name,
        status,
        None,
        "## Related Files\n\n## Functional Overview\n\nA sample overview.\n\n## Scenarios\n",
    );
}

fn parse_json(stdout: &[u8]) -> serde_json::Value {
    serde_json::from_slice(stdout).expect("output should be valid JSON")
}

// -- Scenario: Free-text query matches card content --

#[test]
fn search_free_text_matches_body() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_reset_password.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "user_can_reset_password",
        "stable",
        Some("2026-03-01"),
        "## Related Files\n\n## Functional Overview\n\nUsers can reset their password by providing their registered email address.\n\n## Scenarios\n",
    );
    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_login.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "user_can_login",
        "stable",
        Some("2026-03-01"),
        "## Related Files\n\n## Functional Overview\n\nUsers can login with email and credentials.\n\n## Scenarios\n",
    );

    let output = specre()
        .args(["search", "password"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
    assert_eq!(json["results"][0]["name"], "user_can_reset_password");
    assert_eq!(json["results"][0]["domain"], "auth");
    assert_eq!(json["results"][0]["status"], "stable");
    assert!(json["results"][0]["path"]
        .as_str()
        .unwrap()
        .contains("auth/user_can_reset_password.md"));
    assert_eq!(json["results"][0]["last_verified"], "2026-03-01");
    assert!(json["results"][0]["excerpt"].as_str().unwrap().contains("password"));
}

// -- Scenario: Free-text query matches front-matter name --

#[test]
fn search_free_text_matches_name() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card(
        &tmp,
        "docs/specres/cli/specre_orphans_detects_issues.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "specre_orphans_detects_issues",
        "stable",
    );
    write_specre_card(
        &tmp,
        "docs/specres/cli/specre_trace_resolves.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "specre_trace_resolves",
        "stable",
    );

    let output = specre()
        .args(["search", "orphans"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
    assert_eq!(json["results"][0]["name"], "specre_orphans_detects_issues");
}

// -- Scenario: Case-insensitive matching --

#[test]
fn search_is_case_insensitive() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_reset_password.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "user_can_reset_password",
        "stable",
        None,
        "## Functional Overview\n\nUsers can RESET their Password.\n",
    );

    let output = specre()
        .args(["search", "RESET"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
}

// -- Scenario: Filter by status only (no text query) --

#[test]
fn search_filter_by_status() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card(
        &tmp,
        "docs/specres/cli/card_draft.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_draft",
        "draft",
    );
    write_specre_card(
        &tmp,
        "docs/specres/cli/card_stable.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_stable",
        "stable",
    );

    let output = specre()
        .args(["search", "--status", "draft"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
    assert_eq!(json["results"][0]["name"], "card_draft");
}

// -- Scenario: Filter by domain --

#[test]
fn search_filter_by_domain() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card(
        &tmp,
        "docs/specres/auth/card_auth.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_auth",
        "stable",
    );
    write_specre_card(
        &tmp,
        "docs/specres/cli/card_cli.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_cli",
        "stable",
    );

    let output = specre()
        .args(["search", "--domain", "auth"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
    assert_eq!(json["results"][0]["name"], "card_auth");
    assert_eq!(json["results"][0]["domain"], "auth");
}

// -- Scenario: Filter by verified-before --

#[test]
fn search_filter_verified_before() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card_full(
        &tmp,
        "docs/specres/cli/card_old.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_old",
        "stable",
        Some("2026-01-15"),
        "## Functional Overview\n\nOld card.\n",
    );
    write_specre_card_full(
        &tmp,
        "docs/specres/cli/card_new.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_new",
        "stable",
        Some("2026-02-15"),
        "## Functional Overview\n\nNew card.\n",
    );
    // Card with no last_verified should be included (never verified = "before" any date)
    write_specre_card(
        &tmp,
        "docs/specres/cli/card_no_date.md",
        "01CCCCCCCCCCCCCCCCCCCCCCCC",
        "card_no_date",
        "draft",
    );

    let output = specre()
        .args(["search", "--verified-before", "2026-02-01"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 2);
    let names: Vec<&str> = json["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"card_old"));
    assert!(names.contains(&"card_no_date"));
    assert!(!names.contains(&"card_new"));
}

// -- Scenario: Filter by verified-after --

#[test]
fn search_filter_verified_after() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card_full(
        &tmp,
        "docs/specres/cli/card_old.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_old",
        "stable",
        Some("2026-01-15"),
        "## Functional Overview\n\nOld card.\n",
    );
    write_specre_card_full(
        &tmp,
        "docs/specres/cli/card_new.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_new",
        "stable",
        Some("2026-02-15"),
        "## Functional Overview\n\nNew card.\n",
    );
    // Card with no last_verified should be excluded
    write_specre_card(
        &tmp,
        "docs/specres/cli/card_no_date.md",
        "01CCCCCCCCCCCCCCCCCCCCCCCC",
        "card_no_date",
        "draft",
    );

    let output = specre()
        .args(["search", "--verified-after", "2026-02-01"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
    assert_eq!(json["results"][0]["name"], "card_new");
}

// -- Scenario: Combining text query with filters --

#[test]
fn search_combined_query_and_filters() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_validate_email.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "user_can_validate_email",
        "stable",
        None,
        "## Functional Overview\n\nValidation of email addresses.\n",
    );
    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_login.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "user_can_login",
        "draft",
        None,
        "## Functional Overview\n\nLogin with validation.\n",
    );
    write_specre_card_full(
        &tmp,
        "docs/specres/cli/cli_validates_input.md",
        "01CCCCCCCCCCCCCCCCCCCCCCCC",
        "cli_validates_input",
        "stable",
        None,
        "## Functional Overview\n\nCLI input validation.\n",
    );

    let output = specre()
        .args(["search", "validation", "--status", "stable", "--domain", "auth"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
    assert_eq!(json["results"][0]["name"], "user_can_validate_email");
}

// -- Scenario: No parameters — returns all specres --

#[test]
fn search_no_params_returns_all() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card(
        &tmp,
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "draft",
    );
    write_specre_card(
        &tmp,
        "docs/specres/cli/card_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_b",
        "stable",
    );

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 2);
    assert_eq!(json["results"].as_array().unwrap().len(), 2);
}

// -- Scenario: Results exceed truncation threshold --

#[test]
fn search_truncated_when_exceeds_threshold() {
    let tmp = TempDir::new().unwrap();
    write_config_with_search(&tmp, "docs/specres", &["src"], 3);

    // Create 5 cards across 2 domains with mixed statuses
    write_specre_card(
        &tmp,
        "docs/specres/auth/card_1.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_1",
        "stable",
    );
    write_specre_card(
        &tmp,
        "docs/specres/auth/card_2.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_2",
        "draft",
    );
    write_specre_card(
        &tmp,
        "docs/specres/cli/card_3.md",
        "01CCCCCCCCCCCCCCCCCCCCCCCC",
        "card_3",
        "stable",
    );
    write_specre_card(
        &tmp,
        "docs/specres/cli/card_4.md",
        "01DDDDDDDDDDDDDDDDDDDDDD",
        "card_4",
        "in-development",
    );
    write_specre_card(
        &tmp,
        "docs/specres/cli/card_5.md",
        "01EEEEEEEEEEEEEEEEEEEEEE",
        "card_5",
        "stable",
    );

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 5);
    assert_eq!(json["truncated"], true);
    assert!(json["results"].as_array().unwrap().is_empty());
    // hint should be present
    assert!(json["hint"]["message"].as_str().unwrap().contains('5'));
    let domains = json["hint"]["available_domains"].as_array().unwrap();
    let domain_strs: Vec<&str> = domains.iter().map(|d| d.as_str().unwrap()).collect();
    assert!(domain_strs.contains(&"auth"));
    assert!(domain_strs.contains(&"cli"));
    // status_counts
    let counts = &json["hint"]["status_counts"];
    assert_eq!(counts["stable"], 3);
    assert_eq!(counts["draft"], 1);
    assert_eq!(counts["in-development"], 1);
}

// -- Scenario: Results within truncation threshold --

#[test]
fn search_not_truncated_within_threshold() {
    let tmp = TempDir::new().unwrap();
    write_config_with_search(&tmp, "docs/specres", &["src"], 10);

    write_specre_card(
        &tmp,
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
    assert_eq!(json["truncated"], false);
    assert!(json.get("hint").is_none() || json["hint"].is_null());
    assert_eq!(json["results"].as_array().unwrap().len(), 1);
}

// -- Scenario: --limit overrides truncation threshold --

#[test]
fn search_limit_overrides_threshold() {
    let tmp = TempDir::new().unwrap();
    write_config_with_search(&tmp, "docs/specres", &["src"], 2);

    write_specre_card(
        &tmp,
        "docs/specres/auth/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );
    write_specre_card(
        &tmp,
        "docs/specres/auth/card_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_b",
        "stable",
    );
    write_specre_card(
        &tmp,
        "docs/specres/cli/card_c.md",
        "01CCCCCCCCCCCCCCCCCCCCCCCC",
        "card_c",
        "stable",
    );
    write_specre_card(
        &tmp,
        "docs/specres/cli/card_d.md",
        "01DDDDDDDDDDDDDDDDDDDDDD",
        "card_d",
        "stable",
    );

    let output = specre()
        .args(["search", "--limit", "3"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 4);
    assert_eq!(json["truncated"], false);
    assert_eq!(json["results"].as_array().unwrap().len(), 3);
}

// -- Scenario: Default truncation threshold (10) --

#[test]
fn search_default_threshold_is_10() {
    let tmp = TempDir::new().unwrap();
    // No [search] section in config
    write_config(&tmp, "docs/specres", &["src"]);

    // Create 11 cards to exceed the default threshold of 10
    for i in 0..11 {
        let id = format!("01{:0>24}", format!("{i}AAAAAAAAAAAAAAAAAAAAAAA"));
        write_specre_card(
            &tmp,
            &format!("docs/specres/cli/card_{i}.md"),
            &id[..26],
            &format!("card_{i}"),
            "stable",
        );
    }

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 11);
    assert_eq!(json["truncated"], true);
    assert!(json["results"].as_array().unwrap().is_empty());
}

// -- Scenario: No results found --

#[test]
fn search_no_results() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card(
        &tmp,
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    let output = specre()
        .args(["search", "nonexistent_term_xyz"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 0);
    assert!(json["results"].as_array().unwrap().is_empty());
}

// -- Scenario: Excerpt extraction --

#[test]
fn search_excerpt_from_first_prose_paragraph() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card_full(
        &tmp,
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        None,
        "## Related Files\n\n- `src/foo.rs`\n\n## Functional Overview\n\nThis is the overview paragraph.\n\n## Scenarios\n",
    );

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(
        json["results"][0]["excerpt"].as_str().unwrap(),
        "This is the overview paragraph."
    );
}

#[test]
fn search_excerpt_truncated_at_200_chars() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    let long_text = "A".repeat(250);
    write_specre_card_full(
        &tmp,
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        None,
        &format!("## Functional Overview\n\n{long_text}\n"),
    );

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    let excerpt = json["results"][0]["excerpt"].as_str().unwrap();
    // 200 chars + ellipsis
    assert!(excerpt.ends_with('\u{2026}'));
    // The excerpt before ellipsis should be 200 chars
    assert_eq!(excerpt.chars().count(), 201); // 200 + 1 ellipsis
}

#[test]
fn search_excerpt_null_when_no_prose() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card_full(
        &tmp,
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        None,
        "## Section One\n\n- list item\n- another\n\n## Section Two\n\n- more list\n",
    );

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert!(json["results"][0]["excerpt"].is_null());
}

// -- Scenario: Results are sorted by domain then name --

#[test]
fn search_results_sorted_by_domain_then_name() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card(
        &tmp,
        "docs/specres/cli/zebra.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "zebra",
        "stable",
    );
    write_specre_card(
        &tmp,
        "docs/specres/auth/alpha.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "alpha",
        "stable",
    );
    write_specre_card(
        &tmp,
        "docs/specres/auth/beta.md",
        "01CCCCCCCCCCCCCCCCCCCCCCCC",
        "beta",
        "stable",
    );
    write_specre_card(
        &tmp,
        "docs/specres/cli/apple.md",
        "01DDDDDDDDDDDDDDDDDDDDDD",
        "apple",
        "stable",
    );

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    let names: Vec<&str> = json["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["alpha", "beta", "apple", "zebra"]);
}

// -- Scenario: Paths use forward slashes --

#[test]
fn search_paths_use_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card(
        &tmp,
        "docs/specres/auth/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    let path = json["results"][0]["path"].as_str().unwrap();
    assert!(!path.contains('\\'));
    assert!(path.contains('/'));
}

// -- Scenario: specre.toml does not exist --

#[test]
fn search_errors_without_config() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["search", "anything"])
        .current_dir(&tmp)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "specre.toml not found. Run 'specre init' first.",
        ));
}

// -- Scenario: specre_dir does not exist --

#[test]
fn search_empty_when_specre_dir_missing() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);
    // Don't create docs/specres directory

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 0);
    assert!(json["results"].as_array().unwrap().is_empty());
}

// -- Scenario: Malformed front-matter is skipped --

#[test]
fn search_skips_malformed_cards() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card(
        &tmp,
        "docs/specres/cli/good_card.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "good_card",
        "stable",
    );
    // Malformed card: no front-matter
    let bad_path = tmp.path().join("docs/specres/cli/bad_card.md");
    fs::write(&bad_path, "No front-matter here.\n").unwrap();

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
    assert_eq!(json["results"][0]["name"], "good_card");
}

// -- Scenario: Invalid status value --

#[test]
fn search_invalid_status_error() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    specre()
        .args(["search", "--status", "invalid"])
        .current_dir(&tmp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid status"));
}

// -- Scenario: Invalid date format --

#[test]
fn search_invalid_date_error() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    specre()
        .args(["search", "--verified-before", "not-a-date"])
        .current_dir(&tmp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid date format"));
}

// -- Scenario: Calendar-invalid date rejected --

#[test]
fn search_rejects_calendar_invalid_date() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    // Feb 30 does not exist
    specre()
        .args(["search", "--verified-before", "2025-02-30"])
        .current_dir(&tmp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid date format"));

    // Month 13 does not exist
    specre()
        .args(["search", "--verified-after", "2025-13-01"])
        .current_dir(&tmp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid date format"));

    // Month 00 does not exist
    specre()
        .args(["search", "--verified-before", "2025-00-15"])
        .current_dir(&tmp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid date format"));

    // Day 00 does not exist
    specre()
        .args(["search", "--verified-after", "2025-01-00"])
        .current_dir(&tmp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid date format"));
}

// -- Scenario: --limit must be positive --

#[test]
fn search_limit_zero_error() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    specre()
        .args(["search", "--limit", "0"])
        .current_dir(&tmp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("--limit must be a positive integer"));
}

// -- Scenario: Excerpt with multiline prose paragraph --

#[test]
fn search_excerpt_joins_multiline_paragraph() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card_full(
        &tmp,
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        None,
        "## Functional Overview\n\nFirst line of overview.\nSecond line of overview.\n\n## Scenarios\n",
    );

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    let excerpt = json["results"][0]["excerpt"].as_str().unwrap();
    assert_eq!(excerpt, "First line of overview. Second line of overview.");
}

// -- Scenario: Multi-keyword AND search (default) --

#[test]
fn search_multi_keyword_and() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    // Card A: contains both "password" and "reset"
    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_reset_password.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "user_can_reset_password",
        "stable",
        None,
        "## Functional Overview\n\nUsers can reset their password via email.\n",
    );
    // Card B: contains "password" but not "reset"
    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_change_password.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "user_can_change_password",
        "stable",
        None,
        "## Functional Overview\n\nUsers can change their password from settings.\n",
    );
    // Card C: contains "reset" but not "password"
    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_reset_session.md",
        "01CCCCCCCCCCCCCCCCCCCCCCCC",
        "user_can_reset_session",
        "stable",
        None,
        "## Functional Overview\n\nUsers can reset their active session.\n",
    );

    let output = specre()
        .args(["search", "password reset"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
    assert_eq!(json["results"][0]["name"], "user_can_reset_password");
}

// -- Scenario: Multi-keyword OR search --

#[test]
fn search_multi_keyword_or() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    // Card A: contains both "password" and "reset"
    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_reset_password.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "user_can_reset_password",
        "stable",
        None,
        "## Functional Overview\n\nUsers can reset their password via email.\n",
    );
    // Card B: contains "password" but not "reset"
    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_change_password.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "user_can_change_password",
        "stable",
        None,
        "## Functional Overview\n\nUsers can change their password from settings.\n",
    );
    // Card C: contains "reset" but not "password"
    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_reset_session.md",
        "01CCCCCCCCCCCCCCCCCCCCCCCC",
        "user_can_reset_session",
        "stable",
        None,
        "## Functional Overview\n\nUsers can reset their active session.\n",
    );
    // Card D: contains neither "password" nor "reset"
    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_login.md",
        "01DDDDDDDDDDDDDDDDDDDDDD",
        "user_can_login",
        "stable",
        None,
        "## Functional Overview\n\nUsers can login with email.\n",
    );

    let output = specre()
        .args(["search", "password reset", "--or"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 3);
    let names: Vec<&str> = json["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"user_can_reset_password"));
    assert!(names.contains(&"user_can_change_password"));
    assert!(names.contains(&"user_can_reset_session"));
    assert!(!names.contains(&"user_can_login"));
}

// -- Scenario: Single keyword behaves identically with and without --or --

#[test]
fn search_single_keyword_same_with_or_without_or_flag() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_reset_password.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "user_can_reset_password",
        "stable",
        None,
        "## Functional Overview\n\nUsers can reset their password.\n",
    );
    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_login.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "user_can_login",
        "stable",
        None,
        "## Functional Overview\n\nUsers can login with email.\n",
    );

    let output_and = specre()
        .args(["search", "password"])
        .current_dir(&tmp)
        .output()
        .unwrap();
    let output_or = specre()
        .args(["search", "password", "--or"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output_and.status.success());
    assert!(output_or.status.success());

    let json_and = parse_json(&output_and.stdout);
    let json_or = parse_json(&output_or.stdout);
    assert_eq!(json_and["total"], json_or["total"]);
    assert_eq!(json_and["total"], 1);
    assert_eq!(json_and["results"][0]["name"], "user_can_reset_password");
    assert_eq!(json_or["results"][0]["name"], "user_can_reset_password");
}

// -- Failures / Exceptions: IO error warns and skips --

#[cfg(unix)]
#[test]
fn search_warns_on_unreadable_specre_card() {
    use std::os::unix::fs::PermissionsExt;
    if common::is_root() {
        return; // root bypasses file-permission checks
    }

    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    write_specre_card(
        &tmp,
        "docs/specres/cli/good_card.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "good_card",
        "stable",
    );

    // Create an unreadable card
    let bad_card = tmp.path().join("docs/specres/cli/bad_card.md");
    fs::write(
        &bad_card,
        "---\nid: \"01BBBBBBBBBBBBBBBBBBBBBBBB\"\nname: \"bad_card\"\nstatus: \"draft\"\n---\n",
    )
    .unwrap();
    fs::set_permissions(&bad_card, fs::Permissions::from_mode(0o000)).unwrap();

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning: failed to read"),
        "Expected warning about unreadable file in stderr, got: {stderr}"
    );

    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
    assert_eq!(json["results"][0]["name"], "good_card");

    fs::set_permissions(&bad_card, fs::Permissions::from_mode(0o644)).unwrap();
}

// -- Scenario: No results with multi-keyword AND — keyword match counts --

#[test]
fn search_no_results_multi_keyword_shows_keyword_matches() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);
    // No glossary.toml

    // Card with "password" but not "reset"
    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_change_password.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "user_can_change_password",
        "stable",
        None,
        "## Functional Overview\n\nUsers can change their password from settings.\n",
    );
    // Card with neither
    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_login.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "user_can_login",
        "stable",
        None,
        "## Functional Overview\n\nUsers can login with email.\n",
    );

    let output = specre()
        .args(["search", "password reset"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 0);
    assert_eq!(json["truncated"], false);

    // hint should have keyword_matches
    let hint = &json["hint"];
    assert!(
        hint["message"]
            .as_str()
            .unwrap()
            .contains("No results found"),
    );

    let kw_matches = hint["keyword_matches"].as_array().unwrap();
    assert_eq!(kw_matches.len(), 2);
    // Sorted by match_count descending: password=1, reset=0
    assert_eq!(kw_matches[0]["keyword"], "password");
    assert_eq!(kw_matches[0]["match_count"], 1);
    assert_eq!(kw_matches[1]["keyword"], "reset");
    assert_eq!(kw_matches[1]["match_count"], 0);

    // No suggested_terms (no glossary)
    assert!(hint.get("suggested_terms").is_none() || hint["suggested_terms"].is_null());
}

// -- Scenario: No results with glossary — vocabulary suggestions --

#[test]
fn search_no_results_with_glossary_shows_suggested_terms() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);
    write_glossary(&tmp, &["user", "authentication", "password", "session"]);

    // Cards with "authentication" and "password" but not "login"
    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_authenticate.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "user_can_authenticate",
        "stable",
        None,
        "## Functional Overview\n\nUsers authenticate via authentication service.\n",
    );
    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_reset_password.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "user_can_reset_password",
        "stable",
        None,
        "## Functional Overview\n\nUsers can reset their password.\n",
    );

    let output = specre()
        .args(["search", "login"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 0);

    let hint = &json["hint"];
    assert!(hint["message"].as_str().unwrap().contains("No results found"));

    // keyword_matches present
    let kw_matches = hint["keyword_matches"].as_array().unwrap();
    assert_eq!(kw_matches[0]["keyword"], "login");
    assert_eq!(kw_matches[0]["match_count"], 0);

    // suggested_terms present (glossary terms that match cards, excluding "login")
    let suggested = hint["suggested_terms"].as_array().unwrap();
    assert!(!suggested.is_empty());
    // All suggested terms have match_count > 0
    for term in suggested {
        assert!(term["match_count"].as_u64().unwrap() > 0);
    }
    // "login" should NOT appear in suggested_terms (it's in the query)
    let term_names: Vec<&str> = suggested.iter().map(|t| t["term"].as_str().unwrap()).collect();
    assert!(!term_names.contains(&"login"));
    // "authentication" and "password" should be present
    assert!(term_names.contains(&"authentication"));
    assert!(term_names.contains(&"password"));
}

// -- Scenario: Single keyword, no glossary — no hint (existing behavior) --

#[test]
fn search_no_results_single_keyword_no_glossary_no_hint() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);
    // No glossary

    write_specre_card(
        &tmp,
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    let output = specre()
        .args(["search", "nonexistent_xyz"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 0);
    // No hint field
    assert!(json.get("hint").is_none() || json["hint"].is_null());
}

// -- Scenario: Single keyword with glossary — hint with suggested_terms --

#[test]
fn search_no_results_single_keyword_with_glossary_shows_hint() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);
    write_glossary(&tmp, &["user", "authentication"]);

    write_specre_card_full(
        &tmp,
        "docs/specres/auth/user_can_authenticate.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "user_can_authenticate",
        "stable",
        None,
        "## Functional Overview\n\nAuthentication via user credentials.\n",
    );

    let output = specre()
        .args(["search", "login"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 0);

    // Hint should be present (glossary exists)
    let hint = &json["hint"];
    assert!(hint["message"].as_str().is_some());
    assert!(!hint["suggested_terms"].as_array().unwrap().is_empty());
}

// -- Scenario: Results exceed truncation threshold with glossary --

#[test]
fn search_truncated_with_glossary_shows_suggested_terms() {
    let tmp = TempDir::new().unwrap();
    write_config_with_search(&tmp, "docs/specres", &["src"], 3);
    write_glossary(&tmp, &["create", "delete", "user", "overview"]);

    // Create 5 cards — exceeds threshold of 3
    // 2 cards mention "create", 1 mentions "delete", all mention "overview"
    write_specre_card_full(
        &tmp,
        "docs/specres/cli/card_1.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_1",
        "stable",
        None,
        "## Functional Overview\n\nCreate user overview.\n",
    );
    write_specre_card_full(
        &tmp,
        "docs/specres/cli/card_2.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_2",
        "stable",
        None,
        "## Functional Overview\n\nCreate item overview.\n",
    );
    write_specre_card_full(
        &tmp,
        "docs/specres/cli/card_3.md",
        "01CCCCCCCCCCCCCCCCCCCCCCCC",
        "card_3",
        "stable",
        None,
        "## Functional Overview\n\nDelete item overview.\n",
    );
    write_specre_card_full(
        &tmp,
        "docs/specres/cli/card_4.md",
        "01DDDDDDDDDDDDDDDDDDDDDD",
        "card_4",
        "stable",
        None,
        "## Functional Overview\n\nList items overview.\n",
    );
    write_specre_card_full(
        &tmp,
        "docs/specres/cli/card_5.md",
        "01EEEEEEEEEEEEEEEEEEEEEE",
        "card_5",
        "stable",
        None,
        "## Functional Overview\n\nUser profile overview.\n",
    );

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 5);
    assert_eq!(json["truncated"], true);

    let hint = &json["hint"];
    // Existing fields still present
    assert!(hint["available_domains"].as_array().is_some());
    assert!(hint["status_counts"].is_object());

    // suggested_terms present
    let suggested = hint["suggested_terms"].as_array().unwrap();
    assert!(!suggested.is_empty());

    // "overview" matches all 5 cards (== total), so it should be EXCLUDED
    let term_names: Vec<&str> = suggested.iter().map(|t| t["term"].as_str().unwrap()).collect();
    assert!(
        !term_names.contains(&"overview"),
        "Terms matching all cards should be excluded"
    );

    // "create" and "delete" should be present (match some but not all)
    assert!(term_names.contains(&"create"));
    assert!(term_names.contains(&"delete"));

    // Sorted by match_count descending
    let counts: Vec<u64> = suggested.iter().map(|t| t["match_count"].as_u64().unwrap()).collect();
    for window in counts.windows(2) {
        assert!(window[0] >= window[1], "suggested_terms should be sorted descending");
    }
}

// -- Scenario: Truncation without glossary — no suggested_terms (unchanged) --

#[test]
fn search_truncated_without_glossary_no_suggested_terms() {
    let tmp = TempDir::new().unwrap();
    write_config_with_search(&tmp, "docs/specres", &["src"], 2);
    // No glossary

    for i in 0..3 {
        write_specre_card(
            &tmp,
            &format!("docs/specres/cli/card_{i}.md"),
            &format!("01{i:A<24}"),
            &format!("card_{i}"),
            "stable",
        );
    }

    let output = specre()
        .args(["search"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["truncated"], true);

    let hint = &json["hint"];
    assert!(hint["available_domains"].as_array().is_some());
    assert!(hint["status_counts"].is_object());
    // No suggested_terms
    assert!(
        hint.get("suggested_terms").is_none() || hint["suggested_terms"].is_null(),
    );
}

// -- Scenario: Malformed glossary.toml warns and continues --

#[test]
fn search_glossary_malformed_warns_and_continues() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);

    // Write invalid glossary.toml
    fs::write(tmp.path().join("glossary.toml"), "this is not valid toml [[[").unwrap();

    write_specre_card_full(
        &tmp,
        "docs/specres/cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        None,
        "## Functional Overview\n\nA sample card.\n",
    );

    let output = specre()
        .args(["search", "sample"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());

    // Should warn on stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning: failed to parse glossary.toml"),
        "Expected glossary warning in stderr, got: {stderr}"
    );

    // Search still works
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 1);
    assert_eq!(json["results"][0]["name"], "card_a");
}

// -- Scenario: suggested_terms excludes query terms --

#[test]
fn search_glossary_excludes_query_terms_from_suggestions() {
    let tmp = TempDir::new().unwrap();
    write_config(&tmp, "docs/specres", &["src"]);
    write_glossary(&tmp, &["password", "user", "authentication"]);

    write_specre_card_full(
        &tmp,
        "docs/specres/auth/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
        None,
        "## Functional Overview\n\nUser authentication with password.\n",
    );

    // Search for "password" — it matches, but with glossary + another keyword "xyz" to get 0 results
    let output = specre()
        .args(["search", "password xyz"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["total"], 0);

    let suggested = json["hint"]["suggested_terms"].as_array().unwrap();
    let term_names: Vec<&str> = suggested.iter().map(|t| t["term"].as_str().unwrap()).collect();
    // "password" is in query, should be excluded from suggestions
    assert!(!term_names.contains(&"password"));
    // "xyz" is in query, should be excluded (even if not in glossary — doesn't matter)
    assert!(!term_names.contains(&"xyz"));
    // Other glossary terms with matches should be present
    assert!(term_names.contains(&"user"));
    assert!(term_names.contains(&"authentication"));
}
