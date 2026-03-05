// @specre 01KHFFCX8BCDAYP8YHG0J65H0E
// Each integration test binary compiles `common` independently; helpers not
// used by every binary would otherwise trigger dead_code warnings.
#![allow(dead_code)]
use std::fs;

/// Write a `specre.toml` with `exclude_patterns` into `dir`.
pub fn write_config_with_exclude(
    dir: &std::path::Path,
    specre_dir: &str,
    source_dirs: &[&str],
    exclude_patterns: &[&str],
) {
    let dirs_toml: Vec<String> = source_dirs.iter().map(|s| format!("\"{s}\"")).collect();
    let pats_toml: Vec<String> = exclude_patterns
        .iter()
        .map(|s| format!("\"{s}\""))
        .collect();
    let content = format!(
        "specre_dir = \"{specre_dir}\"\nsource_dirs = [{}]\nexclude_patterns = [{}]\n",
        dirs_toml.join(", "),
        pats_toml.join(", ")
    );
    fs::write(dir.join("specre.toml"), content).unwrap();
}

/// Returns `true` when the process runs as root (effective UID 0).
///
/// Root bypasses POSIX file-permission checks, making permission-based
/// tests meaningless.  Call this at the top of such tests and `return`
/// early when it is `true`.
#[cfg(unix)]
pub fn is_root() -> bool {
    // SAFETY: `geteuid` is a trivial, always-safe POSIX syscall.
    unsafe { libc::geteuid() == 0 }
}

#[cfg(not(unix))]
pub fn is_root() -> bool {
    false
}
