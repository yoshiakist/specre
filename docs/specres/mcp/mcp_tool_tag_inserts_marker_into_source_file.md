---
id: "01KHQJG96BS5STGSENPNDHEH1H"
name: "mcp_tool_tag_inserts_marker_into_source_file"
status: "stable"
last_verified: "2026-02-18"
---

## Related Files

- `src/commands/mcp/tools.rs` (tool handler)
- `src/commands/mcp/mod.rs` (server infrastructure)
- `src/commands/tag.rs` (reused logic: comment syntax detection, marker insertion)
- `src/ulid.rs` (reused: ULID validation)
- `src/card.rs` (reused: `to_forward_slash`)
- `tests/mcp/tool_tag.rs` (Test)
- `tests/mcp/helpers.rs` (Test helper)

## Functional Overview

The MCP server exposes a `tag` tool that inserts a `@specre <ULID>` marker comment into a source file, equivalent to the `specre tag <ULID> <file>` CLI command. AI agents can establish bidirectional traceability between specre cards and source files programmatically without shelling out to the CLI.

## Design Intent

When an AI agent implements a behavior described by a specre card, it needs to link the source file back to the specre. The `tag` tool enables this within the MCP protocol — the agent calls `tools/call` with `name: "tag"` rather than spawning a subprocess. The tool reuses the same ULID validation, comment syntax detection, and marker insertion logic as the CLI command, ensuring identical behavior regardless of invocation method.

The tool returns a `CallToolResult` with a single text content containing JSON. This structured response includes the ULID, file path, and line number of the inserted (or existing) marker, enabling the agent to confirm the traceability link.

## Key Members

- `ulid: String` (required) — the 26-character ULID to insert as a `@specre` marker
- `file: String` (required) — path to the source file where the marker will be inserted

Return value (on success): a `CallToolResult` containing a single text content with JSON:

```json
{ "id": "<ULID>", "file": "<forward-slash-normalized file path>", "line": <line number> }
```

## Scenarios

### Insert marker into a source file

1. Agent calls `tools/call` with `name: "tag"` and `arguments: { "ulid": "01HZYPMZRK8F9R2DGBGGMM2N8T", "file": "src/example.rs" }`
2. Server validates the ULID format (26 uppercase alphanumeric characters)
3. Server detects the file extension `.rs` and selects `// ` as the comment prefix
4. Server prepends `// @specre 01HZYPMZRK8F9R2DGBGGMM2N8T\n` to the file
5. Server returns a success result containing JSON: `{ "id": "01HZYPMZRK8F9R2DGBGGMM2N8T", "file": "src/example.rs", "line": 1 }`

### Marker already exists in the file

1. The file `src/example.rs` already contains `// @specre 01HZYPMZRK8F9R2DGBGGMM2N8T`
2. Agent calls `tools/call` with the same ULID and file
3. Server does not modify the file
4. Server returns a success result containing JSON with the line number where the existing marker was found

### Invalid ULID format

1. Agent calls `tools/call` with `arguments: { "ulid": "abc123", "file": "src/example.rs" }`
2. Server returns an error result with `isError: true` and a message: `invalid ULID format. Expected 26 uppercase alphanumeric characters.`

### File does not exist

1. Agent calls `tools/call` with a `file` path that does not exist
2. Server returns an error result with `isError: true` and a message: `file not found: <path>`

### Target path is a directory

1. Agent calls `tools/call` with a `file` that points to a directory
2. Server returns an error result with `isError: true` and a message: `'<path>' is a directory, not a file`

### Unsupported file extension

1. Agent calls `tools/call` with a file having an unrecognized extension (e.g., `.xyz`)
2. Server does not modify the file
3. Server returns an error result with `isError: true` and a message: `unsupported file extension '.xyz' — comment syntax is unknown`

### Missing required parameters

1. Agent calls `tools/call` with `name: "tag"` but omits `ulid` or `file`
2. Server returns a JSON-RPC error (invalid params)

## Failures / Exceptions

- If the ULID format is invalid, the tool returns an error result (not a JSON-RPC error) with a descriptive message
- If the file does not exist, the tool returns an error result with the file path in the message
- If the file path is a directory, the tool returns an error result indicating it is not a file
- If the file extension is unsupported, the tool returns an error result with the extension in the message
- If file reading or writing fails due to permissions, the OS error is included in the error message via a JSON-RPC internal error
- Missing required parameters (`ulid`, `file`) result in a JSON-RPC invalid params error from the schema validation layer
