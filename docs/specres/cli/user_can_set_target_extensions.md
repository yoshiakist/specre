---
id: "01KHFD5R1G3C5R34XMQXQTTMM9"
name: "user_can_set_target_extensions"
status: "stable"
last_verified: "2026-02-17"
---

## Related Files

- `src/config.rs`
- `src/commands/init.rs`
- `src/scanner.rs`
- `src/cli.rs`
- `tests/cli_init.rs` (Test)
- `tests/cli_index.rs` (Test)

## Functional Overview

Users can configure `target_extensions` in `specre.toml` to restrict which source files are scanned for `@specre` markers. When set, only files whose extension matches the list are scanned by `index`, `orphans`, and `trace`. When unset, all files are scanned (preserving backward compatibility). The `specre init` command accepts a `--ext` flag to configure this setting at project initialization.

## Design Intent

Without extension filtering, specre scans every file in `source_dirs` — including binary files, images, lock files, and other non-source artifacts. This wastes time and can produce false positives if a binary file happens to contain a byte sequence matching the `@specre` pattern. The `target_extensions` setting lets users declare which file types constitute their source code, improving both performance and accuracy.

This setting is project-level because the set of relevant source extensions is stable within a project (e.g., a Rust project always targets `.rs`). Making it a config value avoids repeating `--ext` on every command invocation. The setting is optional and defaults to scanning all files, ensuring zero friction for new users and full backward compatibility for existing projects.

## Key Members

- `Config.target_extensions: Option<Vec<String>>` — list of extensions (without leading dots) read from `specre.toml`. `None` means scan all files.
- `InitArgs.ext: Option<Vec<String>>` — CLI argument `--ext` for `specre init` (comma-separated, e.g., `--ext rs,ts,js`).

## Scenarios

### Default behavior (no target_extensions setting)

1. User runs `specre init` without `--ext`
2. `specre.toml` is created without a `target_extensions` field
3. User runs `specre index`
4. CLI reads `specre.toml`, finds no `target_extensions` field
5. CLI scans all files in `source_dirs` for `@specre` markers (same as current behavior)

### Init with target extensions

1. User runs `specre init --ext rs,ts`
2. `specre.toml` is created with `target_extensions = ["rs", "ts"]` in addition to `specre_dir` and `source_dirs`
3. User runs `specre index`
4. CLI reads `specre.toml`, finds `target_extensions = ["rs", "ts"]`
5. CLI scans only `.rs` and `.ts` files in `source_dirs` for `@specre` markers
6. Files with other extensions (e.g., `.json`, `.md`, `.lock`) are skipped

### Existing specre.toml without target_extensions (backward compatibility)

1. An existing project has `specre.toml` with only `specre_dir` and `source_dirs`
2. User updates specre binary to the new version
3. User runs `specre index`
4. CLI reads `specre.toml`, `target_extensions` field is missing
5. All files in `source_dirs` are scanned — same behavior as before

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

## Failures / Exceptions

- If `specre.toml` exists but `target_extensions` contains invalid values (e.g., numbers), the TOML parser will reject the file — this is handled by the existing config parsing error path.
