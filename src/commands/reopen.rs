// @specre 01KNK6TW0DJQ8TE4FF2ZBC6V67
use crate::card::{self, SpecreCard};
use crate::cli::ReopenArgs;
use crate::config;
use crate::error::SpecreError;
use crate::status::Status;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct ReopenOutput {
    reopened: ReopenedSpecre,
}

#[derive(Serialize)]
struct ReopenedSpecre {
    id: String,
    name: String,
    path: String,
    previous_status: String,
    new_status: String,
    last_verified: Option<String>,
}

/// Rewrite front-matter to set `status` to the given value.
fn rewrite_status(content: &str, new_status: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let has_trailing_newline = content.ends_with('\n');

    let mut result_lines: Vec<String> = Vec::with_capacity(lines.len());
    // Find the closing --- delimiter to know front-matter boundary
    let closing_idx = lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, line)| line.trim() == "---")
        .map(|(i, _)| i);

    for (i, line) in lines.iter().enumerate() {
        if closing_idx.is_some_and(|ci| i < ci) && line.trim().starts_with("status:") {
            result_lines.push(format!("status: \"{new_status}\""));
        } else {
            result_lines.push((*line).to_string());
        }
    }

    let mut result = result_lines.join("\n");
    if has_trailing_newline {
        result.push('\n');
    }
    result
}

fn find_card<'a>(cards: &'a [SpecreCard], ulid: &str) -> Option<&'a SpecreCard> {
    cards.iter().find(|c| c.id == ulid)
}

/// # Errors
///
/// Returns [`SpecreError`] on config, I/O, or serialization failure.
pub fn execute(args: &ReopenArgs, json: bool) -> Result<(), SpecreError> {
    let cfg = config::load()?;
    let specre_dir = Path::new(&cfg.specre_dir);
    let all_cards = card::scan_specre_cards(specre_dir, &cfg.specre_dir);

    let card = find_card(&all_cards, &args.ulid).ok_or_else(|| {
        SpecreError::InvalidArgument(format!("no specre found with id '{}'", args.ulid))
    })?;

    if card.status != Status::Stable {
        return Err(SpecreError::InvalidArgument(format!(
            "specre '{}' has status '{}', only 'stable' specres can be reopened",
            args.ulid, card.status
        )));
    }

    let content = fs::read_to_string(&card.path).map_err(|e| SpecreError::Io {
        path: card.path.clone().into(),
        source: e,
    })?;

    let new_content = rewrite_status(&content, "in-development");

    fs::write(&card.path, new_content).map_err(|e| SpecreError::Io {
        path: card.path.clone().into(),
        source: e,
    })?;

    if json {
        let output = ReopenOutput {
            reopened: ReopenedSpecre {
                id: card.id.clone(),
                name: card.name.clone(),
                path: card.path.clone(),
                previous_status: card.status.to_string(),
                new_status: "in-development".to_string(),
                last_verified: card.last_verified.clone(),
            },
        };
        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{json_str}");
    } else {
        println!("Reopened: {}  {}  ({})", card.id, card.name, card.path);
        println!("  {} -> in-development", card.status);
    }

    Ok(())
}
