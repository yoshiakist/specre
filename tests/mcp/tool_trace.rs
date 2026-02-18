// @specre 01KHQKZ633JHVDK0WADPPVP3CM

use super::helpers::*;
use assert_fs::prelude::*;
use serde_json::{Value, json};
use std::io::BufReader;

/// Helper: call the `trace` tool and return the response.
fn call_trace(
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
                "name": "trace",
                "arguments": args
            }
        }),
    );
    recv(reader)
}

// ============================================================
// mcp_tool_trace — Scenario: Trace by ULID — specre and source refs found
// ============================================================

#[test]
fn mcp_tool_trace_by_ulid_found() {
    let dir = setup_project("docs/specres");
    create_specre_card(
        &dir,
        "docs/specres",
        "cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );
    dir.child("src").create_dir_all().unwrap();
    dir.child("src/main.rs")
        .write_str("// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n")
        .unwrap();

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_trace(
        &mut stdin,
        &mut reader,
        2,
        &json!({
            "query": "01AAAAAAAAAAAAAAAAAAAAAAAA"
        }),
    );

    let result = &response["result"];
    assert!(
        !result["isError"].as_bool().unwrap_or(false),
        "expected success"
    );

    let payload: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert!(payload["specre"].as_str().unwrap().contains("card_a.md"));
    let refs = payload["source_refs"].as_array().unwrap();
    assert_eq!(refs.len(), 1);
    assert!(refs[0]["file"].as_str().unwrap().contains("main.rs"));
    assert_eq!(refs[0]["line"], 1);

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_trace — Scenario: Trace by ULID — nothing found
// ============================================================

#[test]
fn mcp_tool_trace_by_ulid_not_found() {
    let dir = setup_project("docs/specres");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_trace(
        &mut stdin,
        &mut reader,
        2,
        &json!({
            "query": "01ZZZZZZZZZZZZZZZZZZZZZZZZ"
        }),
    );

    let result = &response["result"];
    assert!(!result["isError"].as_bool().unwrap_or(false));

    let payload: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert!(payload["specre"].is_null());
    assert!(payload["source_refs"].as_array().unwrap().is_empty());

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_trace — Scenario: Trace by file — markers found
// ============================================================

#[test]
fn mcp_tool_trace_by_file_found() {
    let dir = setup_project("docs/specres");
    create_specre_card(
        &dir,
        "docs/specres",
        "cli/card_a.md",
        "01AAAAAAAAAAAAAAAAAAAAAAAA",
        "card_a",
        "stable",
    );
    dir.child("src").create_dir_all().unwrap();
    dir.child("src/main.rs")
        .write_str("// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n")
        .unwrap();

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_trace(
        &mut stdin,
        &mut reader,
        2,
        &json!({
            "query": "src/main.rs"
        }),
    );

    let result = &response["result"];
    assert!(!result["isError"].as_bool().unwrap_or(false));

    let payload: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert!(payload["file"].as_str().unwrap().contains("main.rs"));
    let specres = payload["specres"].as_array().unwrap();
    assert_eq!(specres.len(), 1);
    assert_eq!(specres[0]["id"], "01AAAAAAAAAAAAAAAAAAAAAAAA");
    assert!(specres[0]["path"].as_str().unwrap().contains("card_a.md"));

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_trace — Scenario: File does not exist
// ============================================================

#[test]
fn mcp_tool_trace_file_not_found() {
    let dir = setup_project("docs/specres");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_trace(
        &mut stdin,
        &mut reader,
        2,
        &json!({
            "query": "src/nonexistent.rs"
        }),
    );

    let result = &response["result"];
    assert!(
        result["isError"].as_bool().unwrap_or(false),
        "expected isError: true"
    );
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("file not found"),
        "error should mention file not found: {text}"
    );

    drop(reader);
    shutdown(stdin, child);
}
