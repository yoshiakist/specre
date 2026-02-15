// @specre 01KHFTCYJN8YJMW2RNHJTAQV49
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
    let lv = match last_verified {
        Some(d) => format!("last_verified: \"{d}\"\n"),
        None => String::new(),
    };
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
    assert!(json["hint"]["message"].as_str().unwrap().contains("5"));
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
