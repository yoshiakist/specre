---
id: "01KHQKZ6AAMY6Y6AQB3VDVSF6Z"
name: "mcp_tool_orphans_detects_unlinked_specres"
status: "stable"
last_verified: "2026-02-18"
---

## Related Files

- `src/commands/mcp/tools.rs` (tool handler)
- `src/commands/orphans.rs` (reused logic: `compute_orphans`)
- `tests/mcp/tool_orphans.rs` (Test)
- `tests/mcp/helpers.rs` (Test helper)

## Functional Overview

The MCP server exposes an `orphans` tool that detects unlinked specre cards (no source markers) and dangling markers (markers referencing non-existent specres), equivalent to `specre orphans --json`.

## Design Intent

Agents maintaining traceability need to identify gaps — specre cards without source references and source markers without matching cards. This tool provides that detection over MCP.

## Key Members

No parameters.

Return value (on success):
```json
{ "orphan_specres": ["<path>", ...], "dangling_markers": [{ "file": "<path>", "line": N, "id": "<ULID>" }, ...] }
```

## Scenarios

### No orphans or dangling markers

1. All specre cards have source markers, all markers reference existing cards
2. Agent calls `tools/call` with `name: "orphans"`
3. Server returns empty arrays for both

### Orphan specres detected

1. Some specre cards have no matching source markers
2. Server returns their paths in `orphan_specres`

### Dangling markers detected

1. Some source files reference ULIDs with no matching specre card
2. Server returns details in `dangling_markers`

## Failures / Exceptions

- If `specre.toml` cannot be loaded, returns `McpError::internal_error`
