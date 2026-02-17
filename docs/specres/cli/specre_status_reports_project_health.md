---
id: "01KHAN6JE712ZAKXPP97854PKJ"
name: "specre_status_reports_project_health"
status: "stable"
last_verified: "2026-02-17"
---

## Related Files

- `src/commands/status.rs`
- `src/config.rs`
- `tests/cli_status.rs` (Test)

## Functional Overview

`specre status` scans all specre files in the project and reports a summary of project health: counts by lifecycle status (draft, in-development, stable, deprecated) and a list of stable specres whose `last_verified` date is stale. It reads `specre.toml` to determine the specre directory.

## Design Intent

The status command gives developers and AI agents a quick health check of their specification base. By surfacing stale specres — those whose `last_verified` date exceeds a threshold — it encourages regular re-verification and prevents specification drift from going unnoticed.

Staleness threshold defaults to 30 days but can be overridden via the `--threshold` option, allowing teams to set their own cadence.

## Key Members

- `threshold: u32` — number of days after which a stable specre's `last_verified` is considered stale (default: 30)
- `StatusSummary` — aggregated counts: `draft`, `in_development`, `stable`, `deprecated`, `total`
- `StaleEntry` — a stable specre whose `last_verified` is older than the threshold: contains `name`, `path`, `last_verified`, `days_since`

## Scenarios

### Basic invocation shows status summary

1. User runs `specre status` in a project with `specre.toml` and several specre files in different statuses
2. CLI reads `specre.toml` to determine `specre_dir`
3. CLI scans all `.md` files under `specre_dir` recursively, parsing YAML front-matter
4. CLI prints a summary table to stdout:
   ```
   Status Summary:
     draft:          2
     in-development: 1
     stable:         5
     deprecated:     0
     total:          8
   ```

### Stale specres are flagged

1. User runs `specre status` in a project where some stable specres have `last_verified` dates older than 30 days from today
2. CLI prints the status summary (as above)
3. CLI prints a stale specres section to stdout:
   ```
   Stale specres (last_verified > 30 days):
     user_can_reset_password       docs/specres/auth/user_can_reset_password.md       (45 days)
     cart_total_reflects_changes   docs/specres/cart/cart_total_reflects_changes.md    (62 days)
   ```

### No stale specres

1. User runs `specre status` and all stable specres have `last_verified` within 30 days
2. CLI prints the status summary
3. No stale specres section is printed

### Custom threshold via --threshold

1. User runs `specre status --threshold 90`
2. CLI uses 90 days as the staleness threshold instead of the default 30
3. Only stable specres whose `last_verified` exceeds 90 days are flagged

### Stable specres without last_verified are always flagged

1. A stable specre has no `last_verified` field in its front-matter
2. CLI treats it as stale regardless of the threshold
3. The stale entry shows `(no last_verified)` instead of the day count

### Invalid last_verified format is flagged as stale

1. A stable specre has `last_verified: "yesterday"` (not YYYY-MM-DD format)
2. CLI prints a warning to stderr: `Warning: invalid last_verified in <path>: "yesterday"`
3. CLI treats it as stale, showing `(invalid last_verified)` instead of the day count

### Impossible date in last_verified is flagged as stale

1. A stable specre has `last_verified: "2026-02-30"` (valid format but non-existent date)
2. CLI prints a warning to stderr: `Warning: invalid last_verified in <path>: "2026-02-30"`
3. CLI treats it as stale, showing `(invalid last_verified)` instead of the day count

### last_verified on non-stable specres is ignored

1. A draft specre has `last_verified: "2026-01-01"` in its front-matter
2. CLI counts it under `draft` in the status summary
3. CLI does not include it in the stale specres section (stale detection applies only to stable specres)
4. No warning is emitted — the field is simply ignored

### Empty specre directory

1. User runs `specre status` with a valid `specre.toml` but no specre files exist
2. CLI prints the summary with all counts at 0
3. No stale specres section is printed

### specre.toml does not exist

1. User runs `specre status` in a directory without `specre.toml`
2. CLI exits with error: `Error: specre.toml not found. Run 'specre init' first.`

## Failures / Exceptions

- If `specre.toml` is missing, CLI exits with error: `Error: specre.toml not found. Run 'specre init' first.`
- If a specre file has malformed front-matter (missing `---` delimiters or required fields), CLI prints a warning to stderr and skips that file
- If `specre_dir` does not exist, CLI treats it as empty (all counts 0)
- If a stable specre's `last_verified` is not a valid YYYY-MM-DD date (wrong format or impossible date), CLI prints a warning to stderr and treats it as stale with `(invalid last_verified)`
- If a non-stable specre has `last_verified`, the field is silently ignored
