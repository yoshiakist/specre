// @specre 01KHQKZ6ZHSZX3GR2D7DS23XTE

use assert_fs::prelude::*;
use super::helpers::*;
use serde_json::{Value, json};
use std::io::BufReader;

/// Helper: call the `health-check` tool and return the response.
fn call_health_check(stdin: &mut impl std::io::Write, reader: &mut BufReader<impl std::io::Read>, id: u64) -> Value {
    send(stdin, &json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {
            "name": "health-check",
            "arguments": {}
        }
    }));
    recv(reader)
}

/// Helper: create a fresh index.json with a recent generated_at timestamp.
fn create_fresh_index(dir: &assert_fs::TempDir, specre_dir: &str) {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let index = serde_json::json!({
        "version": 1,
        "generated_at": now,
        "specres": [],
        "source_refs": [],
    });
    dir.child(format!("{specre_dir}/index.json")).write_str(&serde_json::to_string_pretty(&index).unwrap()).unwrap();
}

// ============================================================
// mcp_tool_health_check — Scenario: Healthy project
// ============================================================

#[test]
fn mcp_tool_health_check_healthy() {
    let dir = setup_project("docs/specres");
    create_specre_card(&dir, "docs/specres", "cli/card_a.md", "01AAAAAAAAAAAAAAAAAAAAAAAA", "card_a", "stable");
    dir.child("src").create_dir_all().unwrap();
    dir.child("src/main.rs").write_str("// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n").unwrap();
    create_fresh_index(&dir, "docs/specres");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_health_check(&mut stdin, &mut reader, 2);

    let result = &response["result"];
    assert!(!result["isError"].as_bool().unwrap_or(false), "expected success");

    let payload: Value = serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert!(payload["healthy"].as_bool().unwrap(), "should be healthy");
    assert!(payload["coverage"].as_f64().unwrap() >= 0.9);
    assert!(payload["thresholds"].is_object());

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_health_check — Scenario: Unhealthy — no index
// ============================================================

#[test]
fn mcp_tool_health_check_no_index() {
    let dir = setup_project("docs/specres");
    dir.child("src").create_dir_all().unwrap();
    dir.child("src/main.rs").write_str("// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n").unwrap();
    // No index.json created

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_health_check(&mut stdin, &mut reader, 2);

    let payload: Value = serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert!(!payload["healthy"].as_bool().unwrap(), "should be unhealthy without index");
    assert!(payload["index_age_hours"].is_null());

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_health_check — Scenario: Unhealthy — low coverage
// ============================================================

#[test]
fn mcp_tool_health_check_low_coverage() {
    let dir = setup_project("docs/specres");
    dir.child("src").create_dir_all().unwrap();
    // 1 tagged, 9 untagged → coverage ~10%
    dir.child("src/tagged.rs").write_str("// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n").unwrap();
    for i in 1..=9 {
        dir.child(format!("src/untagged_{i}.rs")).write_str("fn f() {}\n").unwrap();
    }
    create_fresh_index(&dir, "docs/specres");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_health_check(&mut stdin, &mut reader, 2);

    let payload: Value = serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert!(!payload["healthy"].as_bool().unwrap(), "should be unhealthy with low coverage");

    drop(reader);
    shutdown(stdin, child);
}
