---
id: "01KHQKZ6ZHSZX3GR2D7DS23XTE"
name: "mcp_tool_health_check_runs_preflight"
status: "stable"
last_verified: "2026-02-18"
---

## Related Files

- `src/commands/mcp/tools.rs` (tool handler)
- `src/commands/health_check.rs` (reused logic)
- `src/commands/coverage.rs` (reused: `coverage_from_scan`)
- `src/commands/orphans.rs` (reused: `orphans_from_scan`)
- `tests/mcp/tool_health_check.rs` (Test)
- `tests/mcp/helpers.rs` (Test helper)

## Functional Overview

The MCP server exposes a `health-check` tool that runs a comprehensive health check, equivalent to `specre health-check`. It assesses coverage, orphan count, and index freshness against configurable thresholds.

## Design Intent

Before modifying specifications or code, agents should verify project health. This tool provides a single preflight check over MCP, returning a boolean `healthy` flag along with detailed metrics and thresholds.

## Key Members

No parameters.

Return value:
```json
{ "healthy": bool, "coverage": 0.0..1.0, "orphans": N, "index_age_hours": N | null, "thresholds": { "coverage": 0.9, "orphans": 5, "index_age_hours": 24.0 } }
```

## Scenarios

### Healthy project

1. Coverage, orphans, and index age are all within thresholds
2. Agent calls `tools/call` with `name: "health-check"`
3. Server returns `healthy: true` with all metrics

### Unhealthy project — low coverage

1. Coverage is below threshold
2. Server returns `healthy: false`

### Unhealthy project — no index

1. `index.json` does not exist
2. Server returns `healthy: false` with `index_age_hours: null`

## Failures / Exceptions

- If `specre.toml` cannot be loaded, returns `McpError::internal_error`
