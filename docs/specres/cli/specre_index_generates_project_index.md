---
id: "01KHAKAYN5WPTDVR99D5Q5TMJE"
name: "specre_index_generates_project_index"
status: "stable"
last_verified: "2026-02-16"
---

## Related Files

- `src/commands/index.rs`
- `src/config.rs`
- `tests/cli_index.rs` (Test)

## Functional Overview

`specre index` scans the specre directory and source tree, then generates `index.json` (a machine-readable cache) inside `specre_dir` and per-domain `_INDEX.md` files (human-readable summaries) in each domain directory. It reads `specre.toml` to determine the specre directory and source directories.

## Design Intent

The index command produces a derived artifact that other commands and external tools can consume for fast lookups without re-parsing every specre file. `index.json` is a cache — it can be regenerated at any time and should never be edited manually.

Per-domain `_INDEX.md` files provide a browsable overview of all specres in a domain, useful for both human developers and AI agents navigating the specification tree.

## Key Members

- `version: u32` — schema version of index.json (currently `1`)
- `generated_at: String` — RFC 3339 timestamp of when the index was generated
- `specres: Vec<SpecreEntry>` — array of specre entries extracted from front-matter
- `source_refs: Vec<SourceRef>` — array of `@specre` marker references found in source files

## Scenarios

### Basic invocation generates index.json

1. User runs `specre index` in a project with `specre.toml` and specre files
2. CLI reads `specre.toml` to determine `specre_dir` and `source_dirs`
3. CLI scans all `.md` files under `specre_dir` recursively, parsing YAML front-matter
4. CLI scans files under `source_dirs` recursively for `@specre <ULID>` markers (filtered by `target_extensions` if set)
5. CLI writes `index.json` inside `specre_dir` (e.g., `docs/specres/index.json`) with `version`, `generated_at`, `specres`, and `source_refs`
6. CLI prints a summary to stdout: `Generated <specre_dir>/index.json (N specres, M source refs)`

### specres array contains correct entries

1. Each specre `.md` file produces one entry in the `specres` array
2. Entry contains `id`, `name`, `status`, and `last_verified` from front-matter
3. Entry contains `domain` derived from the first directory level under `specre_dir` (e.g., `docs/specres/quotation/creation/foo.md` → `"quotation"`)
4. Entry contains `path` as the relative path from project root to the specre file using forward slashes

### source_refs array contains detected markers

1. A source file containing `// @specre 01HZYPMZRK8F9R2DGBGGMM2N8T` on line 3 produces an entry: `{ "specre_id": "01HZYPMZRK8F9R2DGBGGMM2N8T", "file": "src/example.rs", "line": 3 }`
2. A file with multiple `@specre` markers produces one entry per marker
3. The marker pattern is `@specre [0-9A-Z]{26}`, ignoring surrounding comment syntax
4. Markers inside string literals are ignored: if a quote character (`"` or `'`) appears before `@specre` on the same line, the marker is not detected
5. When `target_extensions` is set in `specre.toml`, only files whose extension matches the list are scanned. When unset, all files are scanned.

### Per-domain _INDEX.md is generated

1. For each domain directory (top-level directory under `specre_dir`), CLI generates an `_INDEX.md` in that domain directory
2. `_INDEX.md` contains a Markdown table with columns: Name, Status, Last Verified
3. Name column links to the specre file using a path relative to the domain directory (e.g., `[user_can_sign_up](signup/user_can_sign_up.md)` for a specre nested in a subdirectory)
4. `_INDEX.md` includes all specres within the domain, including those in subdirectories
5. CLI prints each generated `_INDEX.md` path to stdout

### Subdirectories within a domain are handled correctly

1. Given the following structure:
   ```
   docs/specres/auth/
     signup/user_can_sign_up.md
     password/user_can_reset_password.md
     system_rejects_expired_token.md
   ```
2. `specre index` produces one `_INDEX.md` at `docs/specres/auth/_INDEX.md`
3. All three specres appear in the table, with paths relative to the domain directory:
   - `signup/user_can_sign_up.md`
   - `password/user_can_reset_password.md`
   - `system_rejects_expired_token.md`
4. The `domain` field in `index.json` for all three is `"auth"`

### specre.toml does not exist

1. User runs `specre index` in a directory without `specre.toml`
2. CLI exits with error: `Error: specre.toml not found. Run 'specre init' first.`

### Empty specre directory

1. User runs `specre index` with a valid `specre.toml` but no specre files exist
2. CLI generates `index.json` with empty `specres` and `source_refs` arrays
3. No `_INDEX.md` files are generated

### Overwrites existing index files

1. User runs `specre index` when `index.json` and `_INDEX.md` already exist
2. CLI overwrites both files with fresh content
3. No error is raised

## Failures / Exceptions

- If `specre.toml` is missing, CLI exits with error: `Error: specre.toml not found. Run 'specre init' first.`
- If a specre file has malformed front-matter (missing `---` delimiters or required fields), CLI prints a warning to stderr and skips that file
- If a specre or source file cannot be read (IO error), CLI prints a warning to stderr (`Warning: failed to read '<path>': <reason>`) and skips that file
- If a directory cannot be read during traversal (permission denied, etc.), CLI prints a warning to stderr (`Warning: failed to read directory '<path>': <reason>`) and skips that directory
- If a directory entry cannot be read during traversal, CLI prints a warning to stderr and skips that entry
- If `specre_dir` does not exist, CLI treats it as empty (no specres found)
- If a `source_dirs` entry does not exist, CLI skips it silently
