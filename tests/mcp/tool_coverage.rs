// @specre 01KHQKZ6RE7Z3WEDZ54ZKHM6BM

use assert_fs::prelude::*;
use super::helpers::*;
use serde_json::{Value, json};
use std::io::BufReader;

/// Helper: call the `coverage` tool and return the response.
fn call_coverage(stdin: &mut impl std::io::Write, reader: &mut BufReader<impl std::io::Read>, id: u64) -> Value {
    send(stdin, &json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {
            "name": "coverage",
            "arguments": {}
        }
    }));
    recv(reader)
}

// ============================================================
// mcp_tool_coverage — Scenario: Full coverage
// ============================================================

#[test]
fn mcp_tool_coverage_full() {
    let dir = setup_project("docs/specres");
    dir.child("src").create_dir_all().unwrap();
    dir.child("src/main.rs").write_str("// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n").unwrap();

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_coverage(&mut stdin, &mut reader, 2);

    let result = &response["result"];
    assert!(!result["isError"].as_bool().unwrap_or(false), "expected success");

    let payload: Value = serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["total"], 1);
    assert_eq!(payload["tagged"], 1);
    assert_eq!(payload["coverage"], 1.0);
    assert!(payload["uncovered"].as_array().unwrap().is_empty());

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_coverage — Scenario: Partial coverage
// ============================================================

#[test]
fn mcp_tool_coverage_partial() {
    let dir = setup_project("docs/specres");
    dir.child("src").create_dir_all().unwrap();
    dir.child("src/tagged.rs").write_str("// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn a() {}\n").unwrap();
    dir.child("src/untagged.rs").write_str("fn b() {}\n").unwrap();

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_coverage(&mut stdin, &mut reader, 2);

    let payload: Value = serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["total"], 2);
    assert_eq!(payload["tagged"], 1);
    assert_eq!(payload["coverage"], 0.5);
    let uncovered = payload["uncovered"].as_array().unwrap();
    assert_eq!(uncovered.len(), 1);

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_coverage — Scenario: No source files
// ============================================================

#[test]
fn mcp_tool_coverage_empty() {
    let dir = setup_project("docs/specres");
    // No src directory created — source_dirs points to non-existent "src"

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_coverage(&mut stdin, &mut reader, 2);

    let payload: Value = serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["total"], 0);
    assert_eq!(payload["tagged"], 0);
    assert_eq!(payload["coverage"], 0.0);

    drop(reader);
    shutdown(stdin, child);
}
