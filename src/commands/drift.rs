// @specre 01KNJYEGK8KFVQK7EQGK9ZCZJR
use crate::card::{self, SpecreCard, extract_domain, to_forward_slash};
use crate::cli::DriftArgs;
use crate::config;
use crate::error::SpecreError;
use crate::parser::extract_marker_ulid;
use crate::scanner::{collect_all_files, compile_exclude_patterns};
use crate::status::Status;
use crate::ulid;
use chrono::NaiveDate;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Serialize)]
struct DriftOutput {
    drifted: Vec<DriftedSpecre>,
    clean: usize,
    total: usize,
    grace_days: u64,
}

#[derive(Serialize)]
struct DriftedSpecre {
    id: String,
    name: String,
    path: String,
    domain: String,
    last_verified: Option<String>,
    changed_files: Vec<ChangedFile>,
}

#[derive(Serialize)]
struct ChangedFile {
    file: String,
    last_modified: String,
    diff_stat: String,
}

/// Parse a grace duration string like "0d", "7d", "30d" into days.
fn parse_grace(s: &str) -> Result<u64, SpecreError> {
    let s = s.trim();
    s.strip_suffix('d').map_or_else(
        || {
            Err(SpecreError::InvalidArgument(format!(
                "invalid grace format: '{s}'. Expected format: <number>d (e.g., 0d, 7d, 30d)"
            )))
        },
        |days_str| {
            days_str.parse::<u64>().map_err(|_| {
                SpecreError::InvalidArgument(format!(
                    "invalid grace format: '{s}'. Expected format: <number>d (e.g., 0d, 7d, 30d)"
                ))
            })
        },
    )
}

/// Check that the working directory is inside a git repository.
fn verify_git_repo() -> Result<(), SpecreError> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|e| {
            SpecreError::InvalidArgument(format!(
                "failed to execute git: {e}. Drift detection requires git."
            ))
        })?;
    if !output.status.success() {
        return Err(SpecreError::InvalidArgument(
            "not a git repository. Drift detection requires git history.".to_string(),
        ));
    }
    Ok(())
}

/// Get the last modified date of a file from git log (author date).
/// Returns `None` if the file has no git history.
fn git_last_modified(file: &str) -> Option<NaiveDate> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%aI", "--", file])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.trim();
    if line.is_empty() {
        return None;
    }
    chrono::DateTime::parse_from_rfc3339(line)
        .ok()
        .map(|dt| dt.date_naive())
}

/// Get `git diff --stat` for a file between a date and HEAD.
fn git_diff_stat(file: &str, since_date: NaiveDate) -> String {
    let since = format!("{since_date}");
    let output = Command::new("git")
        .args([
            "log",
            "--numstat",
            "--format=",
            &format!("--since={since}"),
            "--",
            file,
        ])
        .output();
    let Ok(output) = output else {
        return String::new();
    };
    if !output.status.success() {
        return String::new();
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut added: u64 = 0;
    let mut deleted: u64 = 0;
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            added += parts[0].parse::<u64>().unwrap_or(0);
            deleted += parts[1].parse::<u64>().unwrap_or(0);
        }
    }
    if added == 0 && deleted == 0 {
        String::new()
    } else {
        format!("+{added} -{deleted}")
    }
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

/// Collect all files linked to each specre ULID via `@specre` markers in source code.
fn collect_marker_links(cfg: &config::Config) -> HashMap<String, HashSet<String>> {
    let exclude_set = compile_exclude_patterns(cfg.exclude_patterns.as_deref());
    let mut links: HashMap<String, HashSet<String>> = HashMap::new();

    for dir_str in &cfg.source_dirs {
        let dir = Path::new(dir_str);
        if !dir.exists() {
            continue;
        }
        collect_all_files(
            dir,
            cfg.target_extensions.as_deref(),
            exclude_set.as_ref(),
            &mut |path| {
                let Ok(content) = fs::read_to_string(path) else {
                    return;
                };
                let rel_path = to_forward_slash(path).into_owned();
                for line in content.lines() {
                    if let Some(found_ulid) = extract_marker_ulid(line) {
                        links
                            .entry(found_ulid.to_string())
                            .or_default()
                            .insert(rel_path.clone());
                    }
                }
            },
        );
    }

    links
}

/// Filter a card by the target argument (ULID, path prefix, or None for all).
fn filter_by_target(card: &SpecreCard, target: Option<&str>, specre_dir: &str) -> bool {
    let Some(target) = target else {
        return true;
    };
    if ulid::is_valid(target) {
        return card.id == target;
    }
    let target_normalized = target.replace('\\', "/");
    card.path.starts_with(&target_normalized)
        || extract_domain(&card.path, specre_dir) == target_normalized.trim_end_matches('/')
}

/// Check a single specre card for drift against its related source files.
fn check_card_drift(
    card: &SpecreCard,
    related: &HashSet<String>,
    grace_days: u64,
) -> Option<DriftedSpecre> {
    if related.is_empty() {
        return None;
    }

    let verified_date = card
        .last_verified
        .as_ref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    let mut changed_files = Vec::new();

    for file in related {
        if !Path::new(file).exists() {
            continue;
        }

        let Some(file_modified) = git_last_modified(file) else {
            continue;
        };

        let is_drifted = verified_date.is_none_or(|vd| {
            let grace_end =
                vd + chrono::Duration::days(i64::try_from(grace_days).unwrap_or(i64::MAX));
            file_modified > grace_end
        });

        if is_drifted {
            let since_date = verified_date.unwrap_or(file_modified);
            let diff_stat = git_diff_stat(file, since_date);
            changed_files.push(ChangedFile {
                file: file.clone(),
                last_modified: file_modified.format("%Y-%m-%d").to_string(),
                diff_stat,
            });
        }
    }

    if changed_files.is_empty() && verified_date.is_some() {
        return None;
    }

    changed_files.sort_by(|a, b| a.file.cmp(&b.file));
    Some(DriftedSpecre {
        id: card.id.clone(),
        name: card.name.clone(),
        path: card.path.clone(),
        domain: card.domain.clone(),
        last_verified: card.last_verified.clone(),
        changed_files,
    })
}

fn print_human_output(drifted_list: &[DriftedSpecre], clean: usize, total: usize, grace_days: u64) {
    let drifted_count = total - clean;
    println!("Drift: {drifted_count} drifted, {clean} clean (grace: {grace_days} days)");
    if !drifted_list.is_empty() {
        println!();
        for d in drifted_list {
            println!("{}  {}", d.id, d.name);
            for f in &d.changed_files {
                if f.diff_stat.is_empty() {
                    println!("  {}  (modified: {})", f.file, f.last_modified);
                } else {
                    println!(
                        "  {}  (modified: {}, {})",
                        f.file, f.last_modified, f.diff_stat
                    );
                }
            }
        }
    }
}

/// Count the number of drifted stable specre cards.
///
/// Returns `None` if git is not available (not a git repo or git not installed).
/// Uses the grace period from `[drift] grace_days` in config (default: 0).
#[must_use]
pub fn count_drifts(cfg: &config::Config) -> Option<usize> {
    if verify_git_repo().is_err() {
        return None;
    }

    let grace_days = cfg.drift.as_ref().and_then(|d| d.grace_days).unwrap_or(0);
    let specre_dir = Path::new(&cfg.specre_dir);
    let all_cards = card::scan_specre_cards(specre_dir, &cfg.specre_dir);

    let stable_cards: Vec<&SpecreCard> = all_cards
        .iter()
        .filter(|c| c.status == Status::Stable)
        .collect();

    let marker_links = collect_marker_links(cfg);
    let mut count = 0usize;

    for card in &stable_cards {
        let card_content = fs::read_to_string(&card.path).unwrap_or_default();
        let mut related: HashSet<String> =
            extract_related_files(&card_content).into_iter().collect();

        if let Some(marker_files) = marker_links.get(&card.id) {
            for f in marker_files {
                related.insert(f.clone());
            }
        }

        if check_card_drift(card, &related, grace_days).is_some() {
            count += 1;
        }
    }

    Some(count)
}

/// # Errors
///
/// Returns [`SpecreError`] on config, I/O, serialization, or git failure.
/// Returns [`SpecreError::NonZeroExit`] when drifted specres are found.
pub fn execute(args: &DriftArgs, json: bool) -> Result<(), SpecreError> {
    let cfg = config::load()?;
    verify_git_repo()?;

    let grace_days = match &args.grace {
        Some(g) => parse_grace(g)?,
        None => cfg.drift.as_ref().and_then(|d| d.grace_days).unwrap_or(0),
    };

    let status_filter = match &args.status {
        Some(s) => s.parse::<Status>().map_err(SpecreError::InvalidArgument)?,
        None => Status::Stable,
    };

    let specre_dir = Path::new(&cfg.specre_dir);
    let all_cards = card::scan_specre_cards(specre_dir, &cfg.specre_dir);

    let cards: Vec<&SpecreCard> = all_cards
        .iter()
        .filter(|c| c.status == status_filter)
        .filter(|c| filter_by_target(c, args.target.as_deref(), &cfg.specre_dir))
        .filter(|c| args.domain.as_ref().is_none_or(|d| c.domain == *d))
        .collect();

    let marker_links = collect_marker_links(&cfg);
    let mut drifted_list = Vec::new();
    let total = cards.len();

    for card in &cards {
        let card_content = fs::read_to_string(&card.path).unwrap_or_default();
        let mut related: HashSet<String> =
            extract_related_files(&card_content).into_iter().collect();

        if let Some(marker_files) = marker_links.get(&card.id) {
            for f in marker_files {
                related.insert(f.clone());
            }
        }

        if let Some(drifted) = check_card_drift(card, &related, grace_days) {
            drifted_list.push(drifted);
        }
    }

    let clean = total - drifted_list.len();
    let has_drift = !drifted_list.is_empty();

    if json {
        let output = DriftOutput {
            drifted: drifted_list,
            clean,
            total,
            grace_days,
        };
        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{json_str}");
    } else {
        print_human_output(&drifted_list, clean, total, grace_days);
    }

    if has_drift {
        Err(SpecreError::NonZeroExit)
    } else {
        Ok(())
    }
}
