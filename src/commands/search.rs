// @specre 01KHFTCYJN8YJMW2RNHJTAQV49
use crate::card::{extract_domain, to_forward_slash};
use crate::cli::SearchArgs;
use crate::parser::parse_frontmatter;
use crate::scanner::collect_md_files;
use crate::config;
use crate::error::SpecreError;
use crate::status::Status;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
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
    status: Status,
    domain: String,
    path: String,
    last_verified: Option<String>,
    excerpt: Option<String>,
}

#[derive(Serialize)]
struct Hint {
    message: String,
    available_domains: Vec<String>,
    status_counts: BTreeMap<Status, usize>,
}

struct SearchableCard {
    id: String,
    name: String,
    status: Status,
    domain: String,
    path: String,
    last_verified: Option<String>,
    content: String,
    excerpt: Option<String>,
}

pub fn execute(args: SearchArgs) -> Result<(), SpecreError> {
    // Validate inputs
    let status_filter: Option<Status> = match &args.status {
        Some(s) => Some(
            s.parse::<Status>()
                .map_err(SpecreError::InvalidArgument)?,
        ),
        None => None,
    };
    if let Some(ref d) = args.verified_before {
        validate_date(d)?;
    }
    if let Some(ref d) = args.verified_after {
        validate_date(d)?;
    }
    if let Some(limit) = args.limit
        && limit == 0
    {
        return Err(SpecreError::InvalidArgument(
            "--limit must be a positive integer.".to_string(),
        ));
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
    let filtered: Vec<SearchableCard> = cards
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
            if let Some(s) = status_filter
                && card.status != s
            {
                return false;
            }
            // Domain filter
            if let Some(ref d) = args.domain
                && card.domain != *d
            {
                return false;
            }
            // verified-before filter
            if let Some(ref before) = args.verified_before
                && let Some(lv) = &card.last_verified
                && lv >= before
            {
                return false;
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
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| {
        SpecreError::InvalidArgument(format!("invalid date format: {date}. Expected YYYY-MM-DD."))
    })?;
    Ok(())
}

fn scan_cards(dir: &Path, specre_dir_str: &str) -> Vec<SearchableCard> {
    let mut cards = Vec::new();
    if !dir.exists() {
        return cards;
    }

    collect_md_files(dir, &mut |path| {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: failed to read '{}': {e}", path.display());
                return;
            }
        };
        let fm = match parse_frontmatter(&content) {
            Ok(fm) => fm,
            Err(e) => {
                eprintln!(
                    "Warning: skipping '{}': {e}",
                    path.display()
                );
                return;
            }
        };
        let rel_path = to_forward_slash(path);
        let domain = extract_domain(&rel_path, specre_dir_str).to_owned();
        let excerpt = extract_excerpt(&content);

        cards.push(SearchableCard {
            id: fm.id,
            name: fm.name,
            status: fm.status,
            domain,
            path: rel_path.into_owned(),
            last_verified: fm.last_verified,
            content,
            excerpt,
        });
    });

    // Sort by domain then name
    cards.sort_by(|a, b| a.domain.cmp(&b.domain).then(a.name.cmp(&b.name)));
    cards
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

fn card_to_result(card: SearchableCard) -> SearchResult {
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

fn build_hint(cards: &[SearchableCard]) -> Hint {
    let total = cards.len();

    // Collect unique domains (BTreeSet gives sorted + deduplicated without cloning)
    let domain_set: std::collections::BTreeSet<&str> =
        cards.iter().map(|c| c.domain.as_str()).collect();

    // Count by status
    let mut status_counts: BTreeMap<Status, usize> = BTreeMap::new();
    for card in cards {
        *status_counts.entry(card.status).or_insert(0) += 1;
    }

    Hint {
        message: format!(
            "Too many results ({total}). Refine your query with --status, --domain, or a more specific search term."
        ),
        available_domains: domain_set.into_iter().map(String::from).collect(),
        status_counts,
    }
}
