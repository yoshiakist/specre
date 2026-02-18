---
id: "01KHQKZ5VKTHSD483ZWK0RYPR9"
name: "mcp_tool_status_reports_project_health"
status: "stable"
last_verified: "2026-02-18"
---

## Related Files

- `src/commands/mcp/tools.rs` (tool handler)
- `src/commands/mcp/helpers.rs` (tool logic)
- `src/commands/status.rs` (reused logic)
- `tests/mcp/tool_status.rs` (Test)
- `tests/mcp/helpers.rs` (Test helper)

## Functional Overview

The MCP server exposes a `status` tool that reports specre counts by status and flags stale `last_verified` dates, equivalent to `specre status --json`. AI agents can assess project specification health at a glance.

## Design Intent

Before beginning work, agents need to understand the overall state of specifications — how many are draft, stable, or stale. This tool surfaces that summary over MCP.

## Key Members

- `threshold: u32` (optional, default: `30`) — number of days after which a stable specre's `last_verified` is considered stale

Return value (on success): a `CallToolResult` containing JSON:

```json
{ "summary": { "draft": N, "in_development": N, "stable": N, "deprecated": N, "total": N }, "stale": [...] }
```

## Scenarios

### Report status with mixed specre statuses

1. Agent calls `tools/call` with `name: "status"` and no arguments
2. Server scans specre cards and counts by status
3. Server returns summary counts and any stale entries

### Report status with custom threshold

1. Agent calls `tools/call` with `name: "status"` and `arguments: { "threshold": 7 }`
2. Server uses 7-day threshold for staleness detection
3. Server returns results accordingly

### Empty specre directory

1. No specre cards exist
2. Server returns all-zero summary and empty stale array

## Failures / Exceptions

- If `specre.toml` cannot be loaded, returns `McpError::internal_error`
