---
id: "01KHQKZ6H8FB46ESFXB03N85AN"
name: "mcp_tool_search_queries_specre_cards"
status: "stable"
last_verified: "2026-02-18"
---

## Related Files

- `src/commands/mcp/tools.rs` (tool handler)
- `src/commands/mcp/search.rs` (search logic)
- `src/commands/mcp/helpers.rs` (shared helpers)
- `src/commands/search/mod.rs` (shared `SearchableCard`, `scan_cards`, `extract_excerpt`)
- `src/commands/search/hint.rs` (shared hint logic)
- `tests/mcp/tool_search.rs` (Test)
- `tests/mcp/helpers.rs` (Test helper)

## Functional Overview

The MCP server exposes a `search` tool for full-text and filtered search across specre cards, equivalent to `specre search`. Multi-keyword queries use AND logic by default, with an `or` flag for OR logic. Supports filtering by status, domain, and date ranges. When the number of matching results exceeds a configurable threshold (`[search] max_results` in `specre.toml`, default 10), the tool omits individual results and returns a `hint` object guiding query refinement — identical to the CLI's truncation behavior.

## Design Intent

Agents exploring a project's specifications need to discover relevant cards by keyword, status, or domain. This tool provides that search capability over MCP, enabling agents to find specifications before reading or modifying them. The truncation threshold prevents flooding the agent's context window with too many results, and the hint system guides the agent toward more precise follow-up queries.

## Key Members

- `query: String` (optional) — free-text substring to match (case-insensitive, multi-keyword AND by default)
- `status: String` (optional) — filter by status (draft, in-development, stable, deprecated)
- `domain: String` (optional) — filter by domain
- `or: bool` (optional, default: false) — use OR logic for multi-keyword queries
- `limit: usize` (optional) — return at most N results, bypassing truncation threshold
- `verified_before: String` (optional) — include only specres verified before this date (YYYY-MM-DD)
- `verified_after: String` (optional) — include only specres verified on or after this date (YYYY-MM-DD)

Return value:
```json
{ "results": [...], "total": N, "truncated": false, "hint": { ... } | absent }
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
2. Server returns at most `limit` results with `"truncated": false` (limit bypasses truncation)

### Results exceed truncation threshold

1. `specre.toml` has `[search] max_results = 10` (default)
2. Agent searches and 15 cards match, no `limit` provided
3. Server returns `{ "results": [], "total": 15, "truncated": true, "hint": { "message": "Too many results (15). ...", "available_domains": [...], "status_counts": { ... } } }`
4. Agent uses hint metadata to refine the query

### Limit bypasses truncation threshold

1. `specre.toml` has `[search] max_results = 10`
2. Agent searches with `limit: 5` and 15 cards match
3. Server returns the first 5 results with `"truncated": false`, no `hint`

### Zero results with multi-keyword query — hint with keyword matches

1. Agent searches with `{ "query": "password reset" }` and no cards match both keywords
2. Server returns `{ "results": [], "total": 0, "truncated": false, "hint": { "message": "No results found. ...", "keyword_matches": [...] } }`

## Failures / Exceptions

- If `specre.toml` cannot be loaded, returns `McpError::internal_error`
- Invalid status value returns an error result with `isError: true`
- Invalid date format returns `McpError::invalid_params`
- `limit: 0` returns an error result
