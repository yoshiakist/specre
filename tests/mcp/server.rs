// @specre 01KHJ98T83DPJGMEFH9HAXXAZ1

use super::helpers::*;
use std::io::BufReader;
use std::process::Command;

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
    assert!(
        result["capabilities"]["resources"].is_object(),
        "resources capability missing"
    );
    assert!(
        result["capabilities"]["tools"].is_object(),
        "tools capability missing"
    );
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
    assert!(
        stderr.contains("specre.toml"),
        "stderr should mention specre.toml: {stderr}"
    );
}
