// @specre 01KHFTCYJN8YJMW2RNHJTAQV49
use crate::cli::SearchArgs;
use crate::commands::index::{collect_md_files, parse_frontmatter, to_forward_slash};
use crate::config;
use crate::error::SpecreError;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const VALID_STATUSES: &[&str] = &["draft", "in-development", "stable", "deprecated"];
const DEFAULT_MAX_RESULTS: usize = 10;
const EXCERPT_MAX_CHARS: usize = 200;

#[derive(Serialize)]
struct SearchOutput {
    results: Vec<SearchResult>,
    total: usize,
    truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<Hint>,
}

#[derive(Serialize)]
struct SearchResult {
    id: String,
    name: String,
    status: String,
    domain: String,
    path: String,
    last_verified: Option<String>,
    excerpt: Option<String>,
}

#[derive(Serialize)]
struct Hint {
    message: String,
    available_domains: Vec<String>,
    status_counts: BTreeMap<String, usize>,
}

struct CardData {
    id: String,
    name: String,
    status: String,
    domain: String,
    path: String,
    last_verified: Option<String>,
    content: String,
    excerpt: Option<String>,
}

pub fn execute(args: SearchArgs) -> Result<(), SpecreError> {
    // Validate inputs
    if let Some(ref s) = args.status {
        if !VALID_STATUSES.contains(&s.as_str()) {
            return Err(SpecreError::InvalidArgument(format!(
                "invalid status: {s}. Expected one of: draft, in-development, stable, deprecated."
            )));
        }
    }
    if let Some(ref d) = args.verified_before {
        validate_date(d)?;
    }
    if let Some(ref d) = args.verified_after {
        validate_date(d)?;
    }
    if let Some(limit) = args.limit {
        if limit == 0 {
            return Err(SpecreError::InvalidArgument(
                "--limit must be a positive integer.".to_string(),
            ));
        }
    }

    let cfg = config::load()?;
    let max_results = cfg
        .search
        .as_ref()
        .and_then(|s| s.max_results)
        .unwrap_or(DEFAULT_MAX_RESULTS);

    let specre_dir = Path::new(&cfg.specre_dir);
    let cards = scan_cards(specre_dir, &cfg.specre_dir);

    // Apply filters
    let filtered: Vec<CardData> = cards
        .into_iter()
        .filter(|card| {
            // Text query filter
            if let Some(ref q) = args.query {
                let q_lower = q.to_lowercase();
                if !card.content.to_lowercase().contains(&q_lower) {
                    return false;
                }
            }
            // Status filter
            if let Some(ref s) = args.status {
                if card.status != *s {
                    return false;
                }
            }
            // Domain filter
            if let Some(ref d) = args.domain {
                if card.domain != *d {
                    return false;
                }
            }
            // verified-before filter
            if let Some(ref before) = args.verified_before {
                match &card.last_verified {
                    Some(lv) => {
                        if lv >= before {
                            return false;
                        }
                    }
                    None => {} // No last_verified → include (never verified = "before" any date)
                }
            }
            // verified-after filter
            if let Some(ref after) = args.verified_after {
                match &card.last_verified {
                    Some(lv) => {
                        if lv < after {
                            return false;
                        }
                    }
                    None => return false, // No last_verified → exclude
                }
            }
            true
        })
        .collect();

    let total = filtered.len();

    // Determine truncation
    let (truncated, results, hint) = if let Some(limit) = args.limit {
        // --limit bypasses truncation
        let results: Vec<SearchResult> = filtered
            .into_iter()
            .take(limit)
            .map(card_to_result)
            .collect();
        (false, results, None)
    } else if total > max_results {
        // Truncate: return hint instead of results
        let hint = build_hint(&filtered);
        (true, Vec::new(), Some(hint))
    } else {
        let results: Vec<SearchResult> = filtered.into_iter().map(card_to_result).collect();
        (false, results, None)
    };

    let output = SearchOutput {
        results,
        total,
        truncated,
        hint,
    };

    let json = serde_json::to_string_pretty(&output)?;
    println!("{json}");

    Ok(())
}

fn validate_date(date: &str) -> Result<(), SpecreError> {
    if date.len() != 10 {
        return Err(SpecreError::InvalidArgument(format!(
            "invalid date format: {date}. Expected YYYY-MM-DD."
        )));
    }
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return Err(SpecreError::InvalidArgument(format!(
            "invalid date format: {date}. Expected YYYY-MM-DD."
        )));
    }
    if parts[0].len() != 4
        || parts[1].len() != 2
        || parts[2].len() != 2
        || parts[0].parse::<u32>().is_err()
        || parts[1].parse::<u32>().is_err()
        || parts[2].parse::<u32>().is_err()
    {
        return Err(SpecreError::InvalidArgument(format!(
            "invalid date format: {date}. Expected YYYY-MM-DD."
        )));
    }
    Ok(())
}

fn scan_cards(dir: &Path, specre_dir_str: &str) -> Vec<CardData> {
    let mut cards = Vec::new();
    if !dir.exists() {
        return cards;
    }

    let prefix = if specre_dir_str.ends_with('/') {
        specre_dir_str.to_string()
    } else {
        format!("{specre_dir_str}/")
    };

    collect_md_files(dir, &mut |path| {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let fm = match parse_frontmatter(&content) {
            Some(fm) => fm,
            None => return,
        };
        let rel_path = to_forward_slash(path);
        let domain = extract_domain(&rel_path, &prefix);
        let excerpt = extract_excerpt(&content);

        cards.push(CardData {
            id: fm.id,
            name: fm.name,
            status: fm.status,
            domain,
            path: rel_path,
            last_verified: fm.last_verified,
            content,
            excerpt,
        });
    });

    // Sort by domain then name
    cards.sort_by(|a, b| a.domain.cmp(&b.domain).then(a.name.cmp(&b.name)));
    cards
}

fn extract_domain(rel_path: &str, prefix: &str) -> String {
    let after = rel_path.strip_prefix(prefix).unwrap_or(rel_path);
    after.split('/').next().unwrap_or("unknown").to_string()
}

fn extract_excerpt(content: &str) -> Option<String> {
    // Skip front-matter
    let body = skip_frontmatter(content);

    // Find first prose paragraph: contiguous non-empty lines that are not
    // section headings (##) and not list items (- )
    let mut paragraph_lines: Vec<&str> = Vec::new();
    let mut in_paragraph = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if in_paragraph {
                break; // End of paragraph
            }
            continue;
        }
        if trimmed.starts_with("##") || trimmed.starts_with("- ") {
            if in_paragraph {
                break; // End of paragraph
            }
            continue;
        }
        // This is a prose line
        in_paragraph = true;
        paragraph_lines.push(trimmed);
    }

    if paragraph_lines.is_empty() {
        return None;
    }

    let joined = paragraph_lines.join(" ");
    let char_count = joined.chars().count();
    if char_count > EXCERPT_MAX_CHARS {
        let truncated: String = joined.chars().take(EXCERPT_MAX_CHARS).collect();
        Some(format!("{truncated}\u{2026}"))
    } else {
        Some(joined)
    }
}

fn skip_frontmatter(content: &str) -> &str {
    let content = content.trim_start_matches('\u{feff}'); // BOM
    if !content.starts_with("---") {
        return content;
    }
    let after_first = &content[3..];
    match after_first.find("\n---") {
        Some(end) => {
            let rest = &after_first[end + 4..];
            // Skip the newline after closing ---
            rest.strip_prefix('\n').unwrap_or(rest)
        }
        None => content,
    }
}

fn card_to_result(card: CardData) -> SearchResult {
    SearchResult {
        id: card.id,
        name: card.name,
        status: card.status,
        domain: card.domain,
        path: card.path,
        last_verified: card.last_verified,
        excerpt: card.excerpt,
    }
}

fn build_hint(cards: &[CardData]) -> Hint {
    let total = cards.len();

    // Collect unique domains (sorted)
    let mut domains: Vec<String> = cards.iter().map(|c| c.domain.clone()).collect();
    domains.sort();
    domains.dedup();

    // Count by status
    let mut status_counts: BTreeMap<String, usize> = BTreeMap::new();
    for card in cards {
        *status_counts.entry(card.status.clone()).or_insert(0) += 1;
    }

    Hint {
        message: format!(
            "Too many results ({total}). Refine your query with --status, --domain, or a more specific search term."
        ),
        available_domains: domains,
        status_counts,
    }
}
