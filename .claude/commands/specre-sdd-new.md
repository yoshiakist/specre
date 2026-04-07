---
description: "Implement a new feature using the Spec-Driven Development workflow"
---

You are executing the SDD (Spec-Driven Development) workflow to implement a new feature. The user will provide a description of what to implement as $ARGUMENTS.

Follow these phases strictly. There is exactly ONE human checkpoint — after the specre card is written.

---

## Phase 0: MCP Preflight

If specre MCP tools are available, run `health-check` first.

- **healthy = true** → The specre ecosystem is trustworthy. Proceed to Phase 1 and use `specre search` / `specre trace` for all exploration.
- **healthy = false** → Fall back to `grep` / `glob` for code exploration instead of relying on specre tools. Specre cards can still be read as reference, but do not trust coverage or traceability to be complete.

## Phase 1: Analysis

1. Read `README.md` and `docs/project/ROADMAP.md` to understand the project philosophy and where the feature fits in the roadmap
2. Use `specre search` to find existing specre cards related to the new feature, to understand adjacent behavior and avoid duplication:
   - `specre search "<keyword>"` — find specre cards in the same area (multi-keyword AND by default)
   - `specre search "<keyword>" --or` — broaden the search when exploring related concepts
   - `specre search --domain <domain>` — browse all cards in the target domain
3. For relevant existing source files, run `specre trace <file-path>` to understand which specre cards already govern them. This helps determine whether the new feature should extend an existing specre or create a new one
4. Read relevant existing code (commands, tests, config) to understand patterns
5. Identify the specre name (subject + predicate, e.g. `specre_status_reports_project_health`)

## Phase 2: Specre Creation

1. Run `specre new docs/specres/<domain> --name <name>` to scaffold the card
2. Fill in the specre card completely:
   - **Related Files** — expected source and test file paths
   - **Functional Overview** — one-paragraph summary
   - **Design Intent** — why this feature exists
   - **Key Members** — important types, fields, parameters
   - **Scenarios** — step-by-step behavior descriptions covering happy path, edge cases, and error cases
   - **Failures / Exceptions** — error handling behavior

## --- CHECKPOINT ---

Stop here and present the completed specre card to the user for review. Do NOT proceed until the user approves. If the user requests changes, update the specre card and present it again.

Notify the user that the checkpoint has been reached (if a notification script is configured in the user's environment).

## Phase 3: Test-First Implementation (after user approval)

1. **Write integration tests** in `tests/` based on every scenario in the specre card. Use `assert_cmd` + `assert_fs` + `predicates`, following the patterns in existing test files.
2. **Implement the feature:**
   - Add the command/args to `src/cli.rs`
   - Create the handler in `src/commands/`
   - Register in `src/commands/mod.rs` and `src/main.rs`
3. **Run `cargo test`** and fix until all tests pass (both new and existing).

## Phase 4: Finalize & PR

1. Update the specre card: set `status: "stable"` and `last_verified` to today's date
2. Run `specre index`
3. Run the **Pre-PR Quality Gate** (all three must pass before committing):
   1. `cargo fmt --all` — auto-format all code
   2. `cargo clippy --all-targets -- -D warnings` — lint all targets (lib, bins, tests, examples, benches)
   3. `cargo test` — run all tests
4. Create a feature branch from `main` with a descriptive name (e.g., `feature/<specre-name>`)
5. Commit all changes with a descriptive message
6. Push the branch and create a Pull Request using `gh pr create`

## Phase 5: Drift Triage

Changes to shared files (e.g., `mod.rs`, `cli.rs`, `main.rs`) often trigger false-positive drift for unrelated specre cards. Triage them now so the next developer or agent starts with a clean ecosystem.

1. Run `specre drift --json`
2. If no drifted items → report "No drift detected. Ecosystem is clean." and finish
3. For each drifted item, spawn a **subagent** (one per item, sequentially) to compare the specre card against the code changes since `last_verified`:
   - The subagent reads the specre card (Functional Overview, Scenarios, Failures / Exceptions)
   - The subagent runs `git diff` from `last_verified` to HEAD for each changed file
   - The subagent returns a verdict: `no_drift` | `drift_spec_stale` | `drift_impl_wrong` | `uncertain`
4. Process each verdict:
   - **`no_drift`** (false positive) → update `last_verified` to today's date in the specre card's YAML front-matter
   - **`drift_spec_stale`** → report to user: the spec needs updating. Suggest `/specre-sdd-fix` as a follow-up
   - **`drift_impl_wrong`** → report to user: implementation may have regressed
   - **`uncertain`** → report to user: requires manual review
5. If any `last_verified` dates were updated:
   - Run `specre index`
   - Commit the updates as a separate commit (e.g., "chore: update last_verified for false-positive drift")
   - Push to update the PR
6. Present a summary: how many false positives were resolved, how many real drifts remain
