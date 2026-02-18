// @specre 01KHK7MFZJZ12XFPQE4RHCBHQN

use super::helpers::*;
use assert_fs::prelude::*;
use serde_json::{Value, json};
use std::io::BufReader;

/// Helper: call the `new` tool and return the response.
fn call_new(
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
                "name": "new",
                "arguments": args
            }
        }),
    );
    recv(reader)
}

// ============================================================
// mcp_tool_new_creates_specre_card — Scenario: Create a new specre card with a name
// ============================================================

#[test]
fn mcp_tool_new_creates_card_with_name() {
    let dir = setup_project("docs/specres");
    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_new(
        &mut stdin,
        &mut reader,
        2,
        &json!({
            "target_dir": "docs/specres/auth",
            "name": "user_can_sign_up"
        }),
    );

    assert_eq!(response["id"], 2);
    let result = &response["result"];
    assert!(
        !result["isError"].as_bool().unwrap_or(false),
        "expected success"
    );

    let content = result["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "text");

    let payload: Value = serde_json::from_str(content[0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["id"].as_str().unwrap().len(), 26);
    assert_eq!(payload["path"], "docs/specres/auth/user_can_sign_up.md");

    // Verify file was actually created
    let file_path = dir.path().join("docs/specres/auth/user_can_sign_up.md");
    assert!(file_path.exists(), "specre card file should exist");
    let file_content = std::fs::read_to_string(&file_path).unwrap();
    assert!(file_content.contains("name: \"user_can_sign_up\""));
    assert!(file_content.contains("status: \"draft\""));

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_new_creates_specre_card — Scenario: Create a new specre card without a name
// ============================================================

#[test]
fn mcp_tool_new_defaults_to_untitled() {
    let dir = setup_project("docs/specres");
    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_new(
        &mut stdin,
        &mut reader,
        2,
        &json!({
            "target_dir": "docs/specres/misc"
        }),
    );

    let result = &response["result"];
    assert!(!result["isError"].as_bool().unwrap_or(false));

    let payload: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["path"], "docs/specres/misc/untitled.md");

    let file_path = dir.path().join("docs/specres/misc/untitled.md");
    assert!(file_path.exists());
    let file_content = std::fs::read_to_string(&file_path).unwrap();
    assert!(file_content.contains("name: \"untitled\""));

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_new_creates_specre_card — Scenario: Target directory does not exist
// ============================================================

#[test]
fn mcp_tool_new_creates_directory_recursively() {
    let dir = setup_project("docs/specres");
    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_new(
        &mut stdin,
        &mut reader,
        2,
        &json!({
            "target_dir": "docs/specres/new_domain/sub",
            "name": "some_behavior"
        }),
    );

    let result = &response["result"];
    assert!(!result["isError"].as_bool().unwrap_or(false));

    let file_path = dir
        .path()
        .join("docs/specres/new_domain/sub/some_behavior.md");
    assert!(
        file_path.exists(),
        "file should be created in nested directory"
    );

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_new_creates_specre_card — Scenario: File already exists
// ============================================================

#[test]
fn mcp_tool_new_errors_when_file_exists() {
    let dir = setup_project("docs/specres");
    dir.child("docs/specres/domain/existing.md")
        .write_str("existing content")
        .unwrap();

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_new(
        &mut stdin,
        &mut reader,
        2,
        &json!({
            "target_dir": "docs/specres/domain",
            "name": "existing"
        }),
    );

    let result = &response["result"];
    assert!(
        result["isError"].as_bool().unwrap_or(false),
        "expected isError: true"
    );
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("already exists"),
        "error message should mention file exists: {text}"
    );

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_new_creates_specre_card — Scenario: Target path is a file, not a directory
// ============================================================

#[test]
fn mcp_tool_new_errors_when_target_is_file() {
    let dir = setup_project("docs/specres");
    dir.child("docs/specres/not_a_dir")
        .write_str("I am a file")
        .unwrap();

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_new(
        &mut stdin,
        &mut reader,
        2,
        &json!({
            "target_dir": "docs/specres/not_a_dir",
            "name": "some_behavior"
        }),
    );

    let result = &response["result"];
    assert!(
        result["isError"].as_bool().unwrap_or(false),
        "expected isError: true"
    );
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("not a directory"),
        "error message should mention not a directory: {text}"
    );

    drop(reader);
    shutdown(stdin, child);
}
