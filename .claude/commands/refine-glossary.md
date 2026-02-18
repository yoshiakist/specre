You are executing the glossary refinement workflow. The goal is to ensure `glossary.toml` contains terms that maximize the usefulness of `specre search` hints for AI coding agents.

The glossary directly controls the quality of `suggested_terms` in search hints. A well-tuned glossary helps agents narrow searches in 1-2 tool calls; a poorly-tuned one wastes context window tokens on noise.

---

## Phase 0: MCP Preflight

If specre MCP tools are available, run `health-check` first.

- **healthy = true** → The specre ecosystem is trustworthy. Proceed — `specre search` results in Phase 2 will accurately reflect the card corpus.
- **healthy = false** → The index or coverage may be stale. Run `specre index` to regenerate, then re-run `health-check`. If still unhealthy, note the gaps but proceed with caution — audit results may be incomplete.

## Phase 1: Context

1. Read `README.md` — internalize the philosophy, especially "context-window aware" and "one file, one behavior"
2. Read `src/commands/search/hint.rs` — understand how glossary terms become suggestions:
   - **Truncation hints:** terms are excluded if they match ALL results or are already in the query; remaining terms are sorted by `match_count` descending
   - **Zero-result hints:** terms with `match_count > 0` are shown sorted by `match_count` descending, with no total-match exclusion
3. Read the current `glossary.toml` and note the existing term categories and count

## Phase 2: Audit

Run `specre search` across **at least 20 diverse patterns** covering these categories:

### 2a. Single-keyword searches for each specre subcommand

Run searches for each command name (`init`, `new`, `index`, `status`, `trace`, `orphan`, `tag`, `search`, `coverage`, `health-check`, `mcp`). Record the result count and whether truncation occurs.

### 2b. AND searches combining domain concepts

Test combinations that an AI agent would naturally use, e.g.:
- `"trace bidirectional"`, `"tag marker"`, `"mcp server"`, `"coverage scan"`, `"search hint"`, `"index generate"`, `"orphan detect"`

These should narrow results to 1-5 cards. If they don't, the glossary may be missing discriminating terms.

### 2c. Truncation hint quality

For queries that trigger truncation (total > `max_results`), examine `suggested_terms`:
- Are the top suggestions useful for narrowing, or are they near-universal terms?
- Would adding a suggested term as an AND keyword meaningfully reduce results?

### 2d. Zero-result hint quality

Test terms that don't exist in any card (e.g., `"database"`, `"webhook"`, `"deploy"`). Examine `suggested_terms`:
- Do the top suggestions help the agent discover relevant vocabulary, or are they noise?
- Are near-universal terms (matching 90%+ of cards) crowding out useful suggestions?

### 2e. Vocabulary gap detection

Test terms an agent might naturally use but that don't appear in cards — synonyms, abbreviations, or alternative phrasings. If these return 0 results without helpful suggestions, the glossary needs bridging terms.

## Phase 3: Apply Refinement Principles

Evaluate each term against these criteria:

### Remove if:

| Criterion | Rationale |
|-----------|-----------|
| Matches 80%+ of all cards | Provides no narrowing value in truncation hints; dominates zero-result suggestions with noise |
| Already filterable via `--status` or `--domain` | Redundant — agents should use structured filters for these axes |
| Generic data type (`string`, `integer`, `date`, etc.) | Not a searchable concept — agents don't search for data types |
| Duplicate entry | Wastes a suggestion slot |

### Keep if:

| Criterion | Rationale |
|-----------|-----------|
| Command/feature name | Directly maps to a behavior an agent wants to find |
| Discriminating operation verb | Narrows results when combined with a noun via AND search |
| Domain-specific concept unique to this project | Bridges vocabulary gaps — the primary purpose of the glossary |

### Add if:

| Criterion | Rationale |
|-----------|-----------|
| Concept that appears in some cards but is missing from the glossary | Expands the suggestion vocabulary |
| Synonym or alternative phrasing for existing concepts | Helps agents using different vocabulary find the right cards |

## Phase 4: Update and Verify

1. Update `glossary.toml` with the refined term list
2. Re-run the same 20+ search patterns from Phase 2
3. Compare before/after:
   - Truncation hints: are top `suggested_terms` more discriminating?
   - Zero-result hints: are suggestions more relevant and less noisy?
   - AND combinations: do new glossary terms enable effective narrowing?
4. If the results are unsatisfactory, iterate — adjust terms and re-test

## Phase 5: Summary

Present a concise report:
- Terms added (with rationale)
- Terms removed (with rationale)
- Before/after comparison of hint quality for representative queries
- Any observations about the search system's effectiveness for AI agent workflows
