// @specre 01KHQBKWZY2D77XP7A50HGTZQ8
use super::SearchableCard;
use crate::status::Status;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const SUGGESTED_TERMS_LIMIT: usize = 10;
const GLOSSARY_FILE: &str = "glossary.toml";

#[derive(Serialize)]
pub struct Hint<'a> {
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

#[derive(Deserialize)]
struct Glossary {
    terms: Vec<String>,
}

pub fn load_glossary() -> Option<Vec<String>> {
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

pub const fn should_show_zero_hint(keywords: &[&str], glossary: Option<&Vec<String>>) -> bool {
    !keywords.is_empty() && (glossary.is_some() || keywords.len() >= 2)
}

pub fn build_truncation_hint<'a>(
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

pub fn build_zero_result_hint<'a>(
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
