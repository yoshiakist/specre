---
description: "Fix code quality issues across the codebase using the SDD workflow"
---

You are executing the SDD (Spec-Driven Development) code quality workflow. The user will describe the code quality issue identified during a prompt review as $ARGUMENTS.

Follow these phases strictly. This workflow iterates over each affected file, updating specs, tests, and implementation in lockstep.

---

## Phase 0: Preflight

### 0.1 MCP Preflight

If specre MCP tools are available, run `health-check` first.

- **healthy = true** → The specre ecosystem is trustworthy. Use `specre search` / `specre trace` for all exploration in subsequent phases.
- **healthy = false** → Fall back to `grep` / `glob` for code exploration instead of relying on specre tools. Specre cards can still be read as reference, but do not trust coverage or traceability to be complete.

### 0.2 Branch Setup

1. Check the current branch with `git branch --show-current`
2. If on `main`, create and checkout an appropriate branch (e.g., `chore/code-quality-<short-description>`)

## Phase 1: Identify Violations

1. Search the codebase for code that violates the described code quality issue
2. Use `specre search` to locate related specre cards and understand the intended behavior around the violation:
   - `specre search "<keyword>"` — find specre cards related to the affected area (multi-keyword AND by default)
   - `specre search "<keyword>" --or` — broaden the search when the initial query is too narrow
   - `specre search --domain <domain>` — narrow by domain when the affected area is known
3. For each affected file, run `specre trace <file-path>` to check whether it is already linked to a specre card. This reveals:
   - Which specre cards govern the file's behavior (helps assess impact of the fix)
   - Whether the file is untracked (may need a specre card or `@specre` tag)
4. Collect all affected files into a list, along with their associated specre card ULIDs

## Phase 2: Fix Loop (repeat for each affected file)

For each affected file:

### 2.1 Trace the specre card
- Find the `@specre <ULID>` marker in the file (may be at the top, above a class/function, or at the end)
- Run `specre trace <ULID>` to locate the corresponding specre card and all other source files referencing it
- If the file has no `@specre` marker, use `specre search` with relevant keywords to find the governing specre card

### 2.2 Update the specre card (if needed)
- Review the specre card against the code quality concern
- If the expected behavior description is inadequate or inconsistent with the quality fix, update the specre card
- Set `status` to `"in-development"`

### 2.3 Update tests
- Modify the corresponding integration tests to reflect any behavioral changes

### 2.4 Update implementation
- Apply the code quality fix to the source file

### 2.5 Verify
- Run the project's test suite (e.g., `cargo test`) and confirm all tests pass

### 2.6 Update specre card status
- Set `status` to `"stable"` and `last_verified` to today's date

### 2.7 Commit
- Commit all changes for this file with a descriptive message

### 2.8 Next file
- Return to step 2.1 for the next affected file

## Phase 3: Health Check

1. Run `specre orphans` and `specre health-check`
2. If health-check passes, proceed to Phase 4
3. If health-check fails:
   1. Run `specre coverage` to investigate the affected code
   2. For untagged files related to existing behavior, run `specre tag` to link them
   3. For untagged files representing new behavior, create a new specre card

## Phase 4: Finalize & PR

1. Run `specre index`
2. Run the **Pre-PR Quality Gate** (all three must pass before committing):
   1. `cargo fmt --all` — auto-format all code
   2. `cargo clippy --all-targets -- -D warnings` — lint all targets (lib, bins, tests, examples, benches)
   3. `cargo test` — run all tests
3. Commit all remaining changes
4. Push the branch and create a Pull Request using `gh pr create`
5. Notify the user that the workflow is complete
