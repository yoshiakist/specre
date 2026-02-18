# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.2.6] - 2026-02-15

### Added

- `--json` global flag — switches CLI output from human-readable text to structured JSON on stdout; enables AI agents and scripts to consume specre output programmatically; affects `status`, `trace`, `orphans`, `coverage`, `init`, `new`, `tag`, `index` (`search` and `health-check` already output JSON)

### Fixed

- `specre index`: `index.json` is now generated inside `specre_dir` (e.g. `docs/specres/index.json`) instead of the project root, avoiding naming conflicts with other frameworks
- `specre index`: per-domain index files renamed from `INDEX.md` to `_INDEX.md` so they sort to the top of directory listings in file explorers

## [0.2.5] - 2026-02-15

### Added

- `specre search <query>` — full-text search across specre cards with structured filters: `--status`, `--domain`, `--verified-before`, `--verified-after`; multi-keyword AND by default, `--or` for OR logic; configurable truncation threshold to protect agent context budgets; outputs JSON

## [0.2.4] - 2026-02-15

### Added

- `specre health-check` — comprehensive AI agent preflight command; aggregates coverage ratio, orphan count, and index freshness into a single JSON response so agents can verify specre ecosystem trustworthiness in one call
- `specre coverage` — report the percentage of source files tagged with `@specre` markers; lists uncovered files alphabetically; `--ext` flag overrides `target_extensions` from `specre.toml`
- `target_extensions` config field in `specre.toml` — filter which file extensions are scanned for `@specre` markers, excluding binary files and non-source artifacts; `specre init --ext <ext,...>` writes the setting; affects `index`, `orphans`, and `trace`

### Changed

- Refactored: deduplicated ULID validation logic into a shared `ulid::is_valid` function (previously duplicated in `trace.rs` and `tag.rs`)

## [0.2.3] - 2026-02-14

### Added

- `specre tag`: expanded comment syntax support to 80+ file extensions — game engines (Unity C#, Unreal, Godot GDScript), web frameworks (Django, Rails, Laravel, ASP.NET), frontend frameworks (Vue, Svelte, Astro), and template engines (Jinja2, Twig, ERB, EJS, Handlebars, Razor, Pug, Haml)

### Changed

- `specre tag`: unsupported file extensions now return an error instead of silently falling back to `//` comments, preventing accidental file corruption

## [0.2.2] - 2026-02-14

### Added

- `specre init --language <lang>` — write a `language` setting to `specre.toml`; `specre new` reads it and generates localized section headings in specre card templates (supported: `ja` for Japanese; defaults to English when unset)

## [0.2.1] - 2026-02-14

### Added

- Binary releases via `cargo-dist` — pre-built binaries for Linux (x86\_64, aarch64), macOS (x86\_64, aarch64), and Windows (x86\_64); shell and PowerShell installer scripts

## [0.2.0] - 2026-02-13

### Added

- `specre trace <ULID|path>` — Bidirectional traceability lookup; accepts a ULID (shows the specre card and all referencing source files) or a file path (reads `@specre` markers and resolves each to its specre card)
- `specre orphans` — Detect unlinked specres (no `@specre` markers pointing to them) and dangling markers (no matching specre card); exits with code 1 when issues are found, making it CI-friendly; deprecated specres are excluded
- `specre tag <ULID> <file>` — Insert a `@specre <ULID>` marker at line 1 using language-appropriate comment syntax (`//`, `#`, `/* */`, `<!-- -->`, `--`); idempotent — skips insertion if the marker already exists

### Fixed

- `index`, `trace`, `orphans`: skip `@specre` markers that appear inside string literals (preceded by `"` or `'`), preventing false positives from test fixtures and documentation examples
- `trace`: normalize backslash path separators to forward slashes so the command works correctly when users supply Windows-style paths (e.g. `src\config.rs`)

## [0.1.0] - 2026-02-13

### Added

- `specre init` — Initialize a project with `specre.toml` and a specre directory
- `specre new <dir> --name <name>` — Scaffold a new specre card with auto-generated ULID
- `specre index` — Scan specre directory and source tree; generate `index.json` and per-domain `INDEX.md`
- `specre status` — Report specre counts by status and flag stale `last_verified` dates
- Bidirectional traceability via `@specre <ULID>` source markers
- specre card format with YAML front-matter (`id`, `name`, `status`, `last_verified`)

[0.2.6]: https://github.com/yoshiakist/specre/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/yoshiakist/specre/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/yoshiakist/specre/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/yoshiakist/specre/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/yoshiakist/specre/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/yoshiakist/specre/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/yoshiakist/specre/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yoshiakist/specre/releases/tag/v0.1.0
