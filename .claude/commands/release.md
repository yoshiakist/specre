You are executing the release workflow for specre. The user may optionally provide a version as $ARGUMENTS (e.g., `0.2.4`). If not provided, increment the patch version from the current `Cargo.toml`.

Human checkpoints are limited to PR merge only — the agent drives everything else autonomously.

Follow these phases strictly.

---

## Phase 1: Version Bump PR

1. Ensure you are on `main` and up to date: `git checkout main && git pull origin main`
2. Read `Cargo.toml` to determine the current version
3. Calculate the new version (use $ARGUMENTS if provided, otherwise increment the patch version)
4. Create a branch: `chore/bump-version-v<NEW_VERSION>`
5. Update `version` in `Cargo.toml`
6. Run `cargo build` to update `Cargo.lock`
7. Commit `Cargo.toml` and `Cargo.lock`
8. Push the branch and create a PR using `gh pr create`

## --- CHECKPOINT: PR merge ---

Stop here and present the PR URL. Tell the user to merge. Wait for the user to confirm.

## Phase 2: Release Tag & CI

1. `git checkout main && git pull origin main`
2. Create and push the release tag: `git tag v<NEW_VERSION> && git push origin v<NEW_VERSION>`
3. Tell the user that the release CI has been triggered
4. Wait 5 minutes (`sleep 300`), then check CI status: `gh run list --limit 1`
5. If CI is still running, wait another 2 minutes and check again. Repeat until completed or failed.
6. If CI failed, report the failure and stop. Do not proceed to Phase 3.
7. If CI succeeded, verify the release artifacts: `gh release view v<NEW_VERSION> --json tagName,assets -q '{tag: .tagName, assets: [.assets[].name]}'`

## Phase 3: Publish to crates.io

1. Run `cargo publish --dry-run` to verify the package
2. Run `cargo publish` to publish to crates.io
3. Report the final result with links:
   - GitHub Release URL
   - crates.io URL
