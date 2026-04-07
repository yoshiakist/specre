---
id: "01KHFEA9QVV4A127VCRJY97A68"
name: "specre_coverage_reports_source_file_tagging"
status: "stable"
last_verified: "2026-04-07"
---

## Related Files

- `src/commands/coverage.rs`
- `src/scanner.rs` (reuses scanning helpers)
- `src/config.rs`
- `tests/cli_coverage.rs` (Test)

## Functional Overview

`specre coverage` reports what percentage of source files are linked to specre cards via `@specre` tags. It scans all files in the configured `source_dirs`, counts how many contain at least one `@specre` marker, and outputs the coverage ratio. The command applies the standard source-file filters: dot files/directories and SVG files are always skipped, `exclude_patterns` globs are applied if configured, and `target_extensions` whitelist is applied if set. A `--ext` flag can override the `target_extensions` filter for a single invocation. This gives developers and AI agents a quick measure of how well-traced the codebase is.

## Design Intent

Coverage is a key metric for assessing traceability health. If most source files lack `@specre` markers, the bidirectional traceability system is incomplete and agents cannot rely on specre cards as a comprehensive description of the codebase. By surfacing this as a single number, the coverage command enables CI gates, agent preflight checks (via `health-check`), and developer awareness.

## Key Members

- `CoverageResult` — public struct returned by `compute_coverage()`, containing `total: usize`, `tagged: usize`, and `uncovered: Vec<String>`. This struct is the reusable unit: `execute()` formats it for human output, and future commands (e.g., `health-check`) can call `compute_coverage()` directly to obtain the same data without parsing CLI output.
- **Total files:** the number of files found in `source_dirs` (after dot-file, SVG, `exclude_patterns`, and `target_extensions`/`--ext` filters)
- **Tagged files:** the subset of total files that contain at least one `@specre <ULID>` marker
- **Coverage percentage:** `tagged / total * 100`, displayed as a percentage with one decimal place. Derived from `CoverageResult` fields at display time.
- `--ext` flag: comma-separated list of file extensions to filter by, overriding `target_extensions` from `specre.toml` for this invocation
- **Uncovered files display limit:** at most 30 uncovered files are shown. When truncated, a summary line indicates the total count. This prevents excessive output from overwhelming AI agent context windows.

## Scenarios

### Basic coverage report

1. User runs `specre coverage` in a project with 4 source files, 3 of which contain `@specre` markers
2. CLI prints:
   ```
   Coverage: 3/4 files (75.0%)
   ```
3. CLI exits with exit code 0

### Full coverage

1. User runs `specre coverage` in a project where all source files contain `@specre` markers
2. CLI prints:
   ```
   Coverage: 2/2 files (100.0%)
   ```
3. CLI exits with exit code 0

### No source files found

1. User runs `specre coverage` in a project where `source_dirs` exist but contain no files (or no files matching the extension filter)
2. CLI prints:
   ```
   Coverage: 0/0 files (N/A)
   ```
3. CLI exits with exit code 0

### Zero coverage

1. User runs `specre coverage` in a project where source files exist but none contain `@specre` markers
2. CLI prints:
   ```
   Coverage: 0/3 files (0.0%)
   ```
3. CLI exits with exit code 0

### Extension filtering via --ext flag

1. User runs `specre coverage --ext rs` in a project with `.rs` and `.ts` files
2. Only `.rs` files are counted in the denominator and numerator
3. CLI prints the coverage report reflecting only the filtered files

### Extension filtering via specre.toml target_extensions

1. User configures `target_extensions = ["rs"]` in `specre.toml`
2. User runs `specre coverage` (without `--ext`)
3. Only `.rs` files are counted
4. CLI prints the coverage report reflecting only the filtered files

### --ext flag overrides specre.toml target_extensions

1. User configures `target_extensions = ["rs"]` in `specre.toml`
2. User runs `specre coverage --ext ts`
3. Only `.ts` files are counted, ignoring the `target_extensions` config

### Uncovered files are listed

1. User runs `specre coverage` in a project with uncovered files (30 or fewer)
2. After the summary line, CLI prints uncovered files:
   ```
   Coverage: 1/3 files (33.3%)

   Uncovered files:
     src/bar.rs
     src/baz.rs
   ```
3. Uncovered file paths use forward slashes on all platforms and are sorted alphabetically

### Too many uncovered files are truncated

1. User runs `specre coverage` in a project with more than 30 uncovered files
2. CLI prints the first 30 uncovered files followed by a truncation summary:
   ```
   Coverage: 1/50 files (2.0%)

   Uncovered files:
     src/a.rs
     src/b.rs
     ... (30 of 49 shown)
   ```
3. The `--json` output includes all fields: `uncovered` (truncated to 30 items), `uncovered_total` (actual count), and `truncated` (boolean)
4. When `truncated` is false, `uncovered_total` is omitted from JSON output

### Dot files are always excluded

1. A project has `source_dirs = ["src"]` without `target_extensions`
2. `src/` contains `.keep`, `.gitignore`, and `main.rs`
3. User runs `specre coverage`
4. Only `main.rs` is counted as a source file; `.keep` and `.gitignore` are excluded
5. This also applies when `target_extensions` is set — dot files are never scanned

### SVG files are always excluded

1. A project has `source_dirs = ["src"]` without `target_extensions`
2. `src/` contains `icon.svg` and `main.rs`
3. User runs `specre coverage`
4. Only `main.rs` is counted as a source file; `icon.svg` is excluded despite being valid UTF-8 text
5. This also applies when `target_extensions` is set — SVG files are never scanned

### exclude_patterns filters files from coverage

1. User configures `exclude_patterns = ["*.test.ts"]` in `specre.toml`
2. `src/` contains `app.ts` (tagged) and `app.test.ts` (untagged)
3. User runs `specre coverage`
4. Only `app.ts` is counted; `app.test.ts` is excluded by the glob pattern
5. Coverage is 100.0%

### Invalid exclude_patterns are skipped with a warning

1. User configures `exclude_patterns = ["[invalid"]` in `specre.toml`
2. User runs `specre coverage`
3. CLI prints a warning to stderr about the invalid pattern and skips it
4. Remaining valid patterns and other filters still apply

### Paths use forward slashes

1. On all platforms, output paths use forward slashes (`/`), not backslashes

### specre.toml does not exist

1. User runs `specre coverage` in a directory without `specre.toml`
2. CLI exits with error: `Error: specre.toml not found. Run 'specre init' first.`

### source_dirs directory does not exist

1. User runs `specre coverage` where a `source_dirs` entry does not exist
2. CLI skips that directory silently and counts only files from directories that exist

## Failures / Exceptions

- If `specre.toml` is missing, CLI exits with error: `Error: specre.toml not found. Run 'specre init' first.`
- If a `source_dirs` entry does not exist, CLI skips it silently
- If a source file cannot be read (IO error), CLI prints a warning to stderr (`Warning: failed to read '<path>': <reason>`) and counts it as uncovered
- Markers inside string literals are excluded (same logic as `extract_marker_ulid`)
