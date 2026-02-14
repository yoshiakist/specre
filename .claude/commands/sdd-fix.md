You are executing the SDD (Spec-Driven Development) workflow to fix or modify an existing feature. The user will provide a description of the change as $ARGUMENTS.

Follow these phases strictly. There is exactly ONE human checkpoint — after the specre card is updated.

---

## Phase 1: Analysis

1. Read the existing specre cards in `docs/specres/` to identify which specre(s) are affected by the change
2. Read the related source files and test files listed in the specre card's "Related Files" section
3. Understand the current behavior by reading the scenarios and comparing with the implementation
4. Identify the gap between the current behavior and the requested change

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

## Phase 3: Test-First Implementation (after user approval)

1. **Update or add integration tests** in `tests/` to match the updated scenarios. Modify existing tests for changed behavior; add new tests for new scenarios; remove tests for removed scenarios.
2. **Implement the change** in the source code, following existing patterns.
3. **Run `cargo test`** and fix until all tests pass (both modified and existing).

## Phase 4: Finalize & PR

1. Update the specre card: set `status: "stable"` and `last_verified` to today's date
2. Run `specre index`
3. Create a feature branch from `main` with a descriptive name (e.g., `fix/<specre-name>-<short-description>`)
4. Commit all changes with a descriptive message
5. Push the branch and create a Pull Request using `gh pr create`
