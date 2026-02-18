// @specre 01KHQKZ5M6N304YYJNW8VDKT4W

use super::helpers::*;
use assert_fs::prelude::*;
use serde_json::{Value, json};
use std::io::BufReader;

/// Helper: call the `index` tool and return the response.
fn call_index(
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
                "name": "index",
                "arguments": {}
            }
        }),
    );
    recv(reader)
}

// ============================================================
// mcp_tool_index_regenerates_index — Scenario: Regenerate index with specre cards
// ============================================================

#[test]
fn mcp_tool_index_with_cards() {
    let dir = setup_project("docs/specres");
    create_specre_card(
        &dir,
        "docs/specres",
        "cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );
    create_specre_card(
        &dir,
        "docs/specres",
        "cli/card_b.md",
        "01BBBBBBBBBBBBBBBBBBBBBBBB",
        "card_b",
        "draft",
    );
    // Create a source file with a marker
    dir.child("src").create_dir_all().unwrap();
    dir.child("src/main.rs")
        .write_str("// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n")
        .unwrap();

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_index(&mut stdin, &mut reader, 2);

    assert_eq!(response["id"], 2);
    let result = &response["result"];
    assert!(
        !result["isError"].as_bool().unwrap_or(false),
        "expected success"
    );

    let payload: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["index_file"], "docs/specres/index.json");
    assert_eq!(payload["specre_count"], 2);
    assert_eq!(payload["source_ref_count"], 1);
    assert!(!payload["index_md_files"].as_array().unwrap().is_empty());

    // Verify index.json was actually created
    let index_path = dir.path().join("docs/specres/index.json");
    assert!(index_path.exists(), "index.json should exist");
    let index_content: Value =
        serde_json::from_str(&std::fs::read_to_string(&index_path).unwrap()).unwrap();
    assert_eq!(index_content["version"], 1);
    assert_eq!(index_content["specres"].as_array().unwrap().len(), 2);

    // Verify _INDEX.md was created
    let index_md_path = dir.path().join("docs/specres/cli/_INDEX.md");
    assert!(index_md_path.exists(), "_INDEX.md should exist");

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_index_regenerates_index — Scenario: Empty specre directory
// ============================================================

#[test]
fn mcp_tool_index_empty() {
    let dir = setup_project("docs/specres");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_index(&mut stdin, &mut reader, 2);

    let result = &response["result"];
    assert!(!result["isError"].as_bool().unwrap_or(false));

    let payload: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["specre_count"], 0);
    assert_eq!(payload["source_ref_count"], 0);

    drop(reader);
    shutdown(stdin, child);
}
