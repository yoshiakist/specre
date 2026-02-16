---
id: "01KHJ98TFCDTCARMMX1GC5ZHXE"
name: "mcp_resources_expose_specre_cards"
status: "stable"
last_verified: "2026-02-16"
---

## Related Files

- `src/commands/mcp.rs`
- `src/commands/index.rs` (reused: `collect_md_files`, `parse_frontmatter`)
- `tests/mcp/resources.rs` (Test)
- `tests/mcp/helpers.rs` (Test helper)

## Functional Overview

The MCP server exposes specre cards as MCP Resources. AI agents can discover all available specre cards via `resources/list` and read any individual card via `resources/read` using the `specre:///<ULID>` URI scheme.

This allows agents to load relevant specifications into their context before starting a task, without relying on file system access.

## Design Intent

Resources are the read-only discovery mechanism. An agent's typical workflow is:

1. Call `resources/list` to see all available specre cards with their names and statuses
2. Call `resources/read` with a specific `specre:///<ULID>` to load the full Markdown content of a card

The server scans the `specre_dir` (from `specre.toml`) on each request — no caching, no stale state. This ensures the agent always sees the current state of the specre ecosystem.

## Key Members

- URI scheme: `specre:///<ULID>` (e.g., `specre:///01JMBJK7QRVX3N4P5G6H8W9Y0Z`)
- MIME type: `text/markdown`
- Description format: `[<status>] <name>` (e.g., `[stable] specre_new_scaffolds_a_new_specre`)

## Scenarios

### List all resources

1. Client sends `resources/list` request
2. Server scans `specre_dir` for `.md` files with valid front-matter
3. Server returns a resource entry for each card:
   - `uri`: `specre:///<ULID>`
   - `name`: the specre name from front-matter
   - `description`: `[<status>] <name>`
   - `mimeType`: `text/markdown`
4. Resources are sorted by ULID

### List resources with empty specre directory

1. `specre_dir` exists but contains no `.md` files
2. Client sends `resources/list`
3. Server returns an empty `resources` array — no error

### Read a specific resource

1. Client sends `resources/read` with `uri: "specre:///<ULID>"`
2. Server finds the card whose front-matter `id` matches the ULID
3. Server returns the full Markdown content of the card (including front-matter)

### Read a nonexistent resource

1. Client sends `resources/read` with a ULID that does not match any card
2. Server returns a JSON-RPC error with code `-32002` (resource not found)

### Read with invalid URI prefix

1. Client sends `resources/read` with a URI that does not start with `specre:///`
2. Server returns a JSON-RPC error with code `-32602` (invalid params)

## Failures / Exceptions

- Malformed front-matter files are silently skipped in `resources/list` (consistent with `specre index` behavior)
- If a card file becomes unreadable between `list` and `read`, the server returns an internal error
