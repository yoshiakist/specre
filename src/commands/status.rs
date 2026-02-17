// @specre 01KHAN6JE712ZAKXPP97854PKJ
// @specre 01KHG0A2V4YXE918WMJCY7WFE8
use crate::cli::StatusArgs;
use crate::card::to_forward_slash;
use crate::parser::parse_frontmatter;
use crate::scanner::collect_md_files;
use crate::config;
use crate::error::SpecreError;
use crate::status::Status;
use chrono::{NaiveDate, Utc};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct StatusOutput {
    summary: StatusSummary,
    stale: Vec<StaleEntry>,
}

#[derive(Serialize)]
struct StatusSummary {
    draft: u32,
    in_development: u32,
    stable: u32,
    deprecated: u32,
    total: u32,
}

#[derive(Serialize)]
struct StaleEntry {
    name: String,
    path: String,
    reason: String,
}

/// # Errors
///
/// Returns [`SpecreError`] on config or serialization failure.
pub fn execute(args: &StatusArgs, json: bool) -> Result<(), SpecreError> {
    let config = config::load()?;
    let specre_dir = Path::new(&config.specre_dir);

    let today = Utc::now().date_naive();
    let threshold = args.threshold;

    let mut draft = 0u32;
    let mut in_development = 0u32;
    let mut stable = 0u32;
    let mut deprecated = 0u32;
    let mut stale_entries: Vec<StaleEntry> = Vec::new();

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
                Ok(fm) => match fm.status {
                    Status::Draft => draft += 1,
                    Status::InDevelopment => in_development += 1,
                    Status::Stable => {
                        stable += 1;
                        let reason = fm.last_verified.as_ref().map_or_else(
                            || Some("no last_verified".to_string()),
                            |date_str| {
                                NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_or_else(
                                    |_| {
                                        let rel_path = to_forward_slash(path);
                                        eprintln!(
                                            "Warning: invalid last_verified in {rel_path}: \"{date_str}\""
                                        );
                                        Some("invalid last_verified".to_string())
                                    },
                                    |date| {
                                        let days = (today - date).num_days();
                                        if days > i64::from(threshold) {
                                            Some(format!("{days} days"))
                                        } else {
                                            None
                                        }
                                    },
                                )
                            },
                        );
                        if let Some(reason) = reason {
                            stale_entries.push(StaleEntry {
                                name: fm.name,
                                path: to_forward_slash(path).into_owned(),
                                reason,
                            });
                        }
                    }
                    Status::Deprecated => deprecated += 1,
                },
                Err(e) => {
                    eprintln!(
                        "Warning: skipping '{}': {e}",
                        path.display()
                    );
                }
            }
        });
    }

    let total = draft + in_development + stable + deprecated;

    if json {
        let output = StatusOutput {
            summary: StatusSummary {
                draft,
                in_development,
                stable,
                deprecated,
                total,
            },
            stale: stale_entries,
        };
        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{json_str}");
    } else {
        println!("Status Summary:");
        println!("  draft:          {draft}");
        println!("  in-development: {in_development}");
        println!("  stable:         {stable}");
        println!("  deprecated:     {deprecated}");
        println!("  total:          {total}");

        if !stale_entries.is_empty() {
            println!();
            println!("Stale specres (last_verified > {threshold} days):");
            for entry in &stale_entries {
                println!("  {}  {}  ({})", entry.name, entry.path, entry.reason);
            }
        }
    }

    Ok(())
}
