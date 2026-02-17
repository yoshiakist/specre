---
id: "01KHFGVXWP100JXYBZTRJGMB9H"
name: "specre_health_check_verifies_ecosystem_trustworthiness"
status: "stable"
last_verified: "2026-02-17"
---

## Related Files

- `src/commands/health_check.rs`
- `src/commands/index.rs` (reuses `scan_source_markers()`)
- `src/commands/coverage.rs` (reuses `coverage_from_scan()`)
- `src/commands/orphans.rs` (reuses `orphans_from_scan()`)
- `src/cli.rs`
- `src/commands/mod.rs`
- `src/main.rs`
- `src/config.rs`
- `tests/cli_health_check.rs` (Test)

## Functional Overview

`specre health-check` is a single entry point for coding agents to verify that the specre ecosystem is trustworthy before starting a task. It aggregates three metrics — coverage ratio, orphan count, and index freshness — into one structured JSON response, enabling agents to unambiguously determine whether specre cards can be relied upon without interpreting multiple commands individually. Each metric is compared against configurable thresholds (with sensible defaults), and a top-level `healthy` boolean summarizes whether all metrics are within acceptable bounds.

## Design Intent

AI agents need a fast, unambiguous signal before trusting specre cards as a source of truth for a coding session. Rather than requiring agents to run `coverage`, `orphans`, and check `index.json` separately — and then interpret the combined results — `health-check` provides a single JSON object with a clear `healthy` verdict. This makes it ideal as the first command an agent runs at the start of a session, or as an MCP server query.

## Key Members

- `HealthCheckResult` — struct containing `healthy: bool`, `coverage: f64`, `orphans: usize`, `index_age_hours: f64`, and `thresholds: Thresholds`
- `Thresholds` — struct with `coverage: f64` (default `0.90`), `orphans: usize` (default `5`), `index_age_hours: f64` (default `24.0`); configurable via `specre.toml` under `[health_check]` section
- `healthy` — `true` when `coverage >= thresholds.coverage` AND `orphans <= thresholds.orphans` AND `index_age_hours <= thresholds.index_age_hours`
- `coverage` — ratio (0.0–1.0) of source files containing at least one `@specre` marker, computed via `compute_coverage()`
- `orphans` — total count of orphan specres (non-deprecated specres with no source markers) plus dangling markers (markers with no matching specre), computed via `compute_orphans()`
- `index_age_hours` — hours since `index.json` was last generated, derived from its `generated_at` field. If `index.json` does not exist, this is reported as `null` and the metric is treated as failing (unhealthy)

## Scenarios

### Healthy ecosystem — all metrics within thresholds

1. Project has `specre.toml`, source files with good coverage, few orphans, and a recent `index.json`
2. User runs `specre health-check`
3. CLI outputs JSON to stdout:
   ```json
   {
     "healthy": true,
     "coverage": 0.93,
     "orphans": 2,
     "index_age_hours": 3.2,
     "thresholds": { "coverage": 0.90, "orphans": 5, "index_age_hours": 24.0 }
   }
   ```
4. CLI exits with exit code 0

### Unhealthy ecosystem — coverage below threshold

1. User runs `specre health-check` in a project where coverage is 0.50 (below default 0.90)
2. CLI outputs JSON with `"healthy": false` and `"coverage": 0.50`
3. CLI exits with exit code 1

### Unhealthy ecosystem — orphans above threshold

1. User runs `specre health-check` in a project where orphan count is 8 (above default 5)
2. CLI outputs JSON with `"healthy": false` and `"orphans": 8`
3. CLI exits with exit code 1

### Unhealthy ecosystem — index.json missing

1. User runs `specre health-check` in a project without `index.json`
2. CLI outputs JSON with `"healthy": false` and `"index_age_hours": null`
3. CLI exits with exit code 1

### Unhealthy ecosystem — index.json stale

1. User runs `specre health-check` where `index.json` was generated more than 24 hours ago
2. CLI outputs JSON with `"healthy": false` and `"index_age_hours"` reflecting the actual age
3. CLI exits with exit code 1

### Custom thresholds via specre.toml

1. User configures `specre.toml`:
   ```toml
   [health_check]
   coverage = 0.50
   orphans = 10
   index_age_hours = 48.0
   ```
2. User runs `specre health-check`
3. CLI uses the custom thresholds for evaluation and includes them in the output JSON
4. A project with 60% coverage, 8 orphans, and 36-hour-old index would report `"healthy": true`

### No source files — coverage is 0.0

1. User runs `specre health-check` in a project where `source_dirs` contain no files
2. Coverage is `0.0` (below default threshold 0.90)
3. CLI outputs JSON with `"healthy": false` and `"coverage": 0.0`
4. CLI exits with exit code 1

### Multiple metrics failing simultaneously

1. User runs `specre health-check` with low coverage and missing `index.json`
2. CLI outputs JSON with `"healthy": false`, showing all metric values
3. CLI exits with exit code 1

### specre.toml does not exist

1. User runs `specre health-check` in a directory without `specre.toml`
2. CLI outputs to stderr: `Error: specre.toml not found. Run 'specre init' first.`
3. CLI exits with exit code 1

### JSON output is machine-parseable

1. The output is valid JSON on a single object (pretty-printed)
2. Field names use snake_case
3. `coverage` is a floating-point number (0.0–1.0), not a percentage
4. `index_age_hours` is a floating-point number rounded to one decimal place, or `null` if index.json is missing
5. `orphans` is an integer

## Failures / Exceptions

- If `specre.toml` is missing, CLI exits with error: `Error: specre.toml not found. Run 'specre init' first.`
- If `index.json` exists but cannot be read (IO error other than not found), CLI prints a warning to stderr and `index_age_hours` is reported as `null`
- If `index.json` exists but cannot be parsed (malformed JSON or missing `generated_at`), CLI prints a warning to stderr and `index_age_hours` is reported as `null` and treated as failing
- If `source_dirs` entries do not exist, they are skipped silently (consistent with `coverage` behavior)
- If `specre_dir` does not exist, orphan count is 0 (no specres to be orphaned)
