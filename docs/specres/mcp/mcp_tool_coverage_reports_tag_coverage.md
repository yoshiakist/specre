---
id: "01KHQKZ6RE7Z3WEDZ54ZKHM6BM"
name: "mcp_tool_coverage_reports_tag_coverage"
status: "stable"
last_verified: "2026-02-18"
---

## Related Files

- `src/commands/mcp/tools.rs` (tool handler)
- `src/commands/coverage.rs` (reused logic: `compute_coverage`)
- `tests/mcp/tool_coverage.rs` (Test)
- `tests/mcp/helpers.rs` (Test helper)

## Functional Overview

The MCP server exposes a `coverage` tool that reports the percentage of source files covered by `@specre` tags, equivalent to `specre coverage --json`.

## Design Intent

Agents need to assess how well a codebase is linked to specifications. This tool provides a quick coverage check over MCP, returning the total file count, tagged count, coverage ratio, and list of uncovered files.

## Key Members

No parameters.

Return value:
```json
{ "total": N, "tagged": N, "coverage": 0.0..1.0, "uncovered": ["<path>", ...] }
```

## Scenarios

### Full coverage

1. All source files have `@specre` tags
2. Agent calls `tools/call` with `name: "coverage"`
3. Server returns `coverage: 1.0` and empty `uncovered` array

### Partial coverage

1. Some files lack `@specre` tags
2. Server returns the ratio and list of uncovered files

### No source files

1. No source files exist in configured directories
2. Server returns `total: 0, tagged: 0, coverage: 0.0, uncovered: []`

## Failures / Exceptions

- If `specre.toml` cannot be loaded, returns `McpError::internal_error`
