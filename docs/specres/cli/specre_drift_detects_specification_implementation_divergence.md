---
id: "01KNJYEGK8KFVQK7EQGK9ZCZJR"
name: "specre_drift_detects_specification_implementation_divergence"
status: "stable"
last_verified: "2026-04-07"
---

## Related Files

- `src/commands/drift.rs` (Implementation)
- `src/cli.rs` (CLI argument definitions)
- `src/commands/mod.rs` (Module registration)
- `src/main.rs` (Command dispatch)
- `src/config.rs` (Drift config: `[drift] grace_days`)
- `tests/cli_drift.rs` (Test)

## Functional Overview

`specre drift` compares a specre card's `last_verified` date against the git last-modified date of its related source files. When source files have been modified after the specre's `last_verified` date (plus an optional grace period), the specre is reported as "drifted." This enables developers and AI agents to detect where specifications and implementation have diverged without manual review.

## Design Intent

Drift is a derived state, always computable from `last_verified` and git history. It is not an attribute of the specre card itself -- no `drifted: true` flag is written into front-matter. The command reports only; it never modifies specre cards. By detecting divergence early, teams can triage whether the spec or the code needs updating, preventing stale specifications from misleading future development.

## Key Members

- `last_verified: Option<String>` -- the date a specre was last confirmed to match implementation (from front-matter)
- `grace_days: u64` -- buffer period in days; changes within the grace period are not reported as drift
- `changed_files: Vec<ChangedFile>` -- source files linked to a drifted specre that have been modified since `last_verified + grace`
- `diff_stat: String` -- git diff stat for each changed file (`+N -M`)

## Scenarios

### Project-wide drift check with no drift detected

1. All stable specres have `last_verified` dates that are more recent than the git last-modified dates of their related source files
2. User runs `specre drift`
3. Output reports 0 drifted specres with a summary: `clean` count equals `total` count
4. Exit code is 0

### Project-wide drift check with drift detected

1. A stable specre has `last_verified: "2026-03-01"` and its related source file was modified on `2026-04-05`
2. User runs `specre drift`
3. Output lists the drifted specre with its changed files and diff stats
4. Exit code is 1

### Single specre check by ULID

1. User runs `specre drift <ULID>` targeting a specific specre
2. Only that specre is checked for drift
3. Output shows the drift status of the targeted specre only

### Path-based filtering

1. User runs `specre drift docs/specres/cli/` targeting a directory
2. Only specres under that path are checked
3. Output shows drift results for matching specres only

### Domain filtering

1. User runs `specre drift --domain cli`
2. Only specres in the `cli` domain are checked for drift
3. Output shows drift results for the filtered domain

### Status filtering

1. By default, only `stable` specres are checked (draft and in-development are expected to be out of sync)
2. User runs `specre drift --status in-development` to override the default filter
3. Only specres with the specified status are checked

### Grace period from specre.toml

1. `specre.toml` contains `[drift] grace_days = 7`
2. A stable specre has `last_verified: "2026-04-01"` and its source file was modified on `2026-04-05` (4 days later, within grace)
3. User runs `specre drift`
4. The specre is NOT reported as drifted because the change is within the grace period

### Grace period CLI override

1. `specre.toml` contains `[drift] grace_days = 7`
2. User runs `specre drift --grace 0d`
3. The grace period is overridden to 0 days, causing all post-`last_verified` changes to be reported

### Specre with no last_verified date

1. A stable specre has no `last_verified` field
2. User runs `specre drift`
3. The specre is always reported as drifted (no verification date means it has never been confirmed)

### Specre with no related source files

1. A stable specre has no files listed in Related Files and no `@specre` markers reference it in source code
2. User runs `specre drift`
3. The specre is reported as clean (no source files to compare against)

### Related files resolved from both Related Files section and @specre markers

1. A specre card lists `src/foo.rs` in its Related Files section
2. `src/bar.rs` contains a `// @specre <ULID>` marker for the same specre
3. Both `src/foo.rs` and `src/bar.rs` are checked for modifications
4. If either file was modified after `last_verified + grace`, the specre is reported as drifted

### JSON output

1. User runs `specre drift --json`
2. Output is valid JSON matching the schema:
   ```json
   {
     "drifted": [
       {
         "id": "01HZYPMZRK...",
         "name": "...",
         "path": "docs/specres/.../foo.md",
         "domain": "...",
         "last_verified": "2026-03-01",
         "changed_files": [
           { "file": "src/foo.rs", "last_modified": "2026-04-05", "diff_stat": "+12 -3" }
         ]
       }
     ],
     "clean": 42,
     "total": 45,
     "grace_days": 7
   }
   ```
3. Exit code is 1 when `drifted` is non-empty, 0 when empty

### Human-readable output (default)

1. User runs `specre drift` without `--json`
2. Output shows a summary line and a list of drifted specres with their changed files
3. Format:
   ```
   Drift: 3 drifted, 42 clean (grace: 7 days)

   01HZYPMZRK...  specre_name
     src/foo.rs  (modified: 2026-04-05, +12 -3)
   ```

### No specre.toml present

1. User runs `specre drift` in a directory without `specre.toml`
2. stderr shows: `Error: specre.toml not found. Run 'specre init' first.`
3. Exit code is 1

### Non-git repository

1. User runs `specre drift` in a directory that is not a git repository
2. stderr shows an error indicating that git history is required for drift detection
3. Exit code is 1

## Failures / Exceptions

- If `specre.toml` is missing, exit with error and message directing user to run `specre init`
- If the directory is not a git repository, exit with a clear error message (drift detection requires git history)
- If a file listed in Related Files does not exist on disk, skip it silently (it may have been deleted)
- If `git log` fails for a specific file, warn on stderr and skip that file
- If the `--grace` value has an invalid format, exit with an error describing the expected format (e.g., `0d`, `7d`, `30d`)
