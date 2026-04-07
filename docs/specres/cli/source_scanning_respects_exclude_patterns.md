---
id: "01KJ4NJW28F072X64SS68P3N08"
name: "source_scanning_respects_exclude_patterns"
status: "stable"
last_verified: "2026-04-07"
---

## Related Files

- src/config.rs
- src/scanner.rs
- src/commands/init.rs
- src/commands/coverage.rs
- src/commands/orphans.rs
- src/commands/health_check.rs
- src/commands/index.rs
- src/commands/trace.rs
- src/commands/mcp/helpers.rs
- tests/cli_coverage.rs
- tests/cli_orphans.rs
- tests/cli_trace.rs
- tests/cli_health_check.rs
- tests/cli_index.rs

## Functional Overview

Setting `exclude_patterns` (an array of glob patterns) in `specre.toml` excludes matching files and directories from source scanning. The exclusion applies to all commands that scan source files: `coverage`, `orphans`, `trace` (ULID mode), `index`, `health-check`, and the corresponding MCP handlers.

Patterns are compiled into a single `GlobSet` via the `globset` crate and matched against forward-slash-normalized paths. Invalid patterns emit a warning to stderr and are skipped; processing continues with the remaining valid patterns.

Exclusion acts as a filter during directory traversal (scanning). When a file path is specified explicitly — such as `specre trace <file-path>` — exclude patterns are not applied. This follows the same philosophy as `.gitignore` where `git show` still works on ignored files: filters apply to discovery, not to explicit references.

## Scenarios

### File glob excludes matching files from coverage

1. Set `exclude_patterns = ["*.test.ts"]` in `specre.toml`
2. `src/app.ts` (tagged) and `src/app.test.ts` (untagged) exist
3. Run `specre coverage`
4. Coverage is 1/1 (100%) — `.test.ts` files are excluded from the denominator

### Directory pattern excludes entire subtree

1. Set `exclude_patterns = ["*/_generated/*"]` in `specre.toml`
2. `src/main.rs` (tagged), `src/_generated/schema.rs`, and `src/_generated/types.rs` (untagged) exist
3. Run `specre coverage`
4. Coverage is 1/1 (100%) — files under `_generated/` are excluded

### Combined with target_extensions

1. Set `target_extensions = ["ts"]` and `exclude_patterns = ["*.test.ts"]`
2. `src/app.ts` (tagged), `src/util.ts` (untagged), `src/app.test.ts`, and `src/script.py` exist
3. Run `specre coverage`
4. `.py` is excluded by the extension filter, `.test.ts` is excluded by the pattern — coverage is 1/2 (50%)

### Excluded files' dangling markers are not reported by orphans

1. Set `exclude_patterns = ["*.test.ts"]`
2. `src/app.test.ts` contains an `@specre` marker with a non-existent ULID
3. Run `specre orphans`
4. The marker is not reported as dangling

### Excluded files' source refs are hidden from trace by ULID

1. Set `exclude_patterns = ["*.test.ts"]`
2. Both `src/main.rs` and `src/app.test.ts` contain an `@specre` marker with the same ULID
3. Run `specre trace <ULID>`
4. Only `src/main.rs` is shown; `src/app.test.ts` is hidden

### Trace by file-path ignores exclude patterns

1. Set `exclude_patterns = ["*.test.ts"]`
2. `src/app.test.ts` contains `@specre` markers
3. Run `specre trace src/app.test.ts`
4. The marker ULIDs are displayed normally — explicit file references bypass exclusion

### Excluded files' source refs are not in index.json

1. Set `exclude_patterns = ["*.test.ts"]`
2. Both `src/main.rs` and `src/app.test.ts` contain `@specre` markers
3. Run `specre index`
4. `index.json` `source_refs` includes only `src/main.rs`

### Invalid pattern warns and continues

1. Set `exclude_patterns = ["[invalid"]`
2. Run `specre coverage`
3. stderr shows `Warning: invalid exclude pattern`
4. The command completes successfully; the invalid pattern is ignored

### Backward compatibility: no exclude_patterns

1. `specre.toml` has no `exclude_patterns` field
2. All commands work as before (covered by existing tests)
