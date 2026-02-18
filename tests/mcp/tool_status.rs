// @specre 01KHQKZ5VKTHSD483ZWK0RYPR9

use super::helpers::*;
use assert_fs::prelude::*;
use serde_json::{Value, json};
use std::io::BufReader;

/// Helper: call the `status` tool and return the response.
fn call_status(
    stdin: &mut impl std::io::Write,
    reader: &mut BufReader<impl std::io::Read>,
    id: u64,
    args: &Value,
) -> Value {
    send(
        stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {
                "name": "status",
                "arguments": args
            }
        }),
    );
    recv(reader)
}

// ============================================================
// mcp_tool_status_reports_project_health — Scenario: Mixed statuses
// ============================================================

#[test]
fn mcp_tool_status_mixed() {
    let dir = setup_project("docs/specres");
    create_specre_card(
        &dir,
        "docs/specres",
        "cli/a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "draft",
    );
    create_specre_card(
        &dir,
        "docs/specres",
        "cli/b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_b",
        "stable",
    );
    create_specre_card(
        &dir,
        "docs/specres",
        "cli/c.md",
        "01CCCCCCCCCCCCCCCCCCCCCCCC",
        "card_c",
        "deprecated",
    );

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_status(&mut stdin, &mut reader, 2, &json!({}));

    assert_eq!(response["id"], 2);
    let result = &response["result"];
    assert!(
        !result["isError"].as_bool().unwrap_or(false),
        "expected success"
    );

    let payload: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    let summary = &payload["summary"];
    assert_eq!(summary["draft"], 1);
    assert_eq!(summary["stable"], 1);
    assert_eq!(summary["deprecated"], 1);
    assert_eq!(summary["total"], 3);

    // The stable card has no last_verified, so it should be stale
    let stale = payload["stale"].as_array().unwrap();
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0]["name"], "card_b");

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_status_reports_project_health — Scenario: Custom threshold
// ============================================================

#[test]
fn mcp_tool_status_custom_threshold() {
    let dir = setup_project("docs/specres");
    // Card with last_verified set to a recent date — should not be stale with default threshold
    let content = "---\nid: \"01AAAAAAAAAAAAAAAAAAAAAAAA\"\nname: \"recent_card\"\nstatus: \"stable\"\nlast_verified: \"2026-02-17\"\n---\n\n## Functional Overview\n\nTest.\n";
    dir.child("docs/specres/cli/recent.md")
        .write_str(content)
        .unwrap();

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    // With default threshold (30 days), card is not stale
    let response = call_status(&mut stdin, &mut reader, 2, &json!({}));
    let payload: Value =
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert!(
        payload["stale"].as_array().unwrap().is_empty(),
        "should not be stale with default threshold"
    );

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_status_reports_project_health — Scenario: Empty specre directory
// ============================================================

#[test]
fn mcp_tool_status_empty() {
    let dir = setup_project("docs/specres");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_status(&mut stdin, &mut reader, 2, &json!({}));

    let payload: Value =
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["summary"]["total"], 0);
    assert!(payload["stale"].as_array().unwrap().is_empty());

    drop(reader);
    shutdown(stdin, child);
}
