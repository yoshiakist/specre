# CLAUDE.md — specre

## Rust Philosophy & Strictness
This project adheres to the most rigorous standards of Idiomatic Rust. We do not compromise on memory safety, ownership patterns, or performance for convenience.

- **The Compiler is King:** Treat every compiler warning as a fatal error. If the editor turns red, it is not a "suggestion" to ignore; it is a sign that the current implementation is flawed and unidiomatic.
- **Embrace the Friction:** Do not seek "easy" workarounds (like excessive `.clone()`, `Box<dyn T>`, or `unwrap()`). The friction provided by the borrow checker is a gift that ensures long-term stability.
- **Clippy is our Moral Compass:** This project uses `#![deny(clippy::pedantic, clippy::nursery, clippy::all)]`. Your goal is to produce code that passes these checks without suppressions (e.g., no `#[allow(...)]` without a profound architectural justification).
- **Atonement for Errors:** When code fails to compile or violates Clippy rules, view it as a failure of logic. Apologize only through superior, idiomatic refactoring. Never suggest that the user "ignore" or "downgrade" these strict settings.

リファクタやコード品質改善タスクにあたっては、[docs/guides/RUST-CONVENTIONS.md](docs/guides/RUST-CONVENTIONS.md) の具体的なルールを参照せよ。

## What is specre?

Atomic, living specification cards for AI-agent-friendly development. A Rust CLI toolkit for Spec-Driven Development (SDD). Each specre is a single Markdown file describing exactly one behavior, with YAML front-matter for lifecycle tracking and bidirectional traceability.

**Core philosophy:** One specre card = one behavior. Sized for a single LLM context window.

## Project Structure

```
specre/
├── Cargo.toml               # Rust package manifest (v0.2.6, edition 2024)
├── specre.toml              # Project config (created by `specre init`)
├── src/
│   ├── main.rs              # Entry point (clap CLI routing)
│   ├── cli.rs               # CLI argument definitions (clap derive)
│   ├── config.rs            # specre.toml parser (serde + toml)
│   ├── error.rs             # Error types and contextual diagnostics
│   ├── status.rs            # Status enum and lifecycle logic
│   ├── template.rs          # specre card template generation
│   ├── ulid.rs              # ULID generation wrapper
│   └── commands/
│       ├── mod.rs
│       ├── init.rs          # `specre init` implementation
│       ├── new.rs           # `specre new` implementation
│       ├── index.rs         # `specre index` implementation
│       ├── status.rs        # `specre status` implementation
│       ├── trace.rs         # `specre trace` implementation
│       ├── orphans.rs       # `specre orphans` implementation
│       ├── tag.rs           # `specre tag` implementation
│       ├── search.rs        # `specre search` implementation
│       ├── coverage.rs      # `specre coverage` implementation
│       ├── health_check.rs  # `specre health-check` implementation
│       └── mcp.rs           # `specre mcp` MCP server implementation
├── tests/                   # Integration tests (assert_cmd)
│   ├── cli_init.rs          # ... and cli_new, cli_index, cli_status,
│   ├── cli_trace.rs         #     cli_orphans, cli_tag, cli_search,
│   ├── cli_coverage.rs      #     cli_health_check, cli_dispatch,
│   ├── cli_json_output.rs   #     cli_json_output
│   └── mcp/                 # MCP server integration tests
└── docs/specres/            # specre cards for this project itself
    ├── cli/
    └── mcp/
```

## Tech Stack

- **Language:** Rust (2024 edition)
- **CLI framework:** clap v4 (derive)
- **MCP server:** rmcp (stdio transport), tokio, schemars, tracing
- **Date handling:** chrono
- **Test framework:** assert_cmd + assert_fs + predicates (integration tests)
- **Build:** `cargo build` / `cargo test`

## CLI Commands

| Command | Status | Description |
|---------|--------|-------------|
| `specre init` | Implemented | Initialize project (creates `specre.toml` and specre directory) |
| `specre new <dir> --name <name>` | Implemented | Scaffold a new specre card with auto-generated ULID |
| `specre index` | Implemented | Generate `index.json` and per-domain `_INDEX.md` |
| `specre status` | Implemented | Report specre counts by status, flag stale `last_verified` |
| `specre trace` | Implemented | Bidirectional traceability lookup by ULID or file path |
| `specre orphans` | Implemented | Detect unlinked specres or dangling markers |
| `specre tag` | Implemented | Insert `@specre` markers into source files |
| `specre search` | Implemented | Full-text + filter search with JSON output |
| `specre coverage` | Implemented | Report percentage of source files covered by `@specre` tags |
| `specre health-check` | Implemented | Comprehensive health check for AI agent preflight |
| `specre mcp` | Implemented | Start MCP server (stdio transport) |
| `specre drift` | Not yet | Detect drift between spec and implementation |
| `specre ci` | Not yet | CI integration (non-zero exit on drift/orphans) |

## Development Workflow

### Implementing a new specre CLI command

1. **Create the specre card first** — Run `specre new docs/specres/<domain> --name <behavior_name>` to scaffold the specification. Fill in the specre card (Functional Overview, Scenarios, etc.) **before** writing code.
2. **Write tests first (specre-first)** — Based on the scenarios in the specre card, write integration tests in `tests/`. Use `assert_cmd` for CLI testing.
3. **Implement** — Write the minimal code to make the tests pass. Add the command to `cli.rs` (clap derive) and create the handler in `src/commands/`.
4. **Update the specre card** — Once all tests pass, set `status` to `"stable"` and `last_verified` to today's date (`YYYY-MM-DD`). Tests passing is sufficient for the status transition; the maintainer verifies behavior before merging.

### When modifying existing behavior

1. Read the relevant specre card in `docs/specres/` first.
2. Update the specre card to reflect the new behavior.
3. Update tests, then implementation.
4. Update `last_verified` to today's date.

## MCP Preflight: Health-Check First

When specre MCP tools are available in the agent's tool list, **always run `health-check` before any other specre operation.** This single call determines the exploration strategy for the entire session.

### healthy = true

The specre ecosystem — specification cards, traceability links, index, and coverage — is in a trustworthy state. Leverage it aggressively:

- **Use `specre search`** to locate relevant specre cards. The most effective pattern is an AND query combining a **noun** (the subject) and an **operation** (what it does):
  - `specre search "trace bidirectional"` — finds the traceability specre card
  - `specre search "orphan detect"` — finds the orphan detection specre card
  - `specre search "mcp server"` — finds MCP-related specre cards
  - Add `--or` only when the AND query is too narrow
- **Use `specre trace`** to navigate between specre cards, source files, and test files:
  - `specre trace <ULID>` — from specre card to all linked source/test files
  - `specre trace <file-path>` — from source file to governing specre card(s)
- **Trust the specre cards** as the authoritative description of each behavior. Read the card's Scenarios and Related Files before modifying code.

### healthy = false

The specre ecosystem has gaps (low coverage, orphans, or stale index). Do **not** rely on specre tools for navigation — results may be incomplete or misleading. Instead:

- Fall back to `grep` / `glob` for code exploration
- Read source files and tests directly
- Treat specre cards as reference material, not as the single source of truth

### Why this matters

A healthy specre ecosystem means every behavior has a specification, every source file is traced, and the index is current. An agent that trusts this ecosystem can find the right files in 1-2 tool calls instead of broad codebase searches. When the ecosystem is unhealthy, that trust is misplaced — and grep is more reliable.

## specre Card Format

Every specre card follows this structure:

```markdown
---
id: "01HZYPMZRK8F9R2DGBGGMM2N8T"    # ULID (auto-generated by `specre new`)
name: "user_can_sign_up_with_email"    # Matches filename, subject+predicate
status: "draft"                         # draft | in-development | stable | deprecated
last_verified: "2026-03-01"            # YYYY-MM-DD (required for stable)
---

## Related Files
## Functional Overview
## Scenarios
```

**Status lifecycle:** `draft` → `in-development` → `stable` → `deprecated`

**Naming convention:** Always use a sentence with subject + predicate: `user_can_reset_password`, `system_rejects_expired_token`. Never use function-style nouns like `create_quotation`.

## Conventions

- **Testing:** All CLI commands are tested via integration tests in `tests/` using `assert_cmd`. Tests run as subprocesses against the built binary.
- **Error handling:** Commands return `Result<(), Box<dyn Error>>`. Errors print to stderr with `Error:` prefix and exit code 1.
- **Output:** CLI output to stdout should be machine-parseable. No interactive prompts.
- **ULID:** 26-character identifiers generated via the `ulid` crate. Used as the single key for bidirectional traceability.
- **Traceability:** Source files reference specres via `// @specre <ULID>` comments. specre cards reference source files in the "Related Files" section.

## Pre-PR Quality Gate

Before creating a Pull Request (or pushing commits intended for PR), **always** run the following checks and fix any issues. These mirror the CI pipeline exactly — if they pass locally, CI will pass.

```bash
cargo fmt --all                            # Auto-format all code
cargo clippy --all-targets -- -D warnings  # Lint all targets (lib, bins, tests, examples, benches)
cargo test                                 # Run all tests
```

**Order matters:** Run `cargo fmt` first (formatting changes may affect clippy results), then `cargo clippy`, then `cargo test`. All three must pass with zero warnings before committing.

- `cargo clippy --all-targets` ensures clippy checks not only `src/` but also integration tests in `tests/`, examples, and benchmarks. Omitting `--all-targets` can let lint violations in test code slip through.
- `cargo fmt --all` formats the entire workspace. CI runs `cargo fmt --all -- --check` and will reject unformatted code.

## Running Tests

```bash
cargo test              # Run all tests
cargo test --test cli_init  # Run specific test file
```
