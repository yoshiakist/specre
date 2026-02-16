// @specre 01KHB48EES4FR5TFV6ZP2W3MGT
// @specre 01KHG0A2V4YXE918WMJCY7WFE8
use crate::commands::index::{
    collect_all_files, collect_md_files, extract_marker_ulid, parse_frontmatter, to_forward_slash,
};
use crate::config;
use crate::error::SpecreError;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

struct SpecreInfo {
    id: String,
    path: String,
    status: String,
}

struct DanglingMarker {
    file: String,
    line: usize,
    ulid: String,
}

#[derive(Serialize)]
struct OrphansOutput {
    orphan_specres: Vec<String>,
    dangling_markers: Vec<DanglingMarkerOutput>,
}

#[derive(Serialize)]
struct DanglingMarkerOutput {
    file: String,
    line: usize,
    id: String,
}

pub struct OrphanResult {
    pub orphan_specres: usize,
    pub dangling_markers: usize,
}

pub fn compute_orphans(
    specre_dir: &str,
    source_dirs: &[String],
    target_extensions: Option<&[String]>,
) -> OrphanResult {
    let specre_path = Path::new(specre_dir);

    // Collect all specre ids and paths
    let mut specres: Vec<SpecreInfo> = Vec::new();
    if specre_path.exists() {
        collect_md_files(specre_path, &mut |path| {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => return,
            };
            if let Some(fm) = parse_frontmatter(&content) {
                specres.push(SpecreInfo {
                    id: fm.id,
                    path: to_forward_slash(path),
                    status: fm.status,
                });
            }
        });
    }

    let specre_ids: HashSet<&str> = specres.iter().map(|s| s.id.as_str()).collect();

    // Collect all source markers
    let mut marker_ulids: HashSet<String> = HashSet::new();
    let mut dangling_count = 0usize;

    for dir_str in source_dirs {
        let dir = Path::new(dir_str);
        if !dir.exists() {
            continue;
        }
        collect_all_files(dir, target_extensions, &mut |path| {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => return,
            };
            for line in content.lines() {
                if let Some(candidate) = extract_marker_ulid(line) {
                    marker_ulids.insert(candidate.to_string());
                    if !specre_ids.contains(candidate) {
                        dangling_count += 1;
                    }
                }
            }
        });
    }

    let orphan_specres = specres
        .iter()
        .filter(|s| s.status != "deprecated" && !marker_ulids.contains(&s.id))
        .count();

    OrphanResult {
        orphan_specres,
        dangling_markers: dangling_count,
    }
}

pub fn execute(json: bool) -> Result<(), SpecreError> {
    let config = config::load()?;
    let specre_dir = Path::new(&config.specre_dir);

    // Collect all specre ids and paths
    let mut specres: Vec<SpecreInfo> = Vec::new();
    if specre_dir.exists() {
        collect_md_files(specre_dir, &mut |path| {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: failed to read '{}': {e}", path.display());
                    return;
                }
            };
            match parse_frontmatter(&content) {
                Some(fm) => {
                    specres.push(SpecreInfo {
                        id: fm.id,
                        path: to_forward_slash(path),
                        status: fm.status,
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
    }

    let specre_ids: HashSet<&str> = specres.iter().map(|s| s.id.as_str()).collect();

    // Collect all source markers
    let mut marker_ulids: HashSet<String> = HashSet::new();
    let mut dangling: Vec<DanglingMarker> = Vec::new();

    for dir_str in &config.source_dirs {
        let dir = Path::new(dir_str);
        if !dir.exists() {
            continue;
        }
        collect_all_files(dir, config.target_extensions.as_deref(), &mut |path| {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => return,
            };
            let rel_path = to_forward_slash(path);
            for (line_num, line) in content.lines().enumerate() {
                if let Some(candidate) = extract_marker_ulid(line) {
                    marker_ulids.insert(candidate.to_string());
                    if !specre_ids.contains(candidate) {
                        dangling.push(DanglingMarker {
                            file: rel_path.clone(),
                            line: line_num + 1,
                            ulid: candidate.to_string(),
                        });
                    }
                }
            }
        });
    }

    // Find orphan specres (non-deprecated specres with no source markers)
    let mut orphans: Vec<&SpecreInfo> = specres
        .iter()
        .filter(|s| s.status != "deprecated" && !marker_ulids.contains(&s.id))
        .collect();
    orphans.sort_by(|a, b| a.path.cmp(&b.path));

    dangling.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));

    if json {
        let output = OrphansOutput {
            orphan_specres: orphans.iter().map(|s| s.path.clone()).collect(),
            dangling_markers: dangling
                .iter()
                .map(|d| DanglingMarkerOutput {
                    file: d.file.clone(),
                    line: d.line,
                    id: d.ulid.clone(),
                })
                .collect(),
        };
        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{json_str}");

        if orphans.is_empty() && dangling.is_empty() {
            return Ok(());
        }
        return Err(SpecreError::NonZeroExit);
    }

    if orphans.is_empty() && dangling.is_empty() {
        println!("No orphans or dangling markers found.");
        return Ok(());
    }

    if !orphans.is_empty() {
        println!("Orphan specres (no source markers):");
        for s in &orphans {
            println!("  {}", s.path);
        }
    }

    if !orphans.is_empty() && !dangling.is_empty() {
        println!();
    }

    if !dangling.is_empty() {
        println!("Dangling markers (no matching specre):");
        for d in &dangling {
            println!("  {}:{}  {}", d.file, d.line, d.ulid);
        }
    }

    Err(SpecreError::NonZeroExit)
}
