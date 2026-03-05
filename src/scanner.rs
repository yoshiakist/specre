// @specre 01KHB48EYB9686YYQMYFYQ5R1Z
// @specre 01KHB48EES4FR5TFV6ZP2W3MGT
// @specre 01KHFD5R1G3C5R34XMQXQTTMM9
use crate::card::to_forward_slash;
use crate::parser::extract_marker_ulid;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn collect_md_files<F: FnMut(&Path)>(dir: &Path, cb: &mut F) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) => {
            eprintln!("Warning: failed to read directory '{}': {e}", dir.display());
            return;
        }
    };
    let mut sub_entries: Vec<_> = read_dir
        .filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(err) => {
                eprintln!(
                    "Warning: failed to read entry in '{}': {err}",
                    dir.display()
                );
                None
            }
        })
        .collect();
    sub_entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in sub_entries {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with('.'))
            {
                continue;
            }
            collect_md_files(&path, cb);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            // Skip _INDEX.md files
            if path.file_name().is_some_and(|n| n == "_INDEX.md") {
                continue;
            }
            cb(&path);
        }
    }
}

/// Extensions that are always excluded from source scanning.
/// These are text files that pass UTF-8 decoding but are never source code.
const EXCLUDED_EXTENSIONS: &[&str] = &["svg"];

/// Compiles an optional list of glob patterns into a [`GlobSet`] for O(1) matching.
///
/// Invalid patterns are reported as warnings on stderr and skipped.
#[must_use]
pub fn compile_exclude_patterns(patterns: Option<&[String]>) -> Option<GlobSet> {
    let patterns = patterns?;
    if patterns.is_empty() {
        return None;
    }
    let mut builder = GlobSetBuilder::new();
    for pat in patterns {
        match GlobBuilder::new(pat).literal_separator(false).build() {
            Ok(glob) => {
                builder.add(glob);
            }
            Err(e) => {
                eprintln!("Warning: invalid exclude pattern '{pat}': {e}");
            }
        }
    }
    match builder.build() {
        Ok(set) if !set.is_empty() => Some(set),
        Ok(_) => None,
        Err(e) => {
            eprintln!("Warning: failed to compile exclude patterns: {e}");
            None
        }
    }
}

pub fn collect_all_files<F: FnMut(&Path)>(
    dir: &Path,
    target_extensions: Option<&[String]>,
    exclude_set: Option<&GlobSet>,
    cb: &mut F,
) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) => {
            eprintln!("Warning: failed to read directory '{}': {e}", dir.display());
            return;
        }
    };
    let mut sub_entries: Vec<_> = read_dir
        .filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(err) => {
                eprintln!(
                    "Warning: failed to read entry in '{}': {err}",
                    dir.display()
                );
                None
            }
        })
        .collect();
    sub_entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in sub_entries {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with('.'))
            {
                continue;
            }
            // Check exclude patterns on directory before recursing
            if let Some(excludes) = exclude_set {
                let forward = to_forward_slash(&path);
                if excludes.is_match(forward.as_ref()) {
                    continue;
                }
            }
            collect_all_files(&path, target_extensions, exclude_set, cb);
        } else {
            // Skip dot files (mirrors dot-directory exclusion above)
            if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with('.'))
            {
                continue;
            }
            // Skip built-in excluded extensions
            if path.extension().is_some_and(|ext| {
                EXCLUDED_EXTENSIONS
                    .iter()
                    .any(|e| *e == ext.to_string_lossy().as_ref())
            }) {
                continue;
            }
            if let Some(exts) = target_extensions {
                let matches = path
                    .extension()
                    .is_some_and(|ext| exts.iter().any(|e| e == ext.to_string_lossy().as_ref()));
                if !matches {
                    continue;
                }
            }
            // Check exclude patterns on file
            if let Some(excludes) = exclude_set {
                let forward = to_forward_slash(&path);
                if excludes.is_match(forward.as_ref()) {
                    continue;
                }
            }
            cb(&path);
        }
    }
}

/// Result of a single-pass scan of all source files for `@specre` markers.
/// Used by `compute_coverage`, `compute_orphans`, and `health_check` to
/// avoid duplicate directory traversals and file reads.
#[derive(Debug)]
pub struct SourceScanResult {
    /// Total number of source files scanned.
    pub total: usize,
    /// Number of files containing at least one `@specre` marker.
    pub tagged: usize,
    /// Paths of files without any `@specre` marker (sorted).
    pub uncovered: Vec<String>,
    /// Set of all unique ULIDs found across all markers.
    pub marker_ulids: HashSet<String>,
    /// Every marker occurrence: (file, line, ulid).
    pub all_markers: Vec<MarkerLocation>,
}

#[derive(Debug)]
pub struct MarkerLocation {
    pub file: String,
    pub line: usize,
    pub ulid: String,
}

/// Scans all files in `source_dirs` once, collecting both coverage and marker data.
#[must_use]
pub fn scan_source_markers(
    source_dirs: &[String],
    target_extensions: Option<&[String]>,
    exclude_patterns: Option<&[String]>,
) -> SourceScanResult {
    let exclude_set = compile_exclude_patterns(exclude_patterns);
    let mut total = 0usize;
    let mut tagged = 0usize;
    let mut uncovered = Vec::new();
    let mut marker_ulids: HashSet<String> = HashSet::new();
    let mut all_markers = Vec::new();

    for dir_str in source_dirs {
        let dir = Path::new(dir_str);
        if !dir.exists() {
            continue;
        }
        collect_all_files(dir, target_extensions, exclude_set.as_ref(), &mut |path| {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                    // Binary (non-UTF-8) file — skip silently
                    return;
                }
                Err(e) => {
                    total += 1;
                    eprintln!("Warning: failed to read '{}': {e}", path.display());
                    uncovered.push(to_forward_slash(path).into_owned());
                    return;
                }
            };
            total += 1;
            let rel_path = to_forward_slash(path).into_owned();
            let mut file_has_marker = false;
            for (line_num, line) in content.lines().enumerate() {
                if let Some(candidate) = extract_marker_ulid(line) {
                    file_has_marker = true;
                    marker_ulids.insert(candidate.to_string());
                    all_markers.push(MarkerLocation {
                        file: rel_path.clone(),
                        line: line_num + 1,
                        ulid: candidate.to_string(),
                    });
                }
            }
            if file_has_marker {
                tagged += 1;
            } else {
                uncovered.push(rel_path);
            }
        });
    }

    uncovered.sort();
    all_markers.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));

    SourceScanResult {
        total,
        tagged,
        uncovered,
        marker_ulids,
        all_markers,
    }
}
