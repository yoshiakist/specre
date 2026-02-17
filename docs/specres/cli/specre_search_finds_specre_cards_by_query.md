---
id: "01KHFTCYJN8YJMW2RNHJTAQV49"
name: "specre_search_finds_specre_cards_by_query"
status: "stable"
last_verified: "2026-02-18"
---

## Related Files

- `src/commands/search.rs`
- `src/cli.rs`
- `src/commands/mod.rs`
- `src/config.rs`
- `tests/cli_search.rs` (Test)

## Functional Overview

`specre search` discovers specre cards by free-text query and structured filters. The free-text query performs a case-insensitive substring match against the entire content of each specre card (front-matter and Markdown body). Structured filters (`--status`, `--domain`, `--verified-before`, `--verified-after`) narrow results by metadata. All parameters are optional; omitting the text query returns all cards matching the filters. Output is JSON with an `excerpt` field (the first 200 characters of the card's first prose paragraph) so that agents can decide whether to read the full card. When the number of matching results exceeds a configurable threshold, the CLI omits the individual results and instead returns the total count with a `hint` object containing available domains and per-status counts, prompting the caller to refine the query. The `--limit` flag overrides this behavior by explicitly requesting up to N results.

## Design Intent

`search` is the discovery entry point — "find a specre I don't yet know exists." It complements `trace` (navigate from a known ULID or file) and `orphans` (quality audit). In an MCP-enabled workflow, an agent runs `health-check` first, then `search` to locate relevant specre cards before reading them in full. The API is designed so that a single tool call with a natural-language query is the common case, while structured filters serve as optional refinements.

Search results are consumed as LLM input tokens by coding agents. Returning too many results wastes context window capacity and degrades reasoning quality. The truncation threshold acts as a guardrail: when results exceed it, the CLI withholds individual entries and instead provides metadata (total count, available domains, per-status breakdown) that guides the agent toward a more precise follow-up query. This keeps the agent's context budget focused on the specre cards that actually matter.

## Key Members

- `query: Option<String>` — free-text substring to match against card content (positional, optional)
- `--status <status>` — filter by status (`draft`, `in-development`, `stable`, `deprecated`)
- `--domain <domain>` — filter by domain (top-level directory under `specre_dir`)
- `--verified-before <YYYY-MM-DD>` — include only specres whose `last_verified` is before this date. Specres without `last_verified` are included (they have never been verified, so they are "before" any date)
- `--verified-after <YYYY-MM-DD>` — include only specres whose `last_verified` is on or after this date. Specres without `last_verified` are excluded
- `--limit <N>` — return at most N results, bypassing the truncation threshold. When specified, results are never truncated regardless of count

## Scenarios

### Free-text query matches card content

1. Project has multiple specre cards, one of which contains "password" in its Functional Overview
2. User runs `specre search "password"`
3. CLI reads all specre cards under `specre_dir`, performs case-insensitive substring matching against each card's full content
4. CLI outputs JSON to stdout:
   ```json
   {
     "results": [
       {
         "id": "01HZYPMZRK8F9R2DGBGGMM2N8T",
         "name": "user_can_reset_password",
         "status": "stable",
         "domain": "auth",
         "path": "docs/specres/auth/user_can_reset_password.md",
         "last_verified": "2026-03-01",
         "excerpt": "Users can reset their password by providing their registered email address."
       }
     ],
     "total": 1
   }
   ```
5. CLI exits with exit code 0

### Free-text query matches front-matter name

1. User runs `specre search "orphans"`
2. CLI matches the specre whose `name` contains "orphans"
3. CLI outputs JSON with that specre in the results array

### Filter by status only (no text query)

1. User runs `specre search --status draft`
2. CLI returns all specres with `status: "draft"` in their front-matter
3. CLI exits with exit code 0

### Filter by domain

1. User runs `specre search --domain auth`
2. CLI returns all specres whose top-level directory under `specre_dir` is `auth`
3. CLI exits with exit code 0

### Filter by verified-before

1. User runs `specre search --verified-before 2026-02-01`
2. CLI returns specres whose `last_verified` is before `2026-02-01`, plus specres that have no `last_verified` field
3. CLI exits with exit code 0

### Filter by verified-after

1. User runs `specre search --verified-after 2026-02-01`
2. CLI returns only specres whose `last_verified` is on or after `2026-02-01`
3. Specres without `last_verified` are excluded from results
4. CLI exits with exit code 0

### Combining text query with filters

1. User runs `specre search "validation" --status stable --domain auth`
2. CLI applies all filters: text matches card content AND status is stable AND domain is auth
3. Only specres satisfying all conditions are returned
4. CLI exits with exit code 0

### No parameters — returns all specres

1. User runs `specre search` with no query and no filters
2. CLI returns all specres in the project
3. CLI exits with exit code 0

### Results exceed truncation threshold

1. `specre.toml` has `[search] max_results = 10` (default)
2. User runs `specre search "specre"` in a project with 15 matching specres across domains `auth` and `cli`
3. CLI outputs JSON with `results` empty and a `hint` object:
   ```json
   {
     "results": [],
     "total": 15,
     "truncated": true,
     "hint": {
       "message": "Too many results (15). Refine your query with --status, --domain, or a more specific search term.",
       "available_domains": ["auth", "cli"],
       "status_counts": { "draft": 3, "in-development": 2, "stable": 9, "deprecated": 1 }
     }
   }
   ```
4. CLI exits with exit code 0

### Results within truncation threshold

1. `specre.toml` has `[search] max_results = 10`
2. User runs `specre search "password"` and 3 specres match
3. CLI outputs JSON with all 3 results in `results` array, `"truncated": false`, no `hint` field
4. CLI exits with exit code 0

### --limit overrides truncation threshold

1. `specre.toml` has `[search] max_results = 10`
2. User runs `specre search "specre" --limit 5` and 15 specres match
3. CLI returns the first 5 results (by sort order: domain, then name), `"total": 15`, `"truncated": false`
4. CLI exits with exit code 0

### Default truncation threshold

1. `specre.toml` does not have a `[search]` section
2. The default `max_results` is 10
3. Behavior is the same as if `[search] max_results = 10` were specified

### No results found

1. User runs `specre search "nonexistent_term_xyz"`
2. CLI outputs:
   ```json
   {
     "results": [],
     "total": 0
   }
   ```
3. CLI exits with exit code 0 (no match is not an error)

### Excerpt extraction

1. The `excerpt` is extracted from the card's **first prose paragraph** — the first contiguous block of non-empty lines in the Markdown body (after front-matter) that are not section headings (`##`) and not list items (`- `). Lines within the paragraph are joined with a space into a single string
2. The excerpt is capped at 200 characters. If it exceeds this limit, it is truncated and suffixed with `…` (U+2026 horizontal ellipsis)
3. If no prose paragraph is found (e.g., a card containing only headings and lists), `excerpt` is `null`

### Results are sorted by domain then name

1. When multiple specres match, results are sorted alphabetically by `domain`, then by `name` within each domain
2. This provides a stable, predictable ordering for agents

### Paths use forward slashes

1. On all platforms, the `path` field in output uses forward slashes (`/`), not backslashes

### specre.toml does not exist

1. User runs `specre search "anything"` in a directory without `specre.toml`
2. CLI outputs to stderr: `Error: specre.toml not found. Run 'specre init' first.`
3. CLI exits with exit code 1

### specre_dir does not exist

1. User runs `specre search` but the configured `specre_dir` does not exist
2. CLI returns empty results:
   ```json
   {
     "results": [],
     "total": 0
   }
   ```
3. CLI exits with exit code 0

## Failures / Exceptions

- If `specre.toml` is missing, CLI exits with error: `Error: specre.toml not found. Run 'specre init' first.`
- If `specre_dir` does not exist, CLI returns empty results (exit code 0)
- If a specre card has malformed front-matter (unparseable YAML), it is skipped with a warning to stderr and not included in results
- If a specre card cannot be read (IO error), CLI prints a warning to stderr (`Warning: failed to read '<path>': <reason>`) and skips that file
- `--verified-before` and `--verified-after` expect a valid calendar date in `YYYY-MM-DD` format. Both syntactically malformed strings (e.g., `not-a-date`) and well-formed but calendar-invalid dates (e.g., `2025-02-30`, `2025-13-01`) produce an error: `Error: invalid date format: <value>. Expected YYYY-MM-DD.`
- `--status` accepts only `draft`, `in-development`, `stable`, `deprecated`; other values produce an error: `Error: invalid status: <value>. Expected one of: draft, in-development, stable, deprecated.`
- `--limit` must be a positive integer; zero or negative values produce an error: `Error: --limit must be a positive integer.`
