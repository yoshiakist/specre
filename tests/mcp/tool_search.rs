// @specre 01KHQKZ6H8FB46ESFXB03N85AN

use super::helpers::*;
use serde_json::{Value, json};
use std::io::BufReader;

/// Helper: call the `search` tool and return the response.
fn call_search(stdin: &mut impl std::io::Write, reader: &mut BufReader<impl std::io::Read>, id: u64, args: Value) -> Value {
    send(stdin, &json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {
            "name": "search",
            "arguments": args
        }
    }));
    recv(reader)
}

// ============================================================
// mcp_tool_search — Scenario: Search by keyword
// ============================================================

#[test]
fn mcp_tool_search_by_keyword() {
    let dir = setup_project("docs/specres");
    create_specre_card(&dir, "docs/specres", "cli/card_a.md", "01AAAAAAAAAAAAAAAAAAAAAAAA", "card_a", "stable");
    create_specre_card(&dir, "docs/specres", "auth/card_b.md", "01BBBBBBBBBBBBBBBBBBBBBBBB", "card_b", "draft");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_search(&mut stdin, &mut reader, 2, json!({
        "query": "card_a"
    }));

    let result = &response["result"];
    assert!(!result["isError"].as_bool().unwrap_or(false), "expected success");

    let payload: Value = serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["total"], 1);
    assert!(!payload["truncated"].as_bool().unwrap());
    let results = payload["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "card_a");

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_search — Scenario: Search by status filter
// ============================================================

#[test]
fn mcp_tool_search_by_status() {
    let dir = setup_project("docs/specres");
    create_specre_card(&dir, "docs/specres", "cli/card_a.md", "01AAAAAAAAAAAAAAAAAAAAAAAA", "card_a", "stable");
    create_specre_card(&dir, "docs/specres", "cli/card_b.md", "01BBBBBBBBBBBBBBBBBBBBBBBB", "card_b", "draft");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_search(&mut stdin, &mut reader, 2, json!({
        "status": "draft"
    }));

    let payload: Value = serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["total"], 1);
    let results = payload["results"].as_array().unwrap();
    assert_eq!(results[0]["name"], "card_b");

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_search — Scenario: Search by domain filter
// ============================================================

#[test]
fn mcp_tool_search_by_domain() {
    let dir = setup_project("docs/specres");
    create_specre_card(&dir, "docs/specres", "cli/card_a.md", "01AAAAAAAAAAAAAAAAAAAAAAAA", "card_a", "stable");
    create_specre_card(&dir, "docs/specres", "auth/card_b.md", "01BBBBBBBBBBBBBBBBBBBBBBBB", "card_b", "draft");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_search(&mut stdin, &mut reader, 2, json!({
        "domain": "auth"
    }));

    let payload: Value = serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["total"], 1);
    let results = payload["results"].as_array().unwrap();
    assert_eq!(results[0]["name"], "card_b");

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_search — Scenario: No results
// ============================================================

#[test]
fn mcp_tool_search_no_results() {
    let dir = setup_project("docs/specres");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_search(&mut stdin, &mut reader, 2, json!({
        "query": "nonexistent_keyword"
    }));

    let payload: Value = serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["total"], 0);
    assert!(payload["results"].as_array().unwrap().is_empty());

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_search — Scenario: Search with limit
// ============================================================

#[test]
fn mcp_tool_search_with_limit() {
    let dir = setup_project("docs/specres");
    create_specre_card(&dir, "docs/specres", "cli/card_a.md", "01AAAAAAAAAAAAAAAAAAAAAAAA", "card_a", "stable");
    create_specre_card(&dir, "docs/specres", "cli/card_b.md", "01BBBBBBBBBBBBBBBBBBBBBBBB", "card_b", "draft");
    create_specre_card(&dir, "docs/specres", "cli/card_c.md", "01CCCCCCCCCCCCCCCCCCCCCCCC", "card_c", "stable");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_search(&mut stdin, &mut reader, 2, json!({
        "limit": 1
    }));

    let payload: Value = serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["total"], 3);
    assert_eq!(payload["results"].as_array().unwrap().len(), 1);
    assert!(payload["truncated"].as_bool().unwrap());

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_search — Scenario: Invalid status filter
// ============================================================

#[test]
fn mcp_tool_search_invalid_status() {
    let dir = setup_project("docs/specres");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_search(&mut stdin, &mut reader, 2, json!({
        "status": "invalid_status"
    }));

    let result = &response["result"];
    assert!(result["isError"].as_bool().unwrap_or(false), "expected isError: true");
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("invalid status"), "error should mention invalid status: {text}");

    drop(reader);
    shutdown(stdin, child);
}
