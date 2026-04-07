---
id: "01KHK7MFZJZ12XFPQE4RHCBHQN"
name: "mcp_tool_new_creates_specre_card"
status: "stable"
last_verified: "2026-04-07"
---

## Related Files

- `src/commands/mcp/tools.rs` (tool handler)
- `src/commands/mcp/helpers.rs` (tool logic)
- `src/commands/mcp/mod.rs` (server infrastructure)
- `src/commands/new.rs` (reused logic: directory creation, file writing)
- `src/template.rs` (reused: template rendering)
- `src/ulid.rs` (reused: ULID generation)
- `src/config.rs` (reused: language config)
- `tests/mcp/tool_new.rs` (Test)
- `tests/mcp/helpers.rs` (Test helper)

## Functional Overview

The MCP server exposes a `new` tool that creates a new specre card, equivalent to the `specre new <dir> --name <name>` CLI command. AI agents can scaffold specre cards programmatically without shelling out to the CLI.

## Design Intent

This tool is the primary write operation for agents in the SDD workflow. When an agent identifies a new behavior to specify, it calls `tools/call` with `name: "new"` instead of running a subprocess. The tool reuses the same ULID generation, template rendering, and file-writing logic as the CLI command, ensuring identical output regardless of invocation method.

The tool returns a `CallToolResult` with a single text content containing JSON. This structured JSON includes the generated ULID and file path, enabling the agent to immediately reference the new card by ID or read it back via `resources/read`.

## Key Members

- `target_dir: String` (required) — directory where the specre file will be created (e.g., `docs/specres/auth`)
- `name: String` (optional, default: `"untitled"`) — specre name describing the behavior

Return value (on success): a `CallToolResult` containing a single text content with JSON:

```json
{ "id": "<ULID>", "path": "<forward-slash-normalized file path>" }
```

## Scenarios

### Create a new specre card with a name

1. Agent calls `tools/call` with `name: "new"` and `arguments: { "target_dir": "docs/specres/auth", "name": "user_can_sign_up" }`
2. Server generates a new ULID
3. Server creates the directory `docs/specres/auth/` if it does not exist
4. Server writes `docs/specres/auth/user_can_sign_up.md` using the specre template
5. Server returns a success result containing JSON: `{ "id": "<ULID>", "path": "docs/specres/auth/user_can_sign_up.md" }`

### Create a new specre card without a name

1. Agent calls `tools/call` with `name: "new"` and `arguments: { "target_dir": "docs/specres/misc" }`
2. Server uses the default name `"untitled"`
3. Server writes `docs/specres/misc/untitled.md`
4. Server returns success with the generated ULID and path

### Target directory does not exist

1. Agent calls `tools/call` with a `target_dir` that does not exist yet
2. Server creates the directory recursively
3. File is created successfully
4. Server returns success

### File already exists

1. A file with the same name already exists at the target path
2. Agent calls `tools/call` with the same `target_dir` and `name`
3. Server returns an error result with `isError: true` and a message indicating the file already exists

### Target path is a file, not a directory

1. `target_dir` points to an existing file rather than a directory
2. Server returns an error result with `isError: true` and a message indicating the path is a file

### Missing target_dir parameter

1. Agent calls `tools/call` with `name: "new"` but omits `target_dir`
2. Server returns a JSON-RPC error (invalid params)

## Failures / Exceptions

- If `target_dir` points to a file (not a directory), the tool returns an error result (not a JSON-RPC error) with a descriptive message
- If the target file already exists, the tool returns an error result to prevent accidental overwrites
- If directory creation or file writing fails due to permissions, the OS error is included in the error message
- Missing required parameters (`target_dir`) result in a JSON-RPC invalid params error from the schema validation layer
