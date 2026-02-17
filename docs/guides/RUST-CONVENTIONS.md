# Rust Conventions

## Types
- All public structs and enums MUST derive `Debug`
- Filesystem paths MUST use `PathBuf` (owned) or `&Path` (borrowed), never `String`
- CLI args that have a corresponding type (Status, NaiveDate, PathBuf) MUST
  parse at the clap boundary, not in the command handler
- Prefer `Cow<'_, str>` over `String` when the value may or may not need allocation

## Error Handling
- Never use `unwrap()`, `expect()`, or `Box<dyn Error>` in non-test code
- All `From` conversions for `SpecreError` variants MUST be implemented
  when the same `.map_err(SpecreError::Variant)` appears more than twice
- File I/O: attempt the operation first, match on ErrorKind.
  Never check `path.exists()` before `fs::read/write` (TOCTOU)

## Performance
- Never allocate inside a per-item closure when the value is loop-invariant
  (e.g., `to_lowercase()` on a query string)
- Use `Rc<str>` (not Arc, not String::clone) for shared strings in
  single-threaded hot paths

## Dependencies
- Never add a dependency whose repository is archived or unmaintained
- Prefer crates with >1M downloads or maintained by known teams
- Always use `default-features = false` and enable only needed features

## Code Organization
- No logic duplication across modules. If two functions share >10 lines
  of structural logic, extract a shared helper
- Comments in source code MUST be in English
