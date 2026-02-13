You are executing the SDD (Spec-Driven Development) workflow to implement a new feature. The user will provide a description of what to implement as $ARGUMENTS.

Follow these phases strictly. There is exactly ONE human checkpoint — after the specre card is written.

---

## Phase 1: Analysis

1. Read the roadmap in `README.md` to understand where the feature fits
2. Read existing specre cards in `docs/specres/` for reference on format and style
3. Read relevant existing code (commands, tests, config) to understand patterns
4. Identify the specre name (subject + predicate, e.g. `specre_status_reports_project_health`)

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

## Phase 3: Test-First Implementation (after user approval)

1. **Write integration tests** in `tests/` based on every scenario in the specre card. Use `assert_cmd` + `assert_fs` + `predicates`, following the patterns in existing test files.
2. **Implement the feature:**
   - Add the command/args to `src/cli.rs`
   - Create the handler in `src/commands/`
   - Register in `src/commands/mod.rs` and `src/main.rs`
3. **Run `cargo test`** and fix until all tests pass (both new and existing).

## Phase 4: Finalize

1. Update the specre card: set `status: "stable"` and `last_verified` to today's date
2. Run `specre index`
3. Commit all changes with a descriptive message
