---
id: "01KHDF9WHR5HFM4RQCF6HS3KCC"
name: "user_can_set_language_config"
status: "stable"
last_verified: "2026-02-14"
---

## Related Files

- `src/config.rs`
- `src/template.rs`
- `src/commands/new.rs`
- `src/commands/init.rs`
- `src/cli.rs`
- `tests/cli_new.rs` (Test)
- `tests/cli_init.rs` (Test)

## Functional Overview

`specre new` generates a localized specre card template based on the `language` setting in `specre.toml`. When `language` is set to `"ja"`, the template's section headings are rendered in Japanese. The `specre init` command accepts a `--language` flag to configure this setting. If `specre.toml` is absent or `language` is not set, the default language is `"en"` (English).

## Design Intent

specre is designed to be used by AI coding agents and developers across different language backgrounds. When a developer primarily works in Japanese, having specification section headings in Japanese reduces cognitive overhead and makes specre cards more natural to read and write. This is especially important when specre cards are consumed by AI agents that operate in the user's native language — the section names serve as semantic anchors for both humans and agents.

The implementation supports only `"en"` and `"ja"` initially. Unknown language values fall back to `"en"`.

## Key Members

- `Config.language: Option<String>` — language code read from `specre.toml` (e.g., `"en"`, `"ja"`). Defaults to `"en"` when absent.
- `InitArgs.language: String` — CLI argument `--language` for `specre init` (default: `"en"`).
- `template::render(id, name, language)` — accepts a language parameter to select the template variant.

## Scenarios

### Default behavior (no language setting)

1. User runs `specre init` without `--language`
2. `specre.toml` is created without a `language` field (backward compatible)
3. User runs `specre new docs/specres/cli --name some_behavior`
4. CLI reads `specre.toml`, finds no `language` field, defaults to `"en"`
5. Generated template has English section headings: `## Related Files`, `## Functional Overview`, `## Scenarios`

### Init with Japanese language

1. User runs `specre init --language ja`
2. `specre.toml` is created with `language = "ja"` in addition to `specre_dir` and `source_dirs`
3. User runs `specre new docs/specres/cli --name some_behavior`
4. CLI reads `specre.toml`, finds `language = "ja"`
5. Generated template has Japanese section headings: `## 関連ファイル`, `## 機能概要`, `## シナリオ`

### specre.toml does not exist

1. User runs `specre new some_dir --name foo` without having run `specre init`
2. CLI does not find `specre.toml`
3. CLI falls back to the default English template
4. File is created successfully (no error)

### Unknown language falls back to English

1. User manually edits `specre.toml` and sets `language = "fr"`
2. User runs `specre new docs/specres/cli --name some_behavior`
3. CLI reads `language = "fr"`, does not recognize it
4. CLI falls back to the English template

### Japanese template content

The Japanese template contains the same structure as the English template, with translated section headings:

```markdown
---
id: "<generated ULID>"
name: "<provided or 'untitled'>"
status: "draft"
---

## 関連ファイル

-

## 機能概要



## シナリオ

###

1.
```

### Backward compatibility

1. An existing project has `specre.toml` with only `specre_dir` and `source_dirs` (no `language` field)
2. User updates specre binary to the new version
3. User runs `specre new docs/specres/cli --name some_behavior`
4. CLI reads `specre.toml`, `language` field is missing
5. Template is generated in English — same behavior as before

## Failures / Exceptions

- If `specre.toml` exists but cannot be parsed (e.g., invalid TOML syntax), `specre new` should still work by falling back to the English template and printing a warning to stderr.
