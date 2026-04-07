---
id: "01KHQKZ5M6N304YYJNW8VDKT4W"
name: "mcp_tool_index_regenerates_index"
status: "stable"
last_verified: "2026-04-07"
---

## Related Files

- `src/commands/mcp/tools.rs` (tool handler)
- `src/commands/mcp/helpers.rs` (tool logic)
- `src/commands/index.rs` (reused logic: index generation)
- `tests/mcp/tool_index.rs` (Test)
- `tests/mcp/helpers.rs` (Test helper)

## Functional Overview

The MCP server exposes an `index` tool that regenerates `index.json` and per-domain `_INDEX.md` files, equivalent to `specre index --json`. AI agents can refresh the project index without shelling out to the CLI.

## Design Intent

After creating or modifying specre cards, agents need to regenerate the index to keep `index.json` and `_INDEX.md` files up to date. This tool provides that capability over MCP.

## Key Members

No parameters.

Return value (on success): a `CallToolResult` containing a single text content with JSON:

```json
{ "index_file": "<path>", "specre_count": N, "source_ref_count": N, "index_md_files": ["<path>", ...] }
```

## Scenarios

### Regenerate index with specre cards present

1. Agent calls `tools/call` with `name: "index"` and no arguments
2. Server scans specre cards and source references
3. Server writes `index.json` and per-domain `_INDEX.md`
4. Server returns a success result with counts and file paths

### Regenerate index with empty specre directory

1. Specre directory exists but contains no cards
2. Agent calls `tools/call` with `name: "index"`
3. Server writes `index.json` with zero specres and zero source refs
4. Server returns success with `specre_count: 0`

## Failures / Exceptions

- If `specre.toml` cannot be loaded, returns `McpError::internal_error`
- If file writing fails, returns `McpError::internal_error` with the OS error
