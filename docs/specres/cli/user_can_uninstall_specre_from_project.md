---
id: "01KJYMAV7G01B743W72WAG9RGN"
name: "user_can_uninstall_specre_from_project"
status: "stable"
last_verified: "2026-03-05"
---

## Related Files

- `src/commands/destroy.rs` — command handler
- `src/commands/mod.rs` — module registration
- `src/cli.rs` — CLI argument definition
- `src/main.rs` — command dispatch
- `tests/cli_destroy.rs` — integration tests

## Functional Overview

`specre destroy` removes all specre artifacts from the current project so that the user can cleanly stop using specre. It strips every `@specre` marker comment from all source files tracked by `specre.toml`, deletes `specre.toml` and `glossary.toml`, and optionally deletes the specre cards directory. After completing these steps it instructs the user to run `cargo uninstall specre` to remove the binary itself.

## Design Intent

Users who want to stop using specre need a clean exit path. Without a dedicated command, manually hunting down every `@specre` comment and config file is error-prone and tedious. `specre destroy` makes the offboarding experience deterministic, leaving no ghost markers or config files behind. The command is intentionally interactive (no `--yes` bypass) because it performs irreversible filesystem mutations.

## Key Members

- `execute() -> Result<(), SpecreError>` — top-level handler; no args struct needed
- `remove_markers_from_file(path: &Path) -> Result<usize, SpecreError>` — strips marker lines from a single file, returning the count removed; rewrites only when at least one line was removed. A line qualifies as a marker line if and only if **all three** conditions hold: (1) it has no leading whitespace (starts at column 0), (2) `parser::extract_marker_ulid(line)` returns `Some(_)` (valid 26-char ULID, not preceded by a quote character), and (3) the prefix text between the start of the line and `@specre `, when trimmed, contains no embedded space — ensuring `specre tag`-style comment prefixes (`//`, `#`, `/*`, `<!--`, etc.) are accepted while prose comments like `# See @specre …` are rejected.
- `prompt_yes_no(question: &str) -> bool` — reads a single `[y/N]` answer from stdin; empty input or `n`/`N` returns `false`

## Scenarios

### Scenario 1: Happy path — keep specre cards directory

Given a project with `specre.toml` referencing `source_dirs = ["src"]` and source files containing `@specre` markers:

1. The user runs `specre destroy`.
2. The command prints:
   ```
   This will remove all @specre markers from your source files and delete specre.toml.
   ```
3. It prompts:
   ```
   Also delete the specre cards directory 'docs/specres'? [y/N]:
   ```
   The default is **N**.
4. The user presses Enter (or types `n`).
5. The command scans all source files in all configured `source_dirs` and removes every line that is a specre marker line (all three: starts at column 0, `parser::extract_marker_ulid` returns `Some(_)`, and the prefix before `@specre ` has no embedded space), rewriting each file only when at least one marker line was removed.
6. `specre.toml` is deleted.
7. `glossary.toml` is deleted if it exists.
8. The specre cards directory is **not** deleted.
9. The command prints a summary of modified source files and deleted config files, then prints:
   ```
   Done. To remove the specre binary, run: cargo uninstall specre
   ```

### Scenario 2: Happy path — also delete specre cards directory

Same as Scenario 1, but the user types `y` at the prompt. After stripping markers and deleting config files, the command recursively deletes the entire specre cards directory (including any `index.json`, `_INDEX.md`, and all card files within it) and reports the deletion in the summary.

### Scenario 3: No specre.toml — error

Given a directory with no `specre.toml`:

1. The user runs `specre destroy`.
2. The command fails with exit code 1 and prints to stderr:
   ```
   Error: specre.toml not found. …
   ```

### Scenario 4: Source file with no marker lines — left unchanged

A source file that contains no qualifying specre marker lines is read but not rewritten, and is not mentioned in the output summary.

### Scenario 5: Source file with multiple markers — all removed

A source file containing two or more `@specre` marker lines has every such line removed in a single rewrite. The removed count reflects the total lines stripped from that file.

### Scenario 6: glossary.toml does not exist — silently skipped

If `glossary.toml` is absent the command proceeds without error and does not mention it in the summary.

### Scenario 7: Line contains `@specre` but is not a marker — preserved

A line that mentions `@specre` but does **not** qualify as a marker line must be left untouched. The following lines are **not** removed:

- **Indented lines**: `    // @specre 01AAAAAAAAAAAAAAAAAAAAAAAA` — has leading whitespace (condition 1 fails).
- **String literals**: `let s = "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA";` — `@specre` is preceded by a quote character, so `extract_marker_ulid` returns `None` (condition 2 fails).
- **Prose comments**: `# See @specre 01AAAAAAAAAAAAAAAAAAAAAAAA for details` — starts at column 0 and `extract_marker_ulid` returns `Some(_)`, but the prefix before `@specre ` is `# See ` which contains an embedded space when trimmed (condition 3 fails).

## Failures / Exceptions

- **Config not found** — exits with `SpecreError::ConfigNotFound` (same as other commands calling `config::load()`).
- **I/O error reading source file** — prints a warning to stderr (`Warning: failed to read '<path>': <cause>`) and continues with the remaining files.
- **I/O error writing source file** — returns `SpecreError::Io` and aborts.
- **I/O error deleting specre.toml** — returns `SpecreError::Io` and aborts.
- **I/O error deleting specre cards directory** — returns `SpecreError::Io` and aborts.
