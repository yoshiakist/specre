// @specre 01KHFFCX8BCDAYP8YHG0J65H0E
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
