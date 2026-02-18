// @specre 01KHFTCYJN8YJMW2RNHJTAQV49
// @specre 01KHQBKWZY2D77XP7A50HGTZQ8
use crate::card::{extract_domain, to_forward_slash};
use crate::cli::SearchArgs;
use crate::parser::parse_frontmatter;
use crate::scanner::collect_md_files;
use crate::config;
use crate::error::SpecreError;
use crate::status::Status;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
const DEFAULT_MAX_RESULTS: usize = 10;
const EXCERPT_MAX_CHARS: usize = 200;
const SUGGESTED_TERMS_LIMIT: usize = 10;
const GLOSSARY_FILE: &str = "glossary.toml";

#[derive(Serialize)]
struct SearchOutput<'a> {
    results: Vec<SearchResult<'a>>,
    total: usize,
    truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<Hint<'a>>,
}

#[derive(Serialize)]
struct SearchResult<'a> {
    id: &'a str,
    name: &'a str,
    status: Status,
    domain: &'a str,
    path: &'a str,
    last_verified: Option<&'a str>,
    excerpt: Option<&'a str>,
}

#[derive(Serialize)]
struct KeywordMatch<'a> {
    keyword: &'a str,
    match_count: usize,
}

#[derive(Serialize)]
struct SuggestedTerm<'a> {
    term: &'a str,
    match_count: usize,
}

#[derive(Serialize)]
struct Hint<'a> {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    available_domains: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_counts: Option<BTreeMap<Status, usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keyword_matches: Option<Vec<KeywordMatch<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggested_terms: Option<Vec<SuggestedTerm<'a>>>,
}

#[derive(Deserialize)]
struct Glossary {
    terms: Vec<String>,
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

/// # Errors
///
/// Returns [`SpecreError`] on invalid arguments, config, or serialization failure.
pub fn execute(args: &SearchArgs) -> Result<(), SpecreError> {
    // Validate inputs
    let status_filter: Option<Status> = match &args.status {
        Some(s) => Some(
            s.parse::<Status>()
                .map_err(SpecreError::InvalidArgument)?,
        ),
        None => None,
    };
    let verified_before: Option<NaiveDate> = args
        .verified_before
        .as_deref()
        .map(parse_date)
        .transpose()?;
    let verified_after: Option<NaiveDate> = args
        .verified_after
        .as_deref()
        .map(parse_date)
        .transpose()?;
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
    let all_cards = scan_cards(specre_dir, &cfg.specre_dir);
    let glossary = load_glossary();
    let keywords: Vec<&str> = args
        .query
        .as_deref()
        .map(|q| q.split_whitespace().collect())
        .unwrap_or_default();

    // Apply filters (use iter() to keep all_cards available for hint computation)
    let filtered: Vec<&SearchableCard> = all_cards
        .iter()
        .filter(|card| {
            matches_filters(card, args, status_filter, verified_before, verified_after)
        })
        .collect();

    let total = filtered.len();

    // Determine truncation
    let (truncated, results, hint) = args.limit.map_or_else(
        || {
            if total > max_results {
                let hint = build_truncation_hint(&filtered, glossary.as_ref(), &keywords);
                (true, Vec::new(), Some(hint))
            } else if total == 0 && should_show_zero_hint(&keywords, glossary.as_ref()) {
                let hint = build_zero_result_hint(&all_cards, glossary.as_ref(), &keywords);
                (false, Vec::new(), Some(hint))
            } else {
                let results: Vec<SearchResult<'_>> =
                    filtered.iter().map(|card| card_to_result(card)).collect();
                (false, results, None)
            }
        },
        |limit| {
            // --limit bypasses truncation
            let results: Vec<SearchResult<'_>> = filtered
                .iter()
                .take(limit)
                .map(|card| card_to_result(card))
                .collect();
            (false, results, None)
        },
    );

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

fn matches_filters(
    card: &SearchableCard,
    args: &SearchArgs,
    status_filter: Option<Status>,
    verified_before: Option<NaiveDate>,
    verified_after: Option<NaiveDate>,
) -> bool {
    // Text query filter (multi-keyword: AND by default, OR with --or)
    if let Some(ref q) = args.query {
        let content_lower = card.content.to_lowercase();
        let keywords: Vec<&str> = q.split_whitespace().collect();
        if !keywords.is_empty() {
            let matched = if args.or {
                keywords
                    .iter()
                    .any(|kw| content_lower.contains(&kw.to_lowercase()))
            } else {
                keywords
                    .iter()
                    .all(|kw| content_lower.contains(&kw.to_lowercase()))
            };
            if !matched {
                return false;
            }
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
    if let Some(before) = verified_before
        && let Some(lv) = &card.last_verified
        && let Ok(lv_date) = NaiveDate::parse_from_str(lv, "%Y-%m-%d")
        && lv_date >= before
    {
        return false;
    }
    // verified-after filter
    if let Some(after) = verified_after {
        match &card.last_verified {
            Some(lv) => match NaiveDate::parse_from_str(lv, "%Y-%m-%d") {
                Ok(lv_date) => {
                    if lv_date < after {
                        return false;
                    }
                }
                Err(_) => return false,
            },
            None => return false,
        }
    }
    true
}

fn parse_date(date: &str) -> Result<NaiveDate, SpecreError> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| {
        SpecreError::InvalidArgument(format!("invalid date format: {date}. Expected YYYY-MM-DD."))
    })
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
    after_first.find("\n---").map_or(content, |end| {
        let rest = &after_first[end + 4..];
        // Skip the newline after closing ---
        rest.strip_prefix('\n').unwrap_or(rest)
    })
}

fn card_to_result(card: &SearchableCard) -> SearchResult<'_> {
    SearchResult {
        id: &card.id,
        name: &card.name,
        status: card.status,
        domain: &card.domain,
        path: &card.path,
        last_verified: card.last_verified.as_deref(),
        excerpt: card.excerpt.as_deref(),
    }
}

fn load_glossary() -> Option<Vec<String>> {
    let path = Path::new(GLOSSARY_FILE);
    if !path.exists() {
        return None;
    }
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: failed to parse glossary.toml: {e}");
            return None;
        }
    };
    match toml::from_str::<Glossary>(&content) {
        Ok(g) => Some(g.terms),
        Err(e) => {
            eprintln!("Warning: failed to parse glossary.toml: {e}");
            None
        }
    }
}

const fn should_show_zero_hint(keywords: &[&str], glossary: Option<&Vec<String>>) -> bool {
    !keywords.is_empty() && (glossary.is_some() || keywords.len() >= 2)
}

fn build_truncation_hint<'a>(
    cards: &[&'a SearchableCard],
    glossary: Option<&'a Vec<String>>,
    keywords: &[&str],
) -> Hint<'a> {
    let total = cards.len();

    let domain_set: std::collections::BTreeSet<&str> =
        cards.iter().map(|c| c.domain.as_str()).collect();

    let mut status_counts: BTreeMap<Status, usize> = BTreeMap::new();
    for card in cards {
        *status_counts.entry(card.status).or_insert(0) += 1;
    }

    let suggested_terms = glossary.and_then(|terms| {
        let contents: Vec<&str> = cards.iter().map(|c| c.content.as_str()).collect();
        let result = compute_suggested_terms(terms, keywords, &contents, true);
        if result.is_empty() { None } else { Some(result) }
    });

    Hint {
        message: format!(
            "Too many results ({total}). Refine your query with --status, --domain, or a more specific search term."
        ),
        available_domains: Some(domain_set.into_iter().collect()),
        status_counts: Some(status_counts),
        keyword_matches: None,
        suggested_terms,
    }
}

fn build_zero_result_hint<'a>(
    all_cards: &[SearchableCard],
    glossary: Option<&'a Vec<String>>,
    keywords: &[&'a str],
) -> Hint<'a> {
    let keyword_matches = compute_keyword_matches(keywords, all_cards);

    let suggested_terms = glossary.and_then(|terms| {
        let contents: Vec<&str> = all_cards.iter().map(|c| c.content.as_str()).collect();
        let result = compute_suggested_terms(terms, keywords, &contents, false);
        if result.is_empty() { None } else { Some(result) }
    });

    let message = if glossary.is_some() {
        "No results found. Consider adjusting your query."
    } else {
        "No results found. Consider removing or replacing some keywords."
    }
    .to_string();

    Hint {
        message,
        available_domains: None,
        status_counts: None,
        keyword_matches: Some(keyword_matches),
        suggested_terms,
    }
}

fn compute_keyword_matches<'a>(
    keywords: &[&'a str],
    all_cards: &[SearchableCard],
) -> Vec<KeywordMatch<'a>> {
    let lowered: Vec<String> = all_cards.iter().map(|c| c.content.to_lowercase()).collect();

    let mut matches: Vec<KeywordMatch<'a>> = keywords
        .iter()
        .map(|kw| {
            let kw_lower = kw.to_lowercase();
            let count = lowered
                .iter()
                .filter(|c| c.contains(&kw_lower))
                .count();
            KeywordMatch {
                keyword: kw,
                match_count: count,
            }
        })
        .collect();

    matches.sort_by(|a, b| b.match_count.cmp(&a.match_count));
    matches
}

fn compute_suggested_terms<'a>(
    glossary_terms: &'a [String],
    keywords: &[&str],
    contents: &[&str],
    exclude_total_match: bool,
) -> Vec<SuggestedTerm<'a>> {
    let total = contents.len();
    let keywords_lower: Vec<String> = keywords.iter().map(|k| k.to_lowercase()).collect();
    let lowered: Vec<String> = contents.iter().map(|c| c.to_lowercase()).collect();

    let mut terms: Vec<SuggestedTerm<'a>> = glossary_terms
        .iter()
        .filter(|t| !keywords_lower.contains(&t.to_lowercase()))
        .map(|t| {
            let term_lower = t.to_lowercase();
            let count = lowered
                .iter()
                .filter(|c| c.contains(&term_lower))
                .count();
            SuggestedTerm {
                term: t.as_str(),
                match_count: count,
            }
        })
        .filter(|st| st.match_count > 0 && (!exclude_total_match || st.match_count < total))
        .collect();

    terms.sort_by(|a, b| b.match_count.cmp(&a.match_count));
    terms.truncate(SUGGESTED_TERMS_LIMIT);
    terms
}
