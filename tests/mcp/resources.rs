// @specre 01KHJ98TFCDTCARMMX1GC5ZHXE

use super::helpers::*;
use serde_json::json;
use std::io::BufReader;

// ============================================================
// mcp_resources_expose_specre_cards — Scenario: List all resources
// ============================================================

#[test]
fn mcp_list_resources_returns_cards() {
    let dir = setup_project("specs");
    create_specre_card(
        &dir,
        "specs",
        "domain/alpha.md",
        "01AAA000000000000000000001",
        "alpha_behavior",
        "stable",
    );
    create_specre_card(
        &dir,
        "specs",
        "domain/beta.md",
        "01BBB000000000000000000002",
        "beta_behavior",
        "draft",
    );

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    send(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "resources/list",
            "params": {}
        }),
    );
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

    send(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "resources/list",
            "params": {}
        }),
    );
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
    create_specre_card(
        &dir,
        "specs",
        "test_card.md",
        "01CCC000000000000000000003",
        "test_card",
        "draft",
    );

    let mut child = spawn_mcp(dir.path());
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    initialize(&mut stdin, &mut reader);

    send(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "resources/read",
            "params": { "uri": "specre:///01CCC000000000000000000003" }
        }),
    );
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

    send(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "resources/read",
            "params": { "uri": "specre:///01NONEXISTENT0000000000000" }
        }),
    );
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

    send(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "resources/read",
            "params": { "uri": "file:///some/path" }
        }),
    );
    let response = recv(&mut reader);

    assert_eq!(response["id"], 5);
    assert!(response["error"].is_object(), "expected error response");
    assert_eq!(response["error"]["code"], -32602);

    drop(reader);
    shutdown(stdin, child);
}
