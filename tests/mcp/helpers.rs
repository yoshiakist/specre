// @specre 01KHJ98T83DPJGMEFH9HAXXAZ1
// @specre 01KHJ98TFCDTCARMMX1GC5ZHXE
// @specre 01KHK7MFZJZ12XFPQE4RHCBHQN

use assert_fs::prelude::*;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

/// Spawn `specre mcp` with piped stdin/stdout in a given directory.
pub fn spawn_mcp(dir: &std::path::Path) -> std::process::Child {
    Command::new(assert_cmd::cargo::cargo_bin!("specre"))
        .arg("mcp")
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn specre mcp")
}

/// Send a JSON-RPC message (NDJSON) to the server's stdin.
pub fn send(stdin: &mut impl Write, msg: &Value) {
    serde_json::to_writer(&mut *stdin, msg).unwrap();
    stdin.write_all(b"\n").unwrap();
    stdin.flush().unwrap();
}

/// Read one JSON-RPC response line from the server's stdout.
pub fn recv(reader: &mut BufReader<impl std::io::Read>) -> Value {
    let mut line = String::new();
    reader.read_line(&mut line).expect("failed to read line");
    serde_json::from_str(line.trim()).unwrap_or_else(|_| panic!("invalid JSON: {line}"))
}

/// Perform the MCP initialize handshake, return the result.
pub fn initialize(stdin: &mut impl Write, reader: &mut BufReader<impl std::io::Read>) -> Value {
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
    send(stdin, &json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    }));
    response
}

/// Create a temp dir with specre.toml and specre directory.
pub fn setup_project(specre_dir: &str) -> assert_fs::TempDir {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("specre.toml").write_str(&format!(
        "specre_dir = \"{specre_dir}\"\nsource_dirs = [\"src\"]\n"
    )).unwrap();
    dir.child(specre_dir).create_dir_all().unwrap();
    dir
}

/// Create a specre card file in the given directory.
pub fn create_specre_card(dir: &assert_fs::TempDir, specre_dir: &str, filename: &str, id: &str, name: &str, status: &str) {
    let content = format!(
        "---\nid: \"{id}\"\nname: \"{name}\"\nstatus: \"{status}\"\n---\n\n## Functional Overview\n\nTest card.\n"
    );
    dir.child(format!("{specre_dir}/{filename}")).write_str(&content).unwrap();
}

/// Close stdin and wait for the server to exit.
pub fn shutdown(stdin: std::process::ChildStdin, mut child: std::process::Child) {
    drop(stdin);
    child.wait().expect("failed to wait for child");
}
