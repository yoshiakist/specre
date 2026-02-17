// @specre 01KHAKAYN5WPTDVR99D5Q5TMJE
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::fs;

fn specre() -> assert_cmd::Command {
    cargo_bin_cmd!("specre")
}

/// Helper: create specre.toml with given specre_dir and source_dirs
fn write_config(dir: &std::path::Path, specre_dir: &str, source_dirs: &[&str]) {
    let dirs_toml: Vec<String> = source_dirs.iter().map(|s| format!("\"{s}\"")).collect();
    let content = format!(
        "specre_dir = \"{specre_dir}\"\nsource_dirs = [{}]\n",
        dirs_toml.join(", ")
    );
    fs::write(dir.join("specre.toml"), content).unwrap();
}

/// Helper: create specre.toml with target_extensions
fn write_config_with_ext(
    dir: &std::path::Path,
    specre_dir: &str,
    source_dirs: &[&str],
    target_extensions: &[&str],
) {
    let dirs_toml: Vec<String> = source_dirs.iter().map(|s| format!("\"{s}\"")).collect();
    let ext_toml: Vec<String> = target_extensions.iter().map(|s| format!("\"{s}\"")).collect();
    let content = format!(
        "specre_dir = \"{specre_dir}\"\nsource_dirs = [{}]\ntarget_extensions = [{}]\n",
        dirs_toml.join(", "),
        ext_toml.join(", ")
    );
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

/// Helper: create a source file with @specre markers
fn write_source(dir: &std::path::Path, rel_path: &str, content: &str) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

// -- Scenario: Basic invocation generates index.json --

#[test]
fn index_generates_index_json() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/my_spec.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "my_spec",
        "draft",
        None,
    );

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated docs/specres/index.json"));

    assert!(tmp.path().join("docs/specres/index.json").is_file());
}

// -- Scenario: specres array contains correct entries --

#[test]
fn index_json_contains_specre_entries() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/auth/user_can_sign_up.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "user_can_sign_up",
        "stable",
        Some("2026-01-15"),
    );

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(json["version"], 1);
    assert!(json["generated_at"].as_str().is_some());

    let specres = json["specres"].as_array().unwrap();
    assert_eq!(specres.len(), 1);

    let entry = &specres[0];
    assert_eq!(entry["id"], "01AAAAAAAAAAAAAAAAAAAAAAAA");
    assert_eq!(entry["name"], "user_can_sign_up");
    assert_eq!(entry["status"], "stable");
    assert_eq!(entry["domain"], "auth");
    assert_eq!(entry["path"], "docs/specres/auth/user_can_sign_up.md");
    assert_eq!(entry["last_verified"], "2026-01-15");
}

// -- Scenario: source_refs array contains detected markers --

#[test]
fn index_json_contains_source_refs() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/my_spec.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "my_spec",
        "draft",
        None,
    );
    write_source(
        tmp.path(),
        "src/example.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n",
    );

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let refs = json["source_refs"].as_array().unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0]["specre_id"], "01AAAAAAAAAAAAAAAAAAAAAAAA");
    assert_eq!(refs[0]["file"], "src/example.rs");
    assert_eq!(refs[0]["line"], 1);
}

#[test]
fn index_detects_multiple_markers_in_one_file() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    fs::create_dir_all(tmp.path().join("docs/specres/cli")).unwrap();
    write_source(
        tmp.path(),
        "src/multi.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n// some code\n// @specre 01BBBBBBBBBBBBBBBBBBBBBBBBBB\n",
    );

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let refs = json["source_refs"].as_array().unwrap();
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0]["line"], 1);
    assert_eq!(refs[1]["line"], 3);
}

// -- Scenario: Per-domain INDEX.md is generated --

#[test]
fn index_generates_domain_index_md() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/auth/user_can_sign_up.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "user_can_sign_up",
        "stable",
        Some("2026-01-15"),
    );

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("docs/specres/auth/_INDEX.md"));

    let index_md = fs::read_to_string(tmp.path().join("docs/specres/auth/_INDEX.md")).unwrap();
    assert!(index_md.contains("| Name |"));
    assert!(index_md.contains("user_can_sign_up"));
    assert!(index_md.contains("stable"));
}

// -- Scenario: Subdirectories within a domain are handled correctly --

#[test]
fn index_handles_subdirectories_within_domain() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/auth/signup/user_can_sign_up.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "user_can_sign_up",
        "stable",
        Some("2026-01-15"),
    );
    write_specre(
        tmp.path(),
        "docs/specres/auth/password/user_can_reset_password.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBBBB",
        "user_can_reset_password",
        "draft",
        None,
    );
    write_specre(
        tmp.path(),
        "docs/specres/auth/system_rejects_expired_token.md",
        "01CCCCCCCCCCCCCCCCCCCCCCCCCC",
        "system_rejects_expired_token",
        "in-development",
        None,
    );

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // All three should have domain "auth"
    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let specres = json["specres"].as_array().unwrap();
    assert_eq!(specres.len(), 3);
    for entry in specres {
        assert_eq!(entry["domain"], "auth");
    }

    // _INDEX.md should be at domain level only
    let index_md = fs::read_to_string(tmp.path().join("docs/specres/auth/_INDEX.md")).unwrap();
    // Links should use paths relative to domain directory
    assert!(index_md.contains("signup/user_can_sign_up.md"));
    assert!(index_md.contains("password/user_can_reset_password.md"));
    assert!(index_md.contains("system_rejects_expired_token.md"));

    // No _INDEX.md in subdirectories
    assert!(
        !tmp.path()
            .join("docs/specres/auth/signup/_INDEX.md")
            .exists()
    );
    assert!(
        !tmp.path()
            .join("docs/specres/auth/password/_INDEX.md")
            .exists()
    );
}

// -- Scenario: specre.toml does not exist --

#[test]
fn index_errors_without_config() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "specre.toml not found. Run 'specre init' first.",
        ));
}

// -- Scenario: Empty specre directory --

#[test]
fn index_handles_empty_specre_dir() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    fs::create_dir_all(tmp.path().join("docs/specres")).unwrap();

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json["specres"].as_array().unwrap().len(), 0);
    assert_eq!(json["source_refs"].as_array().unwrap().len(), 0);
}

// -- Scenario: Overwrites existing index files --

#[test]
fn index_overwrites_existing_index() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/my_spec.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "my_spec",
        "draft",
        None,
    );

    // Write stale index.json in specre_dir
    fs::write(tmp.path().join("docs/specres/index.json"), "old content").unwrap();

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    assert_ne!(json_str, "old content");
    // Verify it's valid JSON
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json["version"], 1);
}

// -- Scenario: last_verified is null when not present --

#[test]
fn index_json_last_verified_is_null_when_absent() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/draft_spec.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "draft_spec",
        "draft",
        None,
    );

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let entry = &json["specres"][0];
    assert!(entry["last_verified"].is_null());
}

// -- Scenario: Paths use forward slashes --

#[test]
fn index_json_paths_use_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/auth/signup/user_can_sign_up.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "user_can_sign_up",
        "draft",
        None,
    );
    write_source(
        tmp.path(),
        "src/auth/signup.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n",
    );

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    // No backslashes in paths
    assert!(!json_str.contains('\\'));
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(
        json["specres"][0]["path"],
        "docs/specres/auth/signup/user_can_sign_up.md"
    );
    assert_eq!(json["source_refs"][0]["file"], "src/auth/signup.rs");
}

// -- Scenario: Markers inside string literals are ignored --

#[test]
fn index_ignores_markers_inside_string_literals() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src", "tests"]);
    fs::create_dir_all(tmp.path().join("docs/specres/cli")).unwrap();

    // Real marker (comment) — should be detected
    // Fake markers (inside string literals) — should be ignored
    write_source(
        tmp.path(),
        "tests/cli_test.rs",
        &[
            "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA",
            r#"    write_source(tmp.path(), "src/a.rs", "// @specre 01BBBBBBBBBBBBBBBBBBBBBBBB\n");"#,
            r#"    let s = '// @specre 01CCCCCCCCCCCCCCCCCCCCCCCC';"#,
            "",
        ]
        .join("\n"),
    );

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let refs = json["source_refs"].as_array().unwrap();
    // Only the real marker at line 1 should be detected
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0]["specre_id"], "01AAAAAAAAAAAAAAAAAAAAAAAA");
    assert_eq!(refs[0]["line"], 1);
}

// -- Scenario: target_extensions filters source files --

#[test]
fn index_target_extensions_filters_source_files() {
    let tmp = TempDir::new().unwrap();
    write_config_with_ext(tmp.path(), "docs/specres", &["src"], &["rs"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/my_spec.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "my_spec",
        "draft",
        None,
    );
    // .rs file — should be scanned
    write_source(
        tmp.path(),
        "src/main.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n",
    );
    // .py file — should be skipped
    write_source(
        tmp.path(),
        "src/helper.py",
        "# @specre 01BBBBBBBBBBBBBBBBBBBBBBBB\ndef helper(): pass\n",
    );

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let refs = json["source_refs"].as_array().unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0]["specre_id"], "01AAAAAAAAAAAAAAAAAAAAAAAA");
    assert_eq!(refs[0]["file"], "src/main.rs");
}

#[test]
fn index_empty_target_extensions_scans_nothing() {
    let tmp = TempDir::new().unwrap();
    write_config_with_ext(tmp.path(), "docs/specres", &["src"], &[]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/my_spec.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "my_spec",
        "draft",
        None,
    );
    write_source(
        tmp.path(),
        "src/main.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n",
    );

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let refs = json["source_refs"].as_array().unwrap();
    assert_eq!(refs.len(), 0);
}

#[test]
fn index_without_target_extensions_scans_all_files() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);
    write_specre(
        tmp.path(),
        "docs/specres/cli/my_spec.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "my_spec",
        "draft",
        None,
    );
    // .rs file
    write_source(
        tmp.path(),
        "src/main.rs",
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n",
    );
    // .py file
    write_source(
        tmp.path(),
        "src/helper.py",
        "# @specre 01BBBBBBBBBBBBBBBBBBBBBBBB\ndef helper(): pass\n",
    );

    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let refs = json["source_refs"].as_array().unwrap();
    // Both files should be scanned when no target_extensions
    assert_eq!(refs.len(), 2);
}

// -- Failures / Exceptions: IO errors print warning and skip --

#[test]
fn index_warns_on_unreadable_source_file() {
    let tmp = TempDir::new().unwrap();
    let specre_dir = tmp.path().join("docs/specres/cli");
    fs::create_dir_all(&specre_dir).unwrap();
    fs::write(
        specre_dir.join("test_card.md"),
        "---\nid: \"01AAAAAAAAAAAAAAAAAAAAAAAAA\"\nname: \"test_card\"\nstatus: \"draft\"\n---\n",
    )
    .unwrap();

    // Create a source directory with a subdirectory named like a file
    // (reading a directory as a file causes an IO error)
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("good.rs"), "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAAA\n").unwrap();

    write_config(tmp.path(), "docs/specres", &["src"]);

    // Even with potential IO issues, the command should succeed and produce valid output
    specre()
        .args(["index"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    // The good file should still be indexed
    assert_eq!(json["source_refs"].as_array().unwrap().len(), 1);
}

#[cfg(unix)]
#[test]
fn index_warns_on_permission_denied_specre_file() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().unwrap();
    let specre_dir = tmp.path().join("docs/specres/cli");
    fs::create_dir_all(&specre_dir).unwrap();

    // Create a readable specre card
    fs::write(
        specre_dir.join("good_card.md"),
        "---\nid: \"01AAAAAAAAAAAAAAAAAAAAAAAAA\"\nname: \"good_card\"\nstatus: \"draft\"\n---\n",
    )
    .unwrap();

    // Create an unreadable specre card
    let bad_card = specre_dir.join("bad_card.md");
    fs::write(
        &bad_card,
        "---\nid: \"01BBBBBBBBBBBBBBBBBBBBBBBBB\"\nname: \"bad_card\"\nstatus: \"draft\"\n---\n",
    )
    .unwrap();
    fs::set_permissions(&bad_card, fs::Permissions::from_mode(0o000)).unwrap();

    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    let output = specre()
        .args(["index"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    // Command should succeed (skip the bad file)
    assert!(output.status.success());

    // stderr should contain a warning about the unreadable file
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning: failed to read"),
        "Expected warning about unreadable file in stderr, got: {stderr}"
    );

    // The good card should still be indexed
    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json["specres"].as_array().unwrap().len(), 1);

    // Restore permissions for cleanup
    fs::set_permissions(&bad_card, fs::Permissions::from_mode(0o644)).unwrap();
}

#[cfg(unix)]
#[test]
fn index_warns_on_unreadable_directory() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().unwrap();
    let specre_dir = tmp.path().join("docs/specres");
    let good_domain = specre_dir.join("good");
    let bad_domain = specre_dir.join("bad");
    fs::create_dir_all(&good_domain).unwrap();
    fs::create_dir_all(&bad_domain).unwrap();

    // Create a card in the good domain
    fs::write(
        good_domain.join("card.md"),
        "---\nid: \"01AAAAAAAAAAAAAAAAAAAAAAAAA\"\nname: \"good_card\"\nstatus: \"draft\"\n---\n",
    )
    .unwrap();

    // Create a card in the bad domain, then make the directory unreadable
    fs::write(
        bad_domain.join("card.md"),
        "---\nid: \"01BBBBBBBBBBBBBBBBBBBBBBBBB\"\nname: \"bad_card\"\nstatus: \"draft\"\n---\n",
    )
    .unwrap();
    fs::set_permissions(&bad_domain, fs::Permissions::from_mode(0o000)).unwrap();

    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    write_config(tmp.path(), "docs/specres", &["src"]);

    let output = specre()
        .args(["index"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning: failed to read directory"),
        "Expected warning about unreadable directory in stderr, got: {stderr}"
    );

    // Only the good card should be indexed
    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json["specres"].as_array().unwrap().len(), 1);

    // Restore permissions for cleanup
    fs::set_permissions(&bad_domain, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
#[test]
fn index_warns_on_unreadable_source_file_permission() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().unwrap();
    let specre_dir = tmp.path().join("docs/specres/cli");
    fs::create_dir_all(&specre_dir).unwrap();
    fs::write(
        specre_dir.join("card.md"),
        "---\nid: \"01AAAAAAAAAAAAAAAAAAAAAAAAA\"\nname: \"card\"\nstatus: \"draft\"\n---\n",
    )
    .unwrap();

    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("good.rs"), "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAAA\n").unwrap();

    let bad_file = src_dir.join("bad.rs");
    fs::write(&bad_file, "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAAA\n").unwrap();
    fs::set_permissions(&bad_file, fs::Permissions::from_mode(0o000)).unwrap();

    write_config(tmp.path(), "docs/specres", &["src"]);

    let output = specre()
        .args(["index"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning: failed to read"),
        "Expected warning about unreadable source file in stderr, got: {stderr}"
    );

    // The good source ref should still be indexed
    let json_str = fs::read_to_string(tmp.path().join("docs/specres/index.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json["source_refs"].as_array().unwrap().len(), 1);

    // Restore permissions for cleanup
    fs::set_permissions(&bad_file, fs::Permissions::from_mode(0o644)).unwrap();
}
