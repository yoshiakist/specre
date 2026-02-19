[English](README.md) | [日本語](README.ja.md)

# specre

**Atomic, living specification cards for AI-agent-friendly development.**

specre ( /spékré/ ) is a minimal specification format and toolkit for Spec-Driven Development (SDD). Each specre is a single Markdown file describing exactly one behavior, with machine-readable front-matter for lifecycle tracking and agent navigation.

## The Problem

Specifications are essential for keeping development intent visible and traceable. But in practice, they rot:

- **Specs drift from code in silence.** No one notices when an implementation diverges from its specification — until the next developer (or AI agent) builds on stale assumptions.
- **Monolithic specs waste AI context.** Large specification documents force agents to parse entire features just to understand a single behavior, consuming the finite context window that should be reserved for code and tests.
- **Small changes never get specced.** When the cost of writing a specification is high, only greenfield features get documented. Bug fixes, refactors, and incremental changes slip through unspecified.

specre exists to make specifications so small and so cheap to write that there is no reason to skip them.

## Philosophy

Most SDD tools (GitHub Spec Kit, Amazon Kiro, BMAD) treat specifications as large, monolithic documents that feed into linear pipelines. specre takes the opposite approach:

- **One file, one behavior.** Keep the unit of specification atomic. An AI agent should never need to parse an entire feature to understand a single behavior.
- **Context-window aware.** LLM context is a finite resource. A specre is sized to fit comfortably alongside its test file and implementation in a single agent session.
- **Living documents, not session artifacts.** Each specre carries its own lifecycle status and verification date. Specifications survive beyond the task that created them.
- **Process-agnostic data layer.** specre defines the *format* of specifications, not the *workflow*. Use it with TDD, BDD, or any development process. Pair it with your own workflow commands or CI pipelines.
- **Engine and tool independent.** Plain Markdown with YAML front-matter. No IDE lock-in, no proprietary CLI required.

*Specificatio credibilis crescere potest.*

## Quick Start

### Install Pre-built Binary

```bash
# Linux / macOS
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/yoshiakist/specre/releases/latest/download/specre-installer.sh | sh

# Windows (PowerShell)
powershell -ExecutionPolicy ByPass -c "irm https://github.com/yoshiakist/specre/releases/latest/download/specre-installer.ps1 | iex"
```

### Install via Cargo (Rust users)

```bash
cargo install specre
```

This installs the `specre` CLI from [crates.io](https://crates.io/crates/specre). Requires Rust 1.85+.

### As a Git Submodule

```bash
git submodule add git@github.com:yoshiakist/specre.git specre
```

### Adopting specre in an Existing Codebase

If you are retrofitting specre into a project that already has code and tests, see the **[Adoption Strategy Guide](docs/guides/adoption-strategy.md)**. It walks you through assessing your test landscape and choosing the right strategy (test-derived extraction, code-behavior analysis, or top-down domain decomposition).

### Writing Your First specre

Create a Markdown file under your project's specres directory. Organize subdirectories however you like — by domain, module, feature area, or any scheme that fits your project:

```
docs/specres/
  auth/
    signup/
      user_can_sign_up_with_email.md
      system_sends_verification_email_on_signup.md
    password/
      user_can_reset_password.md
  cart/
    user_can_add_item_to_cart.md
    cart_total_reflects_quantity_changes.md
```

Subdirectories within a domain are optional — use them when related behaviors benefit from grouping, but keep the tree flat where a single level is sufficient.

## specre Card Format

Every specre follows this structure:

```markdown
---
id: "01HZYPMZRK8F9R2DGBGGMM2N8T"
name: "user_can_sign_up_with_email"
status: "draft"
---

## Related Files

- `src/auth/signup_controller.rb`
- `src/auth/email_validator.rb`
- `spec/auth/signup_controller_spec.rb` (Test)

## Functional Overview

Users can create an account by providing a valid email address and password.

## Design Intent

Email signup is the primary onboarding path. We validate email format
client-side and uniqueness server-side to provide fast feedback.

## Key Members

- `email: String` — the user's email address, validated against RFC 5322
- `password_hash: String` — bcrypt hash, never stored in plaintext

## Scenarios

### Successful signup

1. User submits a valid email and a password of 8+ characters
2. System creates the account and emits `account_created` signal
3. User is redirected to the welcome screen

### Duplicate email

1. User submits an email that already exists
2. System displays an error: "This email is already registered"
3. No account is created

### Invalid email format

1. User submits a malformed email (e.g., "foo@")
2. System displays a validation error before submission

## Failures / Exceptions

- If the database is unreachable, display a generic error and log the incident
```

### Front-Matter Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | ULID | Yes | Universally unique identifier in [ULID](https://github.com/ulid/spec) format. The single key for bidirectional traceability between specres and source code. |
| `name` | string | Yes | Matches the filename (without `.md`). A sentence with a clear subject and predicate that describes the behavior (e.g., `user_can_sign_up_with_email`, `system_rejects_invalid_email`). Avoid function-style nouns like `create_quotation`. |
| `status` | enum | Yes | `draft` · `in-development` · `stable` · `deprecated` |
| `last_verified` | date | No | `YYYY-MM-DD` — when this specre was last confirmed to match the implementation. Applicable to `stable` specres. Not required for `draft` or `in-development`. |

### Status Lifecycle

```
draft ──→ in-development ──→ stable ──→ deprecated
  ↑            │                │
  └────────────┘                │
  (requirements changed)        │
                                ↓
                           (superseded or removed)
```

- **draft**: Behavior is proposed but not yet implemented.
- **in-development**: Implementation or tests are in progress.
- **stable**: Implementation matches the specre. Tests pass. Verified by `last_verified` date.
- **deprecated**: Behavior has been removed or superseded. Kept for historical reference.

Status records the current state of a specre, not a prescribed workflow. Projects may skip states, move backwards, or adopt only a subset — specre does not enforce transitions.

### Recommended Sections

| Section | Required | Purpose |
|---------|----------|---------|
| Related Files | Yes | Links specre to source and test files by path (human-readable) |
| Functional Overview | Yes | One-paragraph summary of the behavior |
| Design Intent | No | Explains *why*, not *what* |
| Key Members | No | Important state and parameters in natural language |
| Scenarios | Yes | Step-by-step behavior descriptions |
| Failures / Exceptions | No | Edge cases and error handling |

### Naming Conventions

- **Name as a sentence.** Every specre name must have a clear subject and predicate describing the behavior: `user_can_reset_password`, `system_rejects_expired_token`, `cart_total_reflects_quantity_changes`. Avoid function-style nouns like `create_quotation` or `password_reset`.
- **No sequential numbering.** Do not prefix filenames with `001_`, `002_`, etc. The specre's ULID already provides chronological ordering. Sequential numbers create management overhead and friction with AI agents.
- **Use subdirectories for grouping.** When a domain contains many related behaviors, group them with subdirectories instead of numbering. Prefer `quotation/approval/manager_can_approve_quotation.md` over `quotation/201_manager_can_approve_quotation.md`. Keep the tree flat where a single level is sufficient.

### Writing Guidelines

- Write scenarios in **natural language**, not code. Avoid copy-pasting implementation details.
- **Do** use exact names for signals, classes, and enums — these are the contract between spec and code.
- Keep each specre **focused on a single behavior**. If you find yourself writing "also, ..." — split it into a separate specre.
- specres can reference each other via relative paths in a "Referenced Specifications" section.

## Bidirectional Traceability

specre uses a single ULID — the specre's `id` — to link specifications and source code in both directions. No intermediate tag layer is needed.

### How It Works

```
┌─────────────────────────────┐
│  specre file (.md)          │  ← Source of truth
│  ┌────────────────────────┐ │
│  │ id: ULID               │ │
│  │ name / status          │ │
│  │ last_verified          │ │
│  └────────────────────────┘ │
│  ## Related Files           │  ← Path-based reference (human-readable)
│  ## Scenarios               │
└──────────────┬──────────────┘
               │ id (ULID)
               ▼
┌──────────────────────────────┐
│  source file                 │
│  // @specre <ULID>           │  ← Reverse reference (machine-readable)
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│  index.json                  │  ← Generated cache (CLI-managed)
│  specres[]: front-matter     │
│  source_refs[]: marker index │
└──────────────────────────────┘
```

- **specre → Source**: The "Related Files" section lists file paths. Human-readable, always up to date with the specre.
- **Source → specre**: A `@specre` marker in a source comment references the specre's `id`. Machine-readable, scannable by CLI.
- **index.json**: A derived artifact generated by `specre index`. Can be regenerated at any time. Never edit manually.

### Source File Markers

Place a `@specre` marker in a comment to link a source file back to its specre:

```ruby
# @specre 01HZYPMZRK8F9R2DGBGGMM2N8T
class CreateQuotation < Usecase
  # ...
end
```

A file may reference multiple specres:

```python
# @specre 01HZYPMZRK8F9R2DGBGGMM2N8T
# @specre 01HZYQ4N7XW3A8B5C6D9E0F1G2
class QuotationService:
    ...
```

The marker can be placed at the top of a file, above a class or function definition, or at the end — wherever it best communicates the relationship. The CLI detects markers by scanning for the pattern `@specre [0-9A-Z]{26}`, ignoring comment syntax.

**Marker syntax by language:**

| Language | Marker |
|----------|--------|
| Ruby / Python / GDScript / Shell | `# @specre 01HZYPM...` |
| JavaScript / TypeScript / Java / C# / C++ | `// @specre 01HZYPM...` |
| HTML / XML | `<!-- @specre 01HZYPM... -->` |
| CSS | `/* @specre 01HZYPM... */` |
| SQL | `-- @specre 01HZYPM...` |

### Why ULID?

[ULID](https://github.com/ulid/spec) (Universally Unique Lexicographically Sortable Identifier) was chosen over UUID or sequential IDs because:

- **Sortable by creation time**: specres created earlier sort first, providing a natural chronological order without relying on filenames.
- **No coordination required**: ULIDs can be generated independently by any developer or agent without a central registry.
- **Compact**: 26 characters (vs. 36 for UUID), reducing noise in source comments.
- **Monotonic within a millisecond**: Multiple specres created in rapid succession maintain order.

## Index Format

`specre index` scans the specification directory and source tree to generate `index.json` inside `specre_dir` — a machine-readable cache for fast lookups.

```json
{
  "version": 1,
  "generated_at": "2026-03-01T12:00:00Z",
  "specres": [
    {
      "id": "01HZYPMZRK8F9R2DGBGGMM2N8T",
      "name": "user_can_create_new_quotation",
      "status": "draft",
      "domain": "quotation",
      "path": "docs/specres/quotation/creation/user_can_create_new_quotation.md",
      "last_verified": "2026-03-01"
    }
  ],
  "source_refs": [
    {
      "specre_id": "01HZYPMZRK8F9R2DGBGGMM2N8T",
      "file": "app/usecases/create_quotation.rb",
      "line": 1
    }
  ]
}
```

### `specres` Array

Each entry mirrors the front-matter of a specre file, with two derived fields:

| Field | Source | Description |
|-------|--------|-------------|
| `id` | front-matter | The specre's ULID |
| `name` | front-matter | Human-readable title |
| `status` | front-matter | Current lifecycle status |
| `domain` | directory name | Extracted from the top-level directory under the specres root (e.g., `docs/specres/quotation/creation/foo.md` → `"quotation"`). Subdirectories within a domain do not affect the domain value. |
| `path` | file system | Relative path from project root to the specre file |
| `last_verified` | front-matter | Date of last verification |

### `source_refs` Array

Each entry records a `@specre` marker found in the source tree:

| Field | Description |
|-------|-------------|
| `specre_id` | The ULID referenced by the marker |
| `file` | Relative path to the source file |
| `line` | Line number where the marker was found |

### Design Principles

- **index.json is a cache, not a source of truth.** If it's missing or stale, run `specre index` to regenerate. Never edit it manually.
- **specre files are the source of truth.** All authoritative data lives in the front-matter and body of each `.md` file.
- **Per-directory _INDEX.md** can also be generated for human browsing — a Markdown table summarizing all specres in a subdirectory.

## Comparison with Other Tools

| | specre | Plain Markdown | GitHub Spec Kit | Amazon Kiro | Gherkin |
|---|---|---|---|---|---|
| Granularity | One behavior | Varies | One feature | One feature | One feature (multi-scenario) |
| Unique ID | ULID per specre | None | None | None | None |
| Lifecycle status | Yes (4 states) | None | No | No | No |
| Verification date | Yes | None | No | No | No |
| Code traceability | Bidirectional (ULID) | None | None | None | Step definitions |
| Test integration | By convention | None | Optional | Optional | Executable |
| Process coupling | None | None | Linear pipeline | Linear pipeline | Test runner |
| Files per spec | 1 | Varies | 3-4 | 3 | 1 |
| IDE dependency | None | None | None | Kiro IDE | None |
| Index generation | CLI (JSON + Markdown) | None | None | Built-in | None |

## Roadmap

> Full details: [docs/project/ROADMAP.md](docs/project/ROADMAP.md)

- **v0.1 — Core CLI** ✅ `init`, `new`, `index`, `status`
- **v0.2 — Traceability** ✅ `trace`, `orphans`, `tag`
- **v0.3 — Agent Integration** ✅ `coverage`, `health-check`, `search`, `--json` output, MCP server
- **v0.4 — Drift Detection** — `drift`, `ci`, GitHub Actions template
- **v0.5 — QA Support** — `impact`, `diff`, `export`

## Contributing

specre is in its early stages. Contributions, feedback, and real-world usage reports are welcome. Please open an issue or pull request on GitHub.

If you are contributing Rust code, please read the **[Rust Conventions](docs/guides/RUST-CONVENTIONS.md)** before submitting a pull request.

## License

MIT

Atomic, living specification cards for AI-agent-friendly development. Minimal, agnostic, and traceable.
