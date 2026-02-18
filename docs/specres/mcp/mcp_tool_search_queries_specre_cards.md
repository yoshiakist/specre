---
id: "01KHQKZ6H8FB46ESFXB03N85AN"
name: "mcp_tool_search_queries_specre_cards"
status: "stable"
last_verified: "2026-02-18"
---

## Related Files

- `src/commands/mcp/tools.rs` (tool handler)
- `src/commands/mcp/search.rs` (search logic)
- `src/commands/mcp/helpers.rs` (tool logic)
- `src/commands/search/mod.rs` (reused logic)
- `tests/mcp/tool_search.rs` (Test)
- `tests/mcp/helpers.rs` (Test helper)

## Functional Overview

The MCP server exposes a `search` tool for full-text and filtered search across specre cards, equivalent to `specre search`. Multi-keyword queries use AND logic by default, with an `or` flag for OR logic. Supports filtering by status, domain, and date ranges.

## Design Intent

Agents exploring a project's specifications need to discover relevant cards by keyword, status, or domain. This tool provides that search capability over MCP, enabling agents to find specifications before reading or modifying them.

## Key Members

- `query: String` (optional) — free-text substring to match (case-insensitive, multi-keyword AND by default)
- `status: String` (optional) — filter by status (draft, in-development, stable, deprecated)
- `domain: String` (optional) — filter by domain
- `or: bool` (optional, default: false) — use OR logic for multi-keyword queries
- `limit: usize` (optional) — return at most N results

Return value:
```json
{ "results": [{ "id": "<ULID>", "name": "...", "status": "...", "domain": "...", "path": "...", "last_verified": "..." | null, "excerpt": "..." | null }], "total": N, "truncated": false }
```

## Scenarios

### Search by keyword — matches found

1. Agent calls `tools/call` with `name: "search"` and `arguments: { "query": "auth" }`
2. Server scans cards and returns matching results

### Search by status filter

1. Agent calls with `arguments: { "status": "draft" }`
2. Server returns only draft cards

### Search by domain filter

1. Agent calls with `arguments: { "domain": "cli" }`
2. Server returns only cards in the `cli` domain

### Search with no results

1. Agent searches for a term that matches no cards
2. Server returns `{ "results": [], "total": 0, "truncated": false }`

### Search with limit

1. Agent provides `limit` to cap results
2. Server returns at most `limit` results

## Failures / Exceptions

- If `specre.toml` cannot be loaded, returns `McpError::internal_error`
- Invalid status value returns an error result with `isError: true`
