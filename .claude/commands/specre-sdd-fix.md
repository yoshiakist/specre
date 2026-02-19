---
description: "Fix or modify an existing feature using the Spec-Driven Development workflow"
---

You are executing the SDD (Spec-Driven Development) workflow to fix or modify an existing feature. The user will provide a description of the change as $ARGUMENTS.

Follow these phases strictly. There is exactly ONE human checkpoint — after the specre card is updated.

---

## Phase 0: MCP Preflight

If specre MCP tools are available, run `health-check` first.

- **healthy = true** → The specre ecosystem is trustworthy. Proceed to Phase 1 and use `specre search` / `specre trace` for all exploration.
- **healthy = false** → Fall back to `grep` / `glob` for code exploration instead of relying on specre tools. Specre cards can still be read as reference, but do not trust coverage or traceability to be complete.

## Phase 1: Analysis

1. Read `README.md` and `docs/project/ROADMAP.md` to understand the project philosophy and roadmap context
2. Use `specre search` to locate the specre cards affected by the change:
   - `specre search "<keyword>"` — find specre cards related to the change (multi-keyword AND by default)
   - `specre search "<keyword>" --or` — broaden the search when the initial query is too narrow
   - `specre search --domain <domain>` — narrow by domain when the affected area is known
3. For each candidate source file, run `specre trace <file-path>` to discover which specre cards govern it, and vice versa — run `specre trace <ULID>` to find all source files linked to a specre card
4. Read the related source files and test files listed in the specre card's "Related Files" section
5. Understand the current behavior by reading the scenarios and comparing with the implementation
6. Identify the gap between the current behavior and the requested change

## Phase 2: Specre Update

1. Update the affected specre card(s) to reflect the new behavior:
   - Modify **Scenarios** — add, remove, or change steps as needed
   - Update **Functional Overview** if the overall behavior description changes
   - Update **Key Members** if types, fields, or parameters change
   - Update **Failures / Exceptions** if error handling changes
   - Update **Related Files** if new files are introduced or existing ones are removed
2. Set `status` to `"in-development"` (it will return to `"stable"` in Phase 4)
3. Clearly mark what changed — present a diff-style summary to the user

## --- CHECKPOINT ---

Stop here and present the updated specre card to the user for review. Show what was changed and why. Do NOT proceed until the user approves. If the user requests changes, update the specre card and present it again.

Notify the user that the checkpoint has been reached (if a notification script is configured in the user's environment).

## Phase 3: Test-First Implementation (after user approval)

1. **Update or add integration tests** in `tests/` to match the updated scenarios. Modify existing tests for changed behavior; add new tests for new scenarios; remove tests for removed scenarios.
2. **Implement the change** in the source code, following existing patterns.
3. **Run `cargo test`** and fix until all tests pass (both modified and existing).

## Phase 4: Finalize & PR

1. Update the specre card: set `status: "stable"` and `last_verified` to today's date
2. Run `specre index`
3. Run the **Pre-PR Quality Gate** (all three must pass before committing):
   1. `cargo fmt --all` — auto-format all code
   2. `cargo clippy --all-targets -- -D warnings` — lint all targets (lib, bins, tests, examples, benches)
   3. `cargo test` — run all tests
4. Create a feature branch from `main` with a descriptive name (e.g., `fix/<specre-name>-<short-description>`)
5. Commit all changes with a descriptive message
6. Push the branch and create a Pull Request using `gh pr create`
