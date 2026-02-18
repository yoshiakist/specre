// @specre 01KHQJG96BS5STGSENPNDHEH1H

use assert_fs::prelude::*;
use super::helpers::*;
use serde_json::{Value, json};
use std::io::BufReader;

/// Helper: call the `tag` tool and return the response.
fn call_tag(stdin: &mut impl std::io::Write, reader: &mut BufReader<impl std::io::Read>, id: u64, args: Value) -> Value {
    send(stdin, &json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {
            "name": "tag",
            "arguments": args
        }
    }));
    recv(reader)
}

// ============================================================
// mcp_tool_tag_inserts_marker_into_source_file — Scenario: Insert marker into a source file
// ============================================================

#[test]
fn mcp_tool_tag_inserts_marker() {
    let dir = setup_project("docs/specres");
    dir.child("src").create_dir_all().unwrap();
    dir.child("src/example.rs").write_str("fn main() {}\n").unwrap();

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_tag(&mut stdin, &mut reader, 2, json!({
        "ulid": "01HZYPMZRK8F9R2DGBGGMM2N8T",
        "file": "src/example.rs"
    }));

    assert_eq!(response["id"], 2);
    let result = &response["result"];
    assert!(!result["isError"].as_bool().unwrap_or(false), "expected success");

    let content = result["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "text");

    let payload: Value = serde_json::from_str(content[0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["id"], "01HZYPMZRK8F9R2DGBGGMM2N8T");
    assert_eq!(payload["file"], "src/example.rs");
    assert_eq!(payload["line"], 1);

    // Verify file was actually modified
    let file_content = std::fs::read_to_string(dir.path().join("src/example.rs")).unwrap();
    assert!(file_content.starts_with("// @specre 01HZYPMZRK8F9R2DGBGGMM2N8T\n"));
    assert!(file_content.contains("fn main() {}"));

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_tag_inserts_marker_into_source_file — Scenario: Marker already exists in the file
// ============================================================

#[test]
fn mcp_tool_tag_returns_existing_marker() {
    let dir = setup_project("docs/specres");
    dir.child("src").create_dir_all().unwrap();
    dir.child("src/example.rs").write_str(
        "// @specre 01HZYPMZRK8F9R2DGBGGMM2N8T\nfn main() {}\n"
    ).unwrap();

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_tag(&mut stdin, &mut reader, 2, json!({
        "ulid": "01HZYPMZRK8F9R2DGBGGMM2N8T",
        "file": "src/example.rs"
    }));

    let result = &response["result"];
    assert!(!result["isError"].as_bool().unwrap_or(false), "expected success (idempotent)");

    let payload: Value = serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["id"], "01HZYPMZRK8F9R2DGBGGMM2N8T");
    assert_eq!(payload["line"], 1);

    // Verify file was NOT modified (still has the same content)
    let file_content = std::fs::read_to_string(dir.path().join("src/example.rs")).unwrap();
    assert_eq!(file_content, "// @specre 01HZYPMZRK8F9R2DGBGGMM2N8T\nfn main() {}\n");

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_tag_inserts_marker_into_source_file — Scenario: Invalid ULID format
// ============================================================

#[test]
fn mcp_tool_tag_errors_on_invalid_ulid() {
    let dir = setup_project("docs/specres");
    dir.child("src").create_dir_all().unwrap();
    dir.child("src/example.rs").write_str("fn main() {}\n").unwrap();

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_tag(&mut stdin, &mut reader, 2, json!({
        "ulid": "abc123",
        "file": "src/example.rs"
    }));

    let result = &response["result"];
    assert!(result["isError"].as_bool().unwrap_or(false), "expected isError: true");
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("invalid ULID format"), "error message should mention invalid ULID: {text}");

    // Verify file was NOT modified
    let file_content = std::fs::read_to_string(dir.path().join("src/example.rs")).unwrap();
    assert_eq!(file_content, "fn main() {}\n");

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_tag_inserts_marker_into_source_file — Scenario: File does not exist
// ============================================================

#[test]
fn mcp_tool_tag_errors_on_missing_file() {
    let dir = setup_project("docs/specres");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_tag(&mut stdin, &mut reader, 2, json!({
        "ulid": "01HZYPMZRK8F9R2DGBGGMM2N8T",
        "file": "src/nonexistent.rs"
    }));

    let result = &response["result"];
    assert!(result["isError"].as_bool().unwrap_or(false), "expected isError: true");
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("file not found"), "error message should mention file not found: {text}");

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_tag_inserts_marker_into_source_file — Scenario: Target path is a directory
// ============================================================

#[test]
fn mcp_tool_tag_errors_on_directory() {
    let dir = setup_project("docs/specres");
    dir.child("src").create_dir_all().unwrap();

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_tag(&mut stdin, &mut reader, 2, json!({
        "ulid": "01HZYPMZRK8F9R2DGBGGMM2N8T",
        "file": "src"
    }));

    let result = &response["result"];
    assert!(result["isError"].as_bool().unwrap_or(false), "expected isError: true");
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("is a directory"), "error message should mention directory: {text}");

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_tool_tag_inserts_marker_into_source_file — Scenario: Unsupported file extension
// ============================================================

#[test]
fn mcp_tool_tag_errors_on_unsupported_extension() {
    let dir = setup_project("docs/specres");
    dir.child("data").create_dir_all().unwrap();
    dir.child("data/config.xyz").write_str("some data\n").unwrap();

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    let response = call_tag(&mut stdin, &mut reader, 2, json!({
        "ulid": "01HZYPMZRK8F9R2DGBGGMM2N8T",
        "file": "data/config.xyz"
    }));

    let result = &response["result"];
    assert!(result["isError"].as_bool().unwrap_or(false), "expected isError: true");
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("unsupported file extension"), "error message should mention unsupported: {text}");
    assert!(text.contains(".xyz"), "error message should mention the extension: {text}");

    // Verify file was NOT modified
    let file_content = std::fs::read_to_string(dir.path().join("data/config.xyz")).unwrap();
    assert_eq!(file_content, "some data\n");

    drop(reader);
    shutdown(stdin, child);
}
