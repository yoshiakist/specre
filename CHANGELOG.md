# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.5.0] - 2026-04-07

### Added

- `specre drift` — detect specification-implementation divergence by comparing `last_verified` dates against source file modification times; configurable `grace_days` in `specre.toml`
- `specre verify` — bulk-update `last_verified` dates for specre cards, streamlining the drift triage workflow
- `specre reopen` — transition stable specre cards back to in-development status when re-work is needed
- `specre health-check`: now includes drifts count in the health report, giving AI agents a more complete preflight picture
- `specre init`: generated `specre.toml` now includes drift configuration hints (e.g. `grace_days`) as commented-out defaults
- CI integration guide and health-check step added to CI workflow documentation

### Changed

- Drift triage workflow added as Phase 5 to the SDD workflow documentation
- Refactored test helpers: extracted `write_config_with_exclude` into shared test utilities

## [0.4.0] - 2026-03-05

### Added

- `specre destroy` — new command to remove `@specre` markers from source files; accepts a ULID or file path and strips all matching markers, making it easy to unlink a specre card from source code
- `exclude_patterns` config field in `specre.toml` — glob patterns to filter paths from source scanning; affects `index`, `orphans`, `coverage`, and `trace`
- `specre init`: generated `specre.toml` now includes commented-out default options (e.g. `exclude_patterns`) as inline documentation

## [0.3.2] - 2026-02-22

### Fixed

- `specre health-check`: use content comparison instead of timestamp for index freshness detection, eliminating false "stale index" warnings
- Source scanner: exclude dot files (e.g. `.gitignore`) and SVG files from scanning, reducing noise in `trace`, `orphans`, `coverage`, and `index` results

### Changed

- CI: skip workflow runs for non-code changes (docs, markdown, config) and add branch protection gate job

## [0.3.1] - 2026-02-22

### Fixed

- `specre coverage`: truncate the uncovered files list to 30 items, preventing excessively long output in large projects
- Source scanner: silently skip binary (non-UTF-8) files instead of erroring, improving reliability of `trace`, `orphans`, `coverage`, and `index` commands

## [0.3.0] - 2026-02-19

### Added

- `specre mcp` — start an MCP server (stdio transport) that exposes all specre cards as resources and provides CLI-equivalent tools for AI agents; enables editors and agent runtimes to consume specre without shell access
- MCP tools: `new`, `tag`, `index`, `status`, `trace`, `orphans`, `search`, `coverage`, `health-check` — full parity with the CLI; AI agents can now run the entire specre workflow without invoking a subprocess
- `specre search`: multi-keyword AND/OR query — space-separated terms are ANDed by default; `--or` switches to OR logic, matching any single term
- `specre search`: glossary-based hint suggestions — when a query returns few results, suggests semantically related terms from `glossary.toml` to guide follow-up queries
- Cross-platform CI — automated test workflow for Linux (x86\_64), macOS (x86\_64, aarch64), and Windows (x86\_64)

### Fixed

- `specre init`: prevented TOML injection — user-controlled values (project name, specre dir, language) are now properly escaped before being written to `specre.toml`
- Front-matter parsing: replaced the hand-written YAML parser with `serde_yml` for correctness and safety
- `specre search`: date filter now validates calendar correctness (e.g. `2026-02-30` is rejected)
- IO errors previously swallowed with `.ok()?` in `search`, `index`, `orphans`, `trace`, `coverage`, `health-check`, and `mcp` are now surfaced as warnings instead of being silently discarded
- Permission-based tests are now skipped when running as root, fixing CI failures in privileged container environments

### Changed

- Strict Clippy lints enabled project-wide (`pedantic + nursery + all = deny`) — zero compiler-warning policy enforced in CI
- Migrated from archived `serde_yaml` to maintained `serde_yml`

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

[0.5.0]: https://github.com/yoshiakist/specre/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/yoshiakist/specre/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/yoshiakist/specre/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/yoshiakist/specre/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/yoshiakist/specre/compare/v0.2.6...v0.3.0
[0.2.6]: https://github.com/yoshiakist/specre/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/yoshiakist/specre/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/yoshiakist/specre/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/yoshiakist/specre/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/yoshiakist/specre/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/yoshiakist/specre/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/yoshiakist/specre/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yoshiakist/specre/releases/tag/v0.1.0
