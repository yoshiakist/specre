// @specre 01KHAN6JE712ZAKXPP97854PKJ
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use chrono::{Days, Utc};
use predicates::prelude::*;
use std::fs;

fn specre() -> assert_cmd::Command {
    cargo_bin_cmd!("specre")
}

/// Helper: create specre.toml with given specre_dir
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

fn today_str() -> String {
    Utc::now().date_naive().format("%Y-%m-%d").to_string()
}

fn days_ago_str(days: u64) -> String {
    Utc::now()
        .date_naive()
        .checked_sub_days(Days::new(days))
        .unwrap()
        .format("%Y-%m-%d")
        .to_string()
}

// -- Scenario: Basic invocation shows status summary --

#[test]
fn status_shows_summary() {
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
        "01BBBBBBBBBBBBBBBBBBBBBBBBBB",
        "spec_b",
        "draft",
        None,
    );
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_c.md",
        "01CCCCCCCCCCCCCCCCCCCCCCCCCC",
        "spec_c",
        "in-development",
        None,
    );
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_d.md",
        "01DDDDDDDDDDDDDDDDDDDDDDDD",
        "spec_d",
        "stable",
        Some(&today_str()),
    );
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_e.md",
        "01EEEEEEEEEEEEEEEEEEEEEEEEEE",
        "spec_e",
        "deprecated",
        None,
    );

    specre()
        .args(["status"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Status Summary:")
                .and(predicate::str::contains("draft:          2"))
                .and(predicate::str::contains("in-development: 1"))
                .and(predicate::str::contains("stable:         1"))
                .and(predicate::str::contains("deprecated:     1"))
                .and(predicate::str::contains("total:          5")),
        );
}

// -- Scenario: Stale specres are flagged --

#[test]
fn status_flags_stale_specres() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_specre(
        tmp.path(),
        "docs/specres/auth/user_can_reset_password.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "user_can_reset_password",
        "stable",
        Some(&days_ago_str(45)),
    );

    specre()
        .args(["status"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Stale specres (last_verified > 30 days):")
                .and(predicate::str::contains("user_can_reset_password"))
                .and(predicate::str::contains("(45 days)")),
        );
}

// -- Scenario: No stale specres --

#[test]
fn status_no_stale_section_when_all_fresh() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_specre(
        tmp.path(),
        "docs/specres/cli/fresh_spec.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "fresh_spec",
        "stable",
        Some(&today_str()),
    );

    specre()
        .args(["status"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Status Summary:")
                .and(predicate::str::contains("Stale specres").not()),
        );
}

// -- Scenario: Custom threshold via --threshold --

#[test]
fn status_custom_threshold() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    // 45 days ago — stale at default 30, but not stale at threshold 90
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
        Some(&days_ago_str(45)),
    );

    specre()
        .args(["status", "--threshold", "90"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Status Summary:")
                .and(predicate::str::contains("Stale specres").not()),
        );
}

#[test]
fn status_custom_threshold_shows_in_header() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_specre(
        tmp.path(),
        "docs/specres/cli/spec_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "spec_a",
        "stable",
        Some(&days_ago_str(100)),
    );

    specre()
        .args(["status", "--threshold", "90"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Stale specres (last_verified > 90 days):",
        ));
}

// -- Scenario: Stable specres without last_verified are always flagged --

#[test]
fn status_stable_without_last_verified_is_stale() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_specre(
        tmp.path(),
        "docs/specres/cli/no_date_spec.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "no_date_spec",
        "stable",
        None,
    );

    specre()
        .args(["status"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("no_date_spec")
                .and(predicate::str::contains("(no last_verified)")),
        );
}

// -- Scenario: Invalid last_verified format is flagged as stale --

#[test]
fn status_invalid_last_verified_format() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_specre(
        tmp.path(),
        "docs/specres/cli/bad_date_spec.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "bad_date_spec",
        "stable",
        Some("yesterday"),
    );

    specre()
        .args(["status"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("bad_date_spec")
                .and(predicate::str::contains("(invalid last_verified)")),
        )
        .stderr(
            predicate::str::contains("Warning: invalid last_verified in")
                .and(predicate::str::contains("\"yesterday\"")),
        );
}

// -- Scenario: Impossible date in last_verified is flagged as stale --

#[test]
fn status_impossible_date_in_last_verified() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    write_specre(
        tmp.path(),
        "docs/specres/cli/impossible_date_spec.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "impossible_date_spec",
        "stable",
        Some("2026-02-30"),
    );

    specre()
        .args(["status"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("impossible_date_spec")
                .and(predicate::str::contains("(invalid last_verified)")),
        )
        .stderr(
            predicate::str::contains("Warning: invalid last_verified in")
                .and(predicate::str::contains("\"2026-02-30\"")),
        );
}

// -- Scenario: last_verified on non-stable specres is ignored --

#[test]
fn status_last_verified_on_non_stable_is_ignored() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    // Draft with a stale last_verified — should be counted as draft, not flagged
    write_specre(
        tmp.path(),
        "docs/specres/cli/draft_with_date.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "draft_with_date",
        "draft",
        Some("2025-01-01"),
    );

    specre()
        .args(["status"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("draft:          1")
                .and(predicate::str::contains("Stale specres").not()),
        );
}

// -- Scenario: Empty specre directory --

#[test]
fn status_empty_specre_directory() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres");
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();

    specre()
        .args(["status"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("draft:          0")
                .and(predicate::str::contains("in-development: 0"))
                .and(predicate::str::contains("stable:         0"))
                .and(predicate::str::contains("deprecated:     0"))
                .and(predicate::str::contains("total:          0"))
                .and(predicate::str::contains("Stale specres").not()),
        );
}

// -- Scenario: specre.toml does not exist --

#[test]
fn status_errors_without_config() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["status"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "specre.toml not found. Run 'specre init' first.",
        ));
}
