// @specre 01KHB48EYB9686YYQMYFYQ5R1Z
// @specre 01KHB48EES4FR5TFV6ZP2W3MGT
// @specre 01KHB48DYZDN8GHXPX7MSYJ1NZ
// @specre 01KHAKAYN5WPTDVR99D5Q5TMJE
use crate::config;
use chrono::Utc;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

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
    status: String,
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

pub fn execute() -> Result<(), String> {
    let config = config::load()?;

    let specre_dir = Path::new(&config.specre_dir);
    let specres = scan_specre_files(specre_dir, &config.specre_dir);
    let source_refs = scan_source_refs(&config.source_dirs);

    let index = Index {
        version: 1,
        generated_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        specres,
        source_refs,
    };

    let json =
        serde_json::to_string_pretty(&index).map_err(|e| format!("Failed to serialize: {e}"))?;
    fs::write("index.json", &json).map_err(|e| format!("Failed to write index.json: {e}"))?;

    println!(
        "Generated index.json ({} specres, {} source refs)",
        index.specres.len(),
        index.source_refs.len()
    );

    generate_index_md(specre_dir, &config.specre_dir, &index.specres)?;

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
                let domain = extract_domain(&rel_path, specre_dir_str);
                entries.push(SpecreEntry {
                    id: fm.id,
                    name: fm.name,
                    status: fm.status,
                    domain,
                    path: rel_path,
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

pub fn collect_md_files(dir: &Path, cb: &mut dyn FnMut(&Path)) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    let mut sub_entries: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
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
            // Skip INDEX.md files
            if path.file_name().is_some_and(|n| n == "INDEX.md") {
                continue;
            }
            cb(&path);
        }
    }
}

pub struct Frontmatter {
    pub id: String,
    pub name: String,
    pub status: String,
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

    let mut id = None;
    let mut name = None;
    let mut status = None;
    let mut last_verified = None;

    for line in block.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim();
            let val = val.trim().trim_matches('"');
            match key {
                "id" => id = Some(val.to_string()),
                "name" => name = Some(val.to_string()),
                "status" => status = Some(val.to_string()),
                "last_verified" => last_verified = Some(val.to_string()),
                _ => {}
            }
        }
    }

    Some(Frontmatter {
        id: id?,
        name: name?,
        status: status?,
        last_verified,
    })
}

fn extract_domain(rel_path: &str, specre_dir: &str) -> String {
    let prefix = if specre_dir.ends_with('/') {
        specre_dir.to_string()
    } else {
        format!("{specre_dir}/")
    };
    let after = rel_path.strip_prefix(&prefix).unwrap_or(rel_path);
    after.split('/').next().unwrap_or("unknown").to_string()
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

fn scan_source_refs(source_dirs: &[String]) -> Vec<SourceRef> {
    let mut refs = Vec::new();
    for dir_str in source_dirs {
        let dir = Path::new(dir_str);
        if !dir.exists() {
            continue;
        }
        collect_all_files(dir, &mut |path| {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => return,
            };
            let rel_path = to_forward_slash(path);
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

pub fn collect_all_files(dir: &Path, cb: &mut dyn FnMut(&Path)) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    let mut sub_entries: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
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
            collect_all_files(&path, cb);
        } else {
            cb(&path);
        }
    }
}

pub fn to_forward_slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn generate_index_md(
    specre_dir: &Path,
    specre_dir_str: &str,
    specres: &[SpecreEntry],
) -> Result<(), String> {
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

    for (domain, entries) in &by_domain {
        let domain_dir = specre_dir.join(domain);
        let index_path = domain_dir.join("INDEX.md");
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

        fs::write(&index_path, &md)
            .map_err(|e| format!("Failed to write '{}': {e}", index_path.display()))?;

        let index_rel =
            to_forward_slash(&PathBuf::from(specre_dir_str).join(domain).join("INDEX.md"));
        println!("Generated {index_rel}");
    }

    Ok(())
}
