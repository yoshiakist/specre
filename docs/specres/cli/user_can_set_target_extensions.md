---
id: "01KHFD5R1G3C5R34XMQXQTTMM9"
name: "user_can_set_target_extensions"
status: "stable"
last_verified: "2026-04-07"
---

## Related Files

- `src/config.rs`
- `src/commands/init.rs`
- `src/scanner.rs`
- `src/cli.rs`
- `tests/cli_init.rs` (Test)
- `tests/cli_index.rs` (Test)

## Functional Overview

Users can configure `target_extensions` in `specre.toml` to restrict which source files are scanned for `@specre` markers. When set, only files whose extension matches the list are scanned by `index`, `orphans`, and `trace`. When unset, all text files are scanned (binary files that fail UTF-8 decoding are always skipped silently). The `specre init` command accepts a `--ext` flag to configure this setting at project initialization.

Additionally, dot files (files whose names start with `.`) and `.svg` files are unconditionally excluded from scanning regardless of `target_extensions`.

## Design Intent

Without extension filtering, specre scans every text file in `source_dirs`. Binary files (images, fonts, lock files, and other non-UTF-8 artifacts) are always skipped silently — they cannot contain `@specre` markers and scanning them wastes time. The `target_extensions` setting lets users further restrict which text file types constitute their source code, improving both performance and accuracy.

This setting is project-level because the set of relevant source extensions is stable within a project (e.g., a Rust project always targets `.rs`). Making it a config value avoids repeating `--ext` on every command invocation. The setting is optional and defaults to scanning all text files, ensuring zero friction for new users and full backward compatibility for existing projects.

Beyond the binary-file check, some text files are clearly not source code. Dot files (`.keep`, `.gitignore`, `.env`, `.editorconfig`, etc.) are universally configuration or metadata — no project would place `@specre` markers in them. SVG files are XML-encoded vector graphics that pass UTF-8 decoding but are image assets. The scanner excludes both categories unconditionally, keeping the built-in exclusion list minimal and unambiguous.

## Key Members

- `Config.target_extensions: Option<Vec<String>>` — list of extensions (without leading dots) read from `specre.toml`. `None` means scan all files.
- `InitArgs.ext: Option<Vec<String>>` — CLI argument `--ext` for `specre init` (comma-separated, e.g., `--ext rs,ts,js`).
- `EXCLUDED_EXTENSIONS: &[&str]` — built-in list of extensions always excluded from scanning (currently: `["svg"]`).

## Scenarios

### Default behavior (no target_extensions setting)

1. User runs `specre init` without `--ext`
2. `specre.toml` is created without an active `target_extensions` field; a commented-out hint `# target_extensions = ["rb", "js", "ts"]` is appended for discoverability
3. User runs `specre index`
4. CLI reads `specre.toml`, finds no `target_extensions` field
5. CLI scans all text files in `source_dirs` for `@specre` markers; binary (non-UTF-8) files, dot files, and SVG files are skipped silently

### Init with target extensions

1. User runs `specre init --ext rs,ts`
2. `specre.toml` is created with `target_extensions = ["rs", "ts"]` as an active field in addition to `specre_dir` and `source_dirs`. Other optional settings (`exclude_patterns`, `language`, `[health_check]`) appear as commented-out hints.
3. User runs `specre index`
4. CLI reads `specre.toml`, finds `target_extensions = ["rs", "ts"]`
5. CLI scans only `.rs` and `.ts` files in `source_dirs` for `@specre` markers
6. Files with other extensions (e.g., `.json`, `.md`, `.lock`) are skipped

### Existing specre.toml without target_extensions (backward compatibility)

1. An existing project has `specre.toml` with only `specre_dir` and `source_dirs`
2. User updates specre binary to the new version
3. User runs `specre index`
4. CLI reads `specre.toml`, `target_extensions` field is missing
5. All text files in `source_dirs` are scanned; binary files, dot files, and SVG files are skipped silently

### Extensions are specified without leading dots

1. User runs `specre init --ext rs,ts`
2. `specre.toml` contains `target_extensions = ["rs", "ts"]` (not `[".rs", ".ts"]`)
3. CLI matches file extensions by comparing against these values without dots

### target_extensions applies to orphans command

1. User sets `target_extensions = ["rs"]` in `specre.toml`
2. A Python file `src/helper.py` contains `# @specre <ULID>` but no `.rs` files reference this ULID
3. User runs `specre orphans`
4. The marker in `helper.py` is not detected (`.py` is not in target_extensions)
5. The specre is reported as orphan because no marker was found in target files

### target_extensions applies to trace command (ULID mode)

1. User sets `target_extensions = ["rs"]` in `specre.toml`
2. A marker exists in `src/helper.py` (not a target extension) and in `src/main.rs` (target extension)
3. User runs `specre trace <ULID>`
4. Only `src/main.rs` appears in "Source references" — `src/helper.py` is skipped

### trace command in file-path mode is unaffected

1. User sets `target_extensions = ["rs"]` in `specre.toml`
2. User runs `specre trace src/helper.py`
3. CLI reads `src/helper.py` directly and shows its `@specre` markers
4. The `target_extensions` filter does not apply to the explicitly specified file

### Empty target_extensions array

1. User manually sets `target_extensions = []` in `specre.toml`
2. User runs `specre index`
3. No source files are scanned (empty filter matches nothing)
4. `source_refs` array in `index.json` is empty

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

## Failures / Exceptions

- If `specre.toml` exists but `target_extensions` contains invalid values (e.g., numbers), the TOML parser will reject the file — this is handled by the existing config parsing error path.
