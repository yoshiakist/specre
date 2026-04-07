// @specre 01KNK4YMXKJ2B395E8NPQ4DV2Q
use crate::card::{self, SpecreCard};
use crate::cli::VerifyArgs;
use crate::config;
use crate::error::SpecreError;
use crate::parser::extract_marker_ulid;
use chrono::Local;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct VerifyOutput {
    verified: Vec<VerifiedSpecre>,
    count: usize,
}

#[derive(Serialize)]
struct VerifiedSpecre {
    id: String,
    name: String,
    path: String,
    last_verified: String,
}

/// Update `last_verified` in the YAML front-matter of a specre card file.
///
/// If `last_verified` already exists, it is replaced. If it does not exist,
/// it is inserted before the closing `---` delimiter.
fn update_last_verified(path: &str, today: &str) -> Result<(), SpecreError> {
    let content = fs::read_to_string(path).map_err(|e| SpecreError::Io {
        path: path.into(),
        source: e,
    })?;

    let new_content = rewrite_frontmatter(&content, today);

    fs::write(path, new_content).map_err(|e| SpecreError::Io {
        path: path.into(),
        source: e,
    })
}

/// Rewrite front-matter to set `last_verified` to the given date.
fn rewrite_frontmatter(content: &str, today: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let has_trailing_newline = content.ends_with('\n');

    // Find the closing --- delimiter (skip the first line which is the opening ---)
    let closing_idx = lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, line)| line.trim() == "---")
        .map(|(i, _)| i);

    let Some(closing_idx) = closing_idx else {
        return content.to_string();
    };

    // Check if last_verified already exists in front-matter
    let existing_idx = lines[1..closing_idx]
        .iter()
        .enumerate()
        .find(|(_, line)| line.trim().starts_with("last_verified:"))
        .map(|(i, _)| i + 1); // offset by 1 because we started at index 1

    let new_line = format!("last_verified: \"{today}\"");

    let mut result_lines: Vec<String> = Vec::with_capacity(lines.len() + 1);
    for (i, line) in lines.iter().enumerate() {
        if Some(i) == existing_idx {
            result_lines.push(new_line.clone());
        } else {
            if existing_idx.is_none() && i == closing_idx {
                result_lines.push(new_line.clone());
            }
            result_lines.push((*line).to_string());
        }
    }

    let mut result = result_lines.join("\n");
    if has_trailing_newline {
        result.push('\n');
    }
    result
}

/// Extract file paths from the "## Related Files" section of a specre card.
fn extract_related_files(content: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut in_related = false;
    for line in content.lines() {
        if line.starts_with("## Related Files") {
            in_related = true;
            continue;
        }
        if in_related && line.starts_with("## ") {
            break;
        }
        if in_related
            && let Some(start) = line.find('`')
            && let Some(end) = line[start + 1..].find('`')
        {
            let path = &line[start + 1..start + 1 + end];
            if !path.is_empty() {
                files.push(path.to_string());
            }
        }
    }
    files
}

/// Resolve which specre ULIDs are linked to a given source file,
/// via `@specre` markers in the file and Related Files sections in cards.
fn resolve_ulids_for_file(
    file_path: &str,
    cards: &[SpecreCard],
    cfg: &config::Config,
) -> HashSet<String> {
    let mut ulids = HashSet::new();
    let normalized = file_path.replace('\\', "/");

    // 1. Scan @specre markers in the file
    if let Ok(content) = fs::read_to_string(file_path) {
        for line in content.lines() {
            if let Some(ulid) = extract_marker_ulid(line) {
                ulids.insert(ulid.to_string());
            }
        }
    }

    // 2. Scan Related Files sections in all cards
    for card in cards {
        let Ok(content) = fs::read_to_string(&card.path) else {
            continue;
        };
        let related = extract_related_files(&content);
        if related.iter().any(|f| f.replace('\\', "/") == normalized) {
            ulids.insert(card.id.clone());
        }
    }

    let _ = cfg; // cfg available for future extension
    ulids
}

/// # Errors
///
/// Returns [`SpecreError`] on config, I/O, or serialization failure.
/// Returns [`SpecreError::NonZeroExit`] when some ULIDs are not found.
pub fn execute(args: &VerifyArgs, json: bool) -> Result<(), SpecreError> {
    let cfg = config::load()?;
    let specre_dir = Path::new(&cfg.specre_dir);
    let all_cards = card::scan_specre_cards(specre_dir, &cfg.specre_dir);

    // Determine which ULIDs to verify
    let target_ulids: HashSet<String> = if !args.ulids.is_empty() {
        args.ulids.iter().cloned().collect()
    } else if let Some(ref domain) = args.domain {
        all_cards
            .iter()
            .filter(|c| c.domain == *domain)
            .map(|c| c.id.clone())
            .collect()
    } else if let Some(ref file) = args.file {
        resolve_ulids_for_file(file, &all_cards, &cfg)
    } else {
        return Err(SpecreError::InvalidArgument(
            "specify at least one ULID, --domain, or --file".to_string(),
        ));
    };

    if target_ulids.is_empty() {
        if let Some(ref domain) = args.domain {
            return Err(SpecreError::InvalidArgument(format!(
                "no specres found in domain '{domain}'"
            )));
        }
        if let Some(ref file) = args.file {
            return Err(SpecreError::InvalidArgument(format!(
                "no specres linked to file '{file}'"
            )));
        }
    }

    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut verified_list = Vec::new();
    let mut not_found: Vec<String> = Vec::new();

    // Track which requested ULIDs were actually found
    let mut found_ulids = HashSet::new();

    for card in &all_cards {
        if !target_ulids.contains(&card.id) {
            continue;
        }
        found_ulids.insert(card.id.clone());

        if let Err(e) = update_last_verified(&card.path, &today) {
            eprintln!("Warning: failed to update '{}': {e}", card.path);
            continue;
        }

        verified_list.push(VerifiedSpecre {
            id: card.id.clone(),
            name: card.name.clone(),
            path: card.path.clone(),
            last_verified: today.clone(),
        });
    }

    // Check for ULIDs that were not found
    for ulid in &target_ulids {
        if !found_ulids.contains(ulid) {
            eprintln!("Error: no specre found with id '{ulid}'");
            not_found.push(ulid.clone());
        }
    }

    let has_errors = !not_found.is_empty();

    if json {
        let output = VerifyOutput {
            count: verified_list.len(),
            verified: verified_list,
        };
        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{json_str}");
    } else {
        println!("Verified {} specre(s):", verified_list.len());
        for v in &verified_list {
            println!("  {}  {}  ({})", v.id, v.name, v.path);
        }
    }

    if has_errors {
        Err(SpecreError::NonZeroExit)
    } else {
        Ok(())
    }
}
