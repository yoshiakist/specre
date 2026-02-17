// @specre 01KHB48EYB9686YYQMYFYQ5R1Z
// @specre 01KHB48EES4FR5TFV6ZP2W3MGT
// @specre 01KHB48DYZDN8GHXPX7MSYJ1NZ
// @specre 01KHAKAYN5WPTDVR99D5Q5TMJE
// @specre 01KHFD5R1G3C5R34XMQXQTTMM9
// @specre 01KHG0A2V4YXE918WMJCY7WFE8
use crate::config;
use crate::error::SpecreError;
use crate::status::Status;
use chrono::Utc;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct IndexOutput {
    index_file: String,
    specre_count: usize,
    source_ref_count: usize,
    index_md_files: Vec<String>,
}

#[derive(Serialize)]
struct Index {
    version: u32,
    generated_at: String,
    specres: Vec<SpecreEntry>,
    source_refs: Vec<SourceRef>,
}

#[derive(Serialize)]
struct SpecreEntry {
    id: String,
    name: String,
    status: Status,
    domain: String,
    path: String,
    last_verified: Option<String>,
}

#[derive(Serialize)]
struct SourceRef {
    specre_id: String,
    file: String,
    line: usize,
}

pub fn execute(json_flag: bool) -> Result<(), SpecreError> {
    let config = config::load()?;

    let specre_dir = Path::new(&config.specre_dir);
    let specres = scan_specre_files(specre_dir, &config.specre_dir);
    let source_refs = scan_source_refs(&config.source_dirs, config.target_extensions.as_deref());

    let index = Index {
        version: 1,
        generated_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        specres,
        source_refs,
    };

    let index_json_path = specre_dir.join("index.json");
    let index_json_rel = format!("{}/index.json", config.specre_dir);
    let json = serde_json::to_string_pretty(&index)?;
    fs::write(&index_json_path, &json).map_err(|e| SpecreError::Io {
        path: index_json_path.clone(),
        source: e,
    })?;

    let index_md_files = generate_index_md(specre_dir, &config.specre_dir, &index.specres)?;

    if json_flag {
        let output = IndexOutput {
            index_file: index_json_rel,
            specre_count: index.specres.len(),
            source_ref_count: index.source_refs.len(),
            index_md_files,
        };
        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{json_str}");
    } else {
        println!(
            "Generated {index_json_rel} ({} specres, {} source refs)",
            index.specres.len(),
            index.source_refs.len()
        );
        for md_file in &index_md_files {
            println!("Generated {md_file}");
        }
    }

    Ok(())
}

fn scan_specre_files(dir: &Path, specre_dir_str: &str) -> Vec<SpecreEntry> {
    let mut entries = Vec::new();
    if !dir.exists() {
        return entries;
    }
    collect_md_files(dir, &mut |path| {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: failed to read '{}': {e}", path.display());
                return;
            }
        };
        match parse_frontmatter(&content) {
            Some(fm) => {
                let rel_path = to_forward_slash(path);
                let domain = extract_domain(&rel_path, specre_dir_str).to_owned();
                entries.push(SpecreEntry {
                    id: fm.id,
                    name: fm.name,
                    status: fm.status,
                    domain,
                    path: rel_path.into_owned(),
                    last_verified: fm.last_verified,
                });
            }
            None => {
                eprintln!(
                    "Warning: skipping '{}' (malformed front-matter)",
                    path.display()
                );
            }
        }
    });
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    entries
}

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
    sub_entries.sort_by_key(|e| e.file_name());
    for entry in sub_entries {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with('.'))
                .unwrap_or(false)
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

pub struct Frontmatter {
    pub id: String,
    pub name: String,
    pub status: Status,
    pub last_verified: Option<String>,
}

pub fn parse_frontmatter(content: &str) -> Option<Frontmatter> {
    let content = content.trim_start_matches('\u{feff}'); // BOM
    if !content.starts_with("---") {
        return None;
    }
    let after_first = &content[3..];
    let end = after_first.find("\n---")?;
    let block = &after_first[..end];

    #[derive(serde::Deserialize)]
    struct RawFrontmatter {
        id: String,
        name: String,
        status: Status,
        last_verified: Option<String>,
    }

    let raw: RawFrontmatter = serde_yaml::from_str(block).ok()?;

    Some(Frontmatter {
        id: raw.id,
        name: raw.name,
        status: raw.status,
        last_verified: raw.last_verified,
    })
}

fn extract_domain<'a>(rel_path: &'a str, specre_dir: &str) -> &'a str {
    let prefix = if specre_dir.ends_with('/') {
        specre_dir.to_string()
    } else {
        format!("{specre_dir}/")
    };
    let after = rel_path.strip_prefix(&prefix).unwrap_or(rel_path);
    after.split('/').next().unwrap_or("unknown")
}

/// Extracts a ULID from a `@specre` marker on a line.
/// Returns None if the marker appears inside a string literal
/// (preceded by a quote character `"` or `'` on the same line).
pub fn extract_marker_ulid(line: &str) -> Option<&str> {
    let pos = line.find("@specre ")?;
    let before = &line[..pos];
    if before.contains('"') || before.contains('\'') {
        return None;
    }
    let after = &line[pos + 8..];
    if after.len() < 26 {
        return None;
    }
    let candidate = &after[..26];
    if candidate
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        Some(candidate)
    } else {
        None
    }
}

/// Result of a single-pass scan of all source files for `@specre` markers.
/// Used by `compute_coverage`, `compute_orphans`, and `health_check` to
/// avoid duplicate directory traversals and file reads.
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

pub struct MarkerLocation {
    pub file: String,
    pub line: usize,
    pub ulid: String,
}

/// Scans all files in `source_dirs` once, collecting both coverage and marker data.
pub fn scan_source_markers(
    source_dirs: &[String],
    target_extensions: Option<&[String]>,
) -> SourceScanResult {
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
        collect_all_files(dir, target_extensions, &mut |path| {
            total += 1;
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: failed to read '{}': {e}", path.display());
                    uncovered.push(to_forward_slash(path).into_owned());
                    return;
                }
            };
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

fn scan_source_refs(
    source_dirs: &[String],
    target_extensions: Option<&[String]>,
) -> Vec<SourceRef> {
    let mut refs = Vec::new();
    for dir_str in source_dirs {
        let dir = Path::new(dir_str);
        if !dir.exists() {
            continue;
        }
        collect_all_files(dir, target_extensions, &mut |path| {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: failed to read '{}': {e}", path.display());
                    return;
                }
            };
            let rel_path = to_forward_slash(path).into_owned();
            for (line_num, line) in content.lines().enumerate() {
                if let Some(ulid) = extract_marker_ulid(line) {
                    refs.push(SourceRef {
                        specre_id: ulid.to_string(),
                        file: rel_path.clone(),
                        line: line_num + 1,
                    });
                }
            }
        });
    }
    refs.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    refs
}

pub fn collect_all_files<F: FnMut(&Path)>(
    dir: &Path,
    target_extensions: Option<&[String]>,
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
    sub_entries.sort_by_key(|e| e.file_name());
    for entry in sub_entries {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }
            collect_all_files(&path, target_extensions, cb);
        } else {
            if let Some(exts) = target_extensions {
                let matches = path
                    .extension()
                    .is_some_and(|ext| exts.iter().any(|e| e == ext.to_string_lossy().as_ref()));
                if !matches {
                    continue;
                }
            }
            cb(&path);
        }
    }
}

pub fn to_forward_slash(path: &Path) -> Cow<'_, str> {
    let s = path.to_string_lossy();
    if s.contains('\\') {
        Cow::Owned(s.replace('\\', "/"))
    } else {
        s
    }
}

fn generate_index_md(
    specre_dir: &Path,
    specre_dir_str: &str,
    specres: &[SpecreEntry],
) -> Result<Vec<String>, SpecreError> {
    // Group specres by domain
    let mut by_domain: BTreeMap<String, Vec<&SpecreEntry>> = BTreeMap::new();
    for entry in specres {
        by_domain
            .entry(entry.domain.clone())
            .or_default()
            .push(entry);
    }

    let prefix = if specre_dir_str.ends_with('/') {
        specre_dir_str.to_string()
    } else {
        format!("{specre_dir_str}/")
    };

    let mut generated_files = Vec::new();

    for (domain, entries) in &by_domain {
        let domain_dir = specre_dir.join(domain);
        let index_path = domain_dir.join("_INDEX.md");
        let domain_prefix = format!("{prefix}{domain}/");

        let mut md = format!(
            "# {domain}\n\n| Name | Status | Last Verified |\n|------|--------|---------------|\n"
        );

        for entry in entries {
            let rel_to_domain = entry
                .path
                .strip_prefix(&domain_prefix)
                .unwrap_or(&entry.path);
            let display_name = &entry.name;
            let last_verified = entry.last_verified.as_deref().unwrap_or("-");
            md.push_str(&format!(
                "| [{display_name}]({rel_to_domain}) | {} | {last_verified} |\n",
                entry.status
            ));
        }

        fs::write(&index_path, &md).map_err(|e| SpecreError::Io {
            path: index_path.clone(),
            source: e,
        })?;

        let index_rel =
            to_forward_slash(&PathBuf::from(specre_dir_str).join(domain).join("_INDEX.md"))
                .into_owned();
        generated_files.push(index_rel);
    }

    Ok(generated_files)
}
