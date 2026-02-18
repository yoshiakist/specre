---
id: "01KHQBKWZY2D77XP7A50HGTZQ8"
name: "specre_search_hints_guide_query_refinement"
status: "stable"
last_verified: "2026-02-18"
---

## Related Files

- `src/commands/search.rs`
- `src/config.rs`
- `tests/cli_search.rs` (Test)

## Functional Overview

When `specre search` returns a problematic result set — too many results or zero results — the CLI provides a `hint` object in the JSON output to guide the caller toward a more effective follow-up query. This keeps coding agents from wasting context window capacity on unhelpful results or from abandoning search after a miss.

The hint system has two triggers:

1. **Truncation (too many results):** When the number of matching results exceeds a configurable threshold (`[search] max_results` in `specre.toml`, default 10), the CLI omits individual results and returns a hint containing `available_domains`, `status_counts`, and optionally `suggested_terms` from the project glossary.
2. **Zero results:** When a text query matches no cards, the CLI returns a hint containing `keyword_matches` (per-keyword match counts against all cards) and optionally `suggested_terms` from the glossary.

When a `glossary.toml` file exists in the project root, the hint is enriched with vocabulary-based suggestions. The glossary is optional; when absent, hints operate without the `suggested_terms` field.

## Design Intent

Search results are consumed as LLM input tokens by coding agents. Returning too many results wastes context window capacity and degrades reasoning quality. The truncation threshold acts as a guardrail: when results exceed it, the CLI withholds individual entries and instead provides metadata (total count, available domains, per-status breakdown) that guides the agent toward a more precise follow-up query. This keeps the agent's context budget focused on the specre cards that actually matter.

The optional glossary feature addresses a common failure mode in agent workflows: the agent searches using vocabulary that doesn't appear in any specre card (e.g., searching for "login" when all cards use "authentication"). Without guidance, the agent either gives up or wastes multiple round-trips guessing terms. The glossary — a manually curated list of project-specific vocabulary stored in `glossary.toml` — bridges this gap. When a search returns zero results, the hint shows which glossary terms actually exist in the spec base, enabling the agent to reformulate in a single follow-up. For too-many results, glossary terms suggest narrowing keywords. The glossary is intentionally a static, human-curated artifact rather than an auto-generated index, because the value lies in capturing domain-specific vocabulary (synonyms, abbreviations, project conventions) that cannot be inferred from card content alone.

## Key Members

- `hint: Option<Hint>` — present in the search output when the result set is problematic (truncated or zero results with actionable refinement)
- `hint.message: String` — human-readable guidance message
- `hint.available_domains: Vec<String>` — unique domains among matched cards (truncation hint only)
- `hint.status_counts: BTreeMap<Status, usize>` — per-status breakdown among matched cards (truncation hint only)
- `hint.keyword_matches: Vec<KeywordMatch>` — per-keyword match counts against all cards, ignoring filters (zero-result hint only)
- `hint.suggested_terms: Vec<SuggestedTerm>` — glossary terms with their match counts (present when `glossary.toml` exists)
- `glossary.toml` — optional TOML file in the project root with a `terms: Vec<String>` field

## Scenarios

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

### Default truncation threshold

1. `specre.toml` does not have a `[search]` section
2. The default `max_results` is 10
3. Behavior is the same as if `[search] max_results = 10` were specified

### No results found (single keyword, no glossary)

1. Project has no `glossary.toml`
2. User runs `specre search "nonexistent_term_xyz"`
3. CLI outputs:
   ```json
   {
     "results": [],
     "total": 0,
     "truncated": false
   }
   ```
4. No `hint` field (single keyword with no glossary provides no actionable refinement)
5. CLI exits with exit code 0 (no match is not an error)

### No results with multi-keyword AND query — keyword match counts

1. Project has no `glossary.toml`
2. Project has specre cards containing "password" but none containing "reset"
3. User runs `specre search "password reset"` (AND mode, default)
4. No cards contain both keywords, so total is 0
5. CLI outputs JSON with a `hint` object:
   ```json
   {
     "results": [],
     "total": 0,
     "truncated": false,
     "hint": {
       "message": "No results found. Consider removing or replacing some keywords.",
       "keyword_matches": [
         {"keyword": "password", "match_count": 3},
         {"keyword": "reset", "match_count": 0}
       ]
     }
   }
   ```
6. `keyword_matches` counts each keyword against ALL cards individually (ignoring filters and other keywords), sorted by `match_count` descending
7. No `suggested_terms` field (glossary not present)
8. CLI exits with exit code 0

### No results with glossary — vocabulary suggestions

1. Project has `glossary.toml` in project root with `terms = ["user", "authentication", "password", "session", "create", "delete"]`
2. Project has specre cards containing "authentication" and "password" in their content, but no cards contain "login"
3. User runs `specre search "login"`
4. CLI outputs JSON with a `hint` object:
   ```json
   {
     "results": [],
     "total": 0,
     "truncated": false,
     "hint": {
       "message": "No results found. Consider adjusting your query.",
       "keyword_matches": [
         {"keyword": "login", "match_count": 0}
       ],
       "suggested_terms": [
         {"term": "authentication", "match_count": 3},
         {"term": "password", "match_count": 2},
         {"term": "user", "match_count": 5},
         {"term": "session", "match_count": 1}
       ]
     }
   }
   ```
5. `suggested_terms` contains only glossary terms with `match_count > 0` that are not in the query, sorted by `match_count` descending, capped at 10 entries
6. CLI exits with exit code 0

### No results with multi-keyword query and glossary — combined hint

1. Project has `glossary.toml` with terms including "authentication"
2. User runs `specre search "login reset"`
3. CLI outputs a hint with both `keyword_matches` (per-keyword counts against all cards) and `suggested_terms` (glossary terms that match cards)
4. The agent can see which keywords are problematic and what alternative terms exist

### Zero-result hint conditions

1. The zero-result hint is generated only when `total == 0` AND a text query is present AND either `glossary.toml` exists OR the query contains two or more keywords
2. When glossary exists: hint includes both `keyword_matches` and `suggested_terms`
3. When glossary is absent but keywords >= 2: hint includes only `keyword_matches`
4. When glossary is absent and keyword count is 1: no hint (existing behavior preserved)

### Results exceed truncation threshold with glossary

1. Project has `glossary.toml` with `terms = ["create", "delete", "update", "user"]`
2. `specre.toml` has `[search] max_results = 10`
3. User runs `specre search "specre"` and 15 cards match
4. CLI outputs JSON with the existing `hint` fields (`message`, `available_domains`, `status_counts`) plus a `suggested_terms` array:
   ```json
   {
     "results": [],
     "total": 15,
     "truncated": true,
     "hint": {
       "message": "Too many results (15). Refine your query with --status, --domain, or a more specific search term.",
       "available_domains": ["auth", "cli"],
       "status_counts": { "draft": 3, "stable": 12 },
       "suggested_terms": [
         {"term": "create", "match_count": 5},
         {"term": "delete", "match_count": 3},
         {"term": "user", "match_count": 8}
       ]
     }
   }
   ```
5. `suggested_terms` excludes terms already in the query, terms with `match_count == 0`, and terms with `match_count` equal to the total (they don't narrow the results)
6. Sorted by `match_count` descending, capped at 10

### Results exceed truncation threshold without glossary (unchanged)

1. When `glossary.toml` does not exist, the too-many-result hint behaves exactly as before: `message`, `available_domains`, `status_counts`
2. No `suggested_terms` field

### Glossary file missing — graceful degradation

1. Project has no `glossary.toml` in the project root
2. All search behavior is identical to the pre-glossary behavior
3. The `suggested_terms` field never appears in the output
4. `keyword_matches` may still appear in the zero-result hint (when query has 2+ keywords)

## Failures / Exceptions

- If `glossary.toml` exists but cannot be parsed (malformed TOML or missing `terms` field), CLI prints a warning to stderr (`Warning: failed to parse glossary.toml: <reason>`) and continues without vocabulary suggestions — the search itself is not affected
