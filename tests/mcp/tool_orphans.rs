// @specre 01KHQKZ6AAMY6Y6AQB3VDVSF6Z

use super::helpers::*;
use assert_fs::prelude::*;
use serde_json::{Value, json};
use std::io::BufReader;

/// Helper: call the `orphans` tool and return the response.
fn call_orphans(
    stdin: &mut impl std::io::Write,
    reader: &mut BufReader<impl std::io::Read>,
    id: u64,
) -> Value {
    send(
        stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {
                "name": "orphans",
                "arguments": {}
            }
        }),
    );
    recv(reader)
}

// ============================================================
// mcp_tool_orphans — Scenario: No orphans or dangling markers
// ============================================================

#[test]
fn mcp_tool_orphans_clean() {
    let dir = setup_project("docs/specres");
    // Deprecated cards are excluded from orphan check, so this card won't appear
    create_specre_card(
        &dir,
        "docs/specres",
        "cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "deprecated",
    );

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_orphans(&mut stdin, &mut reader, 2);

    let result = &response["result"];
    assert!(
        !result["isError"].as_bool().unwrap_or(false),
        "expected success"
    );

    let payload: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert!(payload["orphan_specres"].as_array().unwrap().is_empty());
    assert!(payload["dangling_markers"].as_array().unwrap().is_empty());

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_orphans — Scenario: Orphan specres detected
// ============================================================

#[test]
fn mcp_tool_orphans_with_orphan() {
    let dir = setup_project("docs/specres");
    // Non-deprecated card with no source marker → orphan
    create_specre_card(
        &dir,
        "docs/specres",
        "cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_orphans(&mut stdin, &mut reader, 2);

    let payload: Value =
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    let orphans = payload["orphan_specres"].as_array().unwrap();
    assert_eq!(orphans.len(), 1);
    assert!(orphans[0].as_str().unwrap().contains("card_a.md"));

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_orphans — Scenario: Dangling markers detected
// ============================================================

#[test]
fn mcp_tool_orphans_with_dangling() {
    let dir = setup_project("docs/specres");
    // Source file references a ULID that has no specre card
    dir.child("src").create_dir_all().unwrap();
    dir.child("src/main.rs")
        .write_str("// @specre 01ZZZZZZZZZZZZZZZZZZZZZZZZ\nfn main() {}\n")
        .unwrap();

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_orphans(&mut stdin, &mut reader, 2);

    let payload: Value =
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    let dangling = payload["dangling_markers"].as_array().unwrap();
    assert_eq!(dangling.len(), 1);
    assert_eq!(dangling[0]["id"], "01ZZZZZZZZZZZZZZZZZZZZZZZZ");

    drop(reader);
    shutdown(stdin, child);
}
