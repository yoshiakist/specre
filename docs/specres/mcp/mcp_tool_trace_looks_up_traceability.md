---
id: "01KHQKZ633JHVDK0WADPPVP3CM"
name: "mcp_tool_trace_looks_up_traceability"
status: "stable"
last_verified: "2026-02-18"
---

## Related Files

- `src/commands/mcp/tools.rs` (tool handler)
- `src/commands/trace.rs` (reused logic)
- `tests/mcp/tool_trace.rs` (Test)
- `tests/mcp/helpers.rs` (Test helper)

## Functional Overview

The MCP server exposes a `trace` tool for bidirectional traceability lookup. Given a ULID, it finds the specre card and all source references. Given a file path, it finds all specre markers in that file and resolves their cards. Equivalent to `specre trace --json`.

## Design Intent

Agents navigating the codebase need to quickly discover the relationship between specifications and source code. This tool provides bidirectional lookup over MCP without subprocess invocation.

## Key Members

- `query: String` (required) — a ULID (26 uppercase alphanumeric chars) or a file path

Return value for ULID query:
```json
{ "specre": "<path>" | null, "source_refs": [{ "file": "<path>", "line": N }, ...] }
```

Return value for file query:
```json
{ "file": "<path>", "specres": [{ "id": "<ULID>", "path": "<path>" | null }, ...] }
```

## Scenarios

### Trace by ULID — specre and source refs found

1. Agent calls `tools/call` with `name: "trace"` and `arguments: { "query": "<ULID>" }`
2. Server finds the specre card with matching id
3. Server scans source files for `@specre <ULID>` markers
4. Server returns the specre path and list of source references

### Trace by ULID — nothing found

1. Agent provides a valid ULID that has no specre card and no source markers
2. Server returns `{ "specre": null, "source_refs": [] }`

### Trace by file path — markers found

1. Agent provides a file path containing `@specre` markers
2. Server extracts ULIDs from the file and resolves specre card paths
3. Server returns the file path and list of specre references

### Trace by file path — no markers

1. Agent provides a file path with no `@specre` markers
2. Server returns the file path and an empty specres array

### File does not exist

1. Agent provides a path to a non-existent file
2. Server returns an error result with `isError: true` and "file not found" message

## Failures / Exceptions

- If the query is a file path and the file does not exist, returns an error result (not JSON-RPC error)
- If `specre.toml` cannot be loaded, returns `McpError::internal_error`
