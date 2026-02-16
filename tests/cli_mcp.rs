// @specre 01KHJ98T83DPJGMEFH9HAXXAZ1
// @specre 01KHJ98TFCDTCARMMX1GC5ZHXE

use assert_fs::prelude::*;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

/// Helper: spawn `specre mcp` with piped stdin/stdout in a given directory.
fn spawn_mcp(dir: &std::path::Path) -> std::process::Child {
    Command::new(assert_cmd::cargo::cargo_bin!("specre"))
        .arg("mcp")
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn specre mcp")
}

/// Helper: send a JSON-RPC message (NDJSON) to the server's stdin.
fn send(stdin: &mut impl Write, msg: &Value) {
    serde_json::to_writer(&mut *stdin, msg).unwrap();
    stdin.write_all(b"\n").unwrap();
    stdin.flush().unwrap();
}

/// Helper: read one JSON-RPC response line from the server's stdout.
fn recv(reader: &mut BufReader<impl std::io::Read>) -> Value {
    let mut line = String::new();
    reader.read_line(&mut line).expect("failed to read line");
    serde_json::from_str(line.trim()).unwrap_or_else(|_| panic!("invalid JSON: {line}"))
}

/// Helper: perform the MCP initialize handshake, return the result.
fn initialize(stdin: &mut impl Write, reader: &mut BufReader<impl std::io::Read>) -> Value {
    send(stdin, &json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "1.0.0" }
        }
    }));
    let response = recv(reader);
    // Send initialized notification
    send(stdin, &json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    }));
    response
}

/// Helper: create a temp dir with specre.toml and specre directory.
fn setup_project(specre_dir: &str) -> assert_fs::TempDir {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("specre.toml").write_str(&format!(
        "specre_dir = \"{specre_dir}\"\nsource_dirs = [\"src\"]\n"
    )).unwrap();
    dir.child(specre_dir).create_dir_all().unwrap();
    dir
}

fn create_specre_card(dir: &assert_fs::TempDir, specre_dir: &str, filename: &str, id: &str, name: &str, status: &str) {
    let content = format!(
        "---\nid: \"{id}\"\nname: \"{name}\"\nstatus: \"{status}\"\n---\n\n## Functional Overview\n\nTest card.\n"
    );
    dir.child(format!("{specre_dir}/{filename}")).write_str(&content).unwrap();
}

/// Helper: close stdin and wait for the server to exit.
fn shutdown(stdin: std::process::ChildStdin, mut child: std::process::Child) {
    drop(stdin);
    child.wait().expect("failed to wait for child");
}

// ============================================================
// mcp_server_starts_via_stdio — Scenario: Successful initialization handshake
// ============================================================

#[test]
fn mcp_initialize_returns_capabilities() {
    let dir = setup_project("docs/specres");
    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());

    let response = initialize(&mut stdin, &mut reader);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    let result = &response["result"];
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert!(result["capabilities"]["resources"].is_object(), "resources capability missing");
    assert!(result["capabilities"]["tools"].is_object(), "tools capability missing");
    assert_eq!(result["serverInfo"]["name"], "specre");

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_server_starts_via_stdio — Scenario: Server shuts down when stdin closes
// ============================================================

#[test]
fn mcp_exits_cleanly_on_stdin_close() {
    let dir = setup_project("docs/specres");
    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());

    initialize(&mut stdin, &mut reader);
    drop(reader);
    drop(stdin);

    let status = child.wait().unwrap();
    assert!(status.success());
}

// ============================================================
// mcp_server_starts_via_stdio — Scenario: Missing specre.toml
// ============================================================

#[test]
fn mcp_errors_without_config() {
    let dir = assert_fs::TempDir::new().unwrap();
    let output = Command::new(assert_cmd::cargo::cargo_bin!("specre"))
        .arg("mcp")
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("specre.toml"), "stderr should mention specre.toml: {stderr}");
}

// ============================================================
// mcp_resources_expose_specre_cards — Scenario: List all resources
// ============================================================

#[test]
fn mcp_list_resources_returns_cards() {
    let dir = setup_project("specs");
    create_specre_card(&dir, "specs", "domain/alpha.md", "01AAA000000000000000000001", "alpha_behavior", "stable");
    create_specre_card(&dir, "specs", "domain/beta.md", "01BBB000000000000000000002", "beta_behavior", "draft");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    send(&mut stdin, &json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "resources/list",
        "params": {}
    }));
    let response = recv(&mut reader);

    assert_eq!(response["id"], 2);
    let resources = response["result"]["resources"].as_array().unwrap();
    assert_eq!(resources.len(), 2);

    // Sorted by ULID
    assert_eq!(resources[0]["uri"], "specre:///01AAA000000000000000000001");
    assert_eq!(resources[0]["name"], "alpha_behavior");
    assert_eq!(resources[0]["description"], "[stable] alpha_behavior");
    assert_eq!(resources[0]["mimeType"], "text/markdown");

    assert_eq!(resources[1]["uri"], "specre:///01BBB000000000000000000002");
    assert_eq!(resources[1]["name"], "beta_behavior");
    assert_eq!(resources[1]["description"], "[draft] beta_behavior");

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_resources_expose_specre_cards — Scenario: List resources with empty specre directory
// ============================================================

#[test]
fn mcp_list_resources_empty() {
    let dir = setup_project("docs/specres");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    send(&mut stdin, &json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "resources/list",
        "params": {}
    }));
    let response = recv(&mut reader);
    let resources = response["result"]["resources"].as_array().unwrap();
    assert!(resources.is_empty());

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_resources_expose_specre_cards — Scenario: Read a specific resource
// ============================================================

#[test]
fn mcp_read_resource_returns_card_content() {
    let dir = setup_project("specs");
    create_specre_card(&dir, "specs", "test_card.md", "01CCC000000000000000000003", "test_card", "draft");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    send(&mut stdin, &json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "resources/read",
        "params": { "uri": "specre:///01CCC000000000000000000003" }
    }));
    let response = recv(&mut reader);

    assert_eq!(response["id"], 3);
    let contents = response["result"]["contents"].as_array().unwrap();
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0]["uri"], "specre:///01CCC000000000000000000003");
    let text = contents[0]["text"].as_str().unwrap();
    assert!(text.contains("id: \"01CCC000000000000000000003\""));
    assert!(text.contains("name: \"test_card\""));
    assert!(text.contains("## Functional Overview"));

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_resources_expose_specre_cards — Scenario: Read a nonexistent resource
// ============================================================

#[test]
fn mcp_read_resource_not_found() {
    let dir = setup_project("specs");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    send(&mut stdin, &json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "resources/read",
        "params": { "uri": "specre:///01NONEXISTENT0000000000000" }
    }));
    let response = recv(&mut reader);

    assert_eq!(response["id"], 4);
    assert!(response["error"].is_object(), "expected error response");
    assert_eq!(response["error"]["code"], -32002);

    drop(reader);
    shutdown(stdin, child);
}

// ============================================================
// mcp_resources_expose_specre_cards — Scenario: Read with invalid URI prefix
// ============================================================

#[test]
fn mcp_read_resource_invalid_uri() {
    let dir = setup_project("specs");

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    send(&mut stdin, &json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "resources/read",
        "params": { "uri": "file:///some/path" }
    }));
    let response = recv(&mut reader);

    assert_eq!(response["id"], 5);
    assert!(response["error"].is_object(), "expected error response");
    assert_eq!(response["error"]["code"], -32602);

    drop(reader);
    shutdown(stdin, child);
}
