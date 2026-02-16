---
id: "01KHJ98T83DPJGMEFH9HAXXAZ1"
name: "mcp_server_starts_via_stdio"
status: "stable"
last_verified: "2026-02-16"
---

## Related Files

- `src/commands/mcp.rs`
- `src/cli.rs`
- `tests/cli_mcp.rs` (Test)
- `tests/common/mcp.rs` (Test helper)

## Functional Overview

`specre mcp` starts an MCP (Model Context Protocol) server using stdio transport. The server communicates via JSON-RPC over stdin/stdout, enabling AI tools (Claude Code, Cursor, VS Code Copilot) to interact with specre capabilities programmatically.

The server declares support for Resources and Tools capabilities in its `initialize` response.

## Design Intent

The MCP server is a thin layer over existing CLI logic. It reuses the same config, parsing, and command functions — it does not reimplement them. Logging goes to stderr (never stdout) because stdout is reserved for JSON-RPC protocol messages.

The server requires `specre.toml` to be present in the working directory, just like all other specre commands.

## Scenarios

### Successful initialization handshake

1. Client starts `specre mcp` as a subprocess with piped stdin/stdout
2. Client sends a JSON-RPC `initialize` request with `protocolVersion` and `clientInfo`
3. Server responds with a JSON-RPC result containing:
   - `protocolVersion`: matching the requested version
   - `capabilities.resources`: present (resources are enabled)
   - `capabilities.tools`: present (tools are enabled)
   - `serverInfo.name`: `"specre"`
4. Client sends an `initialized` notification
5. Server is now ready to handle requests

### Server shuts down when stdin closes

1. Client starts `specre mcp`
2. Client sends `initialize` and receives a response
3. Client closes stdin (EOF)
4. Server exits with code 0

### Missing specre.toml

1. User runs `specre mcp` in a directory without `specre.toml`
2. Server prints an error to stderr and exits with code 1
3. No JSON-RPC communication occurs

## Failures / Exceptions

- If `specre.toml` is missing, the server exits immediately with a config error on stderr — it does not attempt to start the JSON-RPC transport
