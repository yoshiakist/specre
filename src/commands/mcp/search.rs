// @specre 01KHQKZ6H8FB46ESFXB03N85AN

use crate::card::{extract_domain, to_forward_slash};
use crate::parser::parse_frontmatter;
use crate::scanner::collect_md_files;
use crate::status::Status;
use chrono::NaiveDate;
use rmcp::{ErrorData as McpError, model::CallToolResult, model::Content};
use std::fs;
use std::path::Path;

use super::helpers::{load_config, parse_date_filter};
use super::tools::SearchToolRequest;

// ---------------------------------------------------------------------------
// Searchable card
// ---------------------------------------------------------------------------

pub struct SearchableCard {
    pub id: String,
    pub name: String,
    pub status: Status,
    pub domain: String,
    pub path: String,
    pub last_verified: Option<String>,
    pub content: String,
    pub excerpt: Option<String>,
}

// ---------------------------------------------------------------------------
// Tool: search
// ---------------------------------------------------------------------------

pub fn execute_search(req: &SearchToolRequest) -> Result<CallToolResult, McpError> {
    let cfg = load_config()?;

    // Validate status filter
    let status_filter: Option<Status> = match &req.status {
        Some(s) => {
            let parsed = s.parse::<Status>().map_err(|msg| {
                McpError::invalid_params(msg, None)
            });
            match parsed {
                Ok(st) => Some(st),
                Err(_) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "invalid status filter: '{s}'. Expected: draft, in-development, stable, deprecated"
                    ))]));
                }
            }
        }
        None => None,
    };

    // Parse date filters
    let verified_before = req
        .verified_before
        .as_deref()
        .map(|d| parse_date_filter(d, "verified_before"))
        .transpose()?;
    let verified_after = req
        .verified_after
        .as_deref()
        .map(|d| parse_date_filter(d, "verified_after"))
        .transpose()?;

    if let Some(limit) = req.limit
        && limit == 0
    {
        return Ok(CallToolResult::error(vec![Content::text(
            "limit must be a positive integer",
        )]));
    }

    let specre_dir = Path::new(&cfg.specre_dir);
    let or_mode = req.or.unwrap_or(false);

    // Scan cards with full content for text search
    let all_cards = scan_searchable_cards(specre_dir, &cfg.specre_dir);

    // Apply filters
    let filtered: Vec<&SearchableCard> = all_cards
        .iter()
        .filter(|c| {
            matches_search_filters(
                c,
                req.query.as_deref(),
                status_filter,
                req.domain.as_deref(),
                verified_before,
                verified_after,
                or_mode,
            )
        })
        .collect();

    let total = filtered.len();
    let results: Vec<serde_json::Value> = filtered
        .iter()
        .take(req.limit.unwrap_or(usize::MAX))
        .map(|c| {
            serde_json::json!({
                "id": c.id,
                "name": c.name,
                "status": c.status,
                "domain": c.domain,
                "path": c.path,
                "last_verified": c.last_verified,
                "excerpt": c.excerpt,
            })
        })
        .collect();

    let truncated = req.limit.is_some_and(|l| total > l);
    let result = serde_json::json!({
        "results": results,
        "total": total,
        "truncated": truncated,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ---------------------------------------------------------------------------
// Scanning & filtering
// ---------------------------------------------------------------------------

fn scan_searchable_cards(dir: &Path, specre_dir_str: &str) -> Vec<SearchableCard> {
    let mut cards = Vec::new();
    if !dir.exists() {
        return cards;
    }
    collect_md_files(dir, &mut |path| {
        let Ok(content) = fs::read_to_string(path) else {
            return;
        };
        let Ok(fm) = parse_frontmatter(&content) else {
            return;
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
    cards.sort_by(|a, b| a.domain.cmp(&b.domain).then(a.name.cmp(&b.name)));
    cards
}

fn extract_excerpt(content: &str) -> Option<String> {
    const EXCERPT_MAX_CHARS: usize = 200;

    // Skip front-matter
    let body = {
        let trimmed = content.trim_start_matches('\u{feff}');
        trimmed.strip_prefix("---").map_or(trimmed, |after_first| {
            after_first.find("\n---").map_or(trimmed, |end| {
                let rest = &after_first[end + 4..];
                rest.strip_prefix('\n').unwrap_or(rest)
            })
        })
    };

    let mut lines = Vec::new();
    let mut in_para = false;
    for line in body.lines() {
        let t = line.trim();
        if t.is_empty() {
            if in_para {
                break;
            }
            continue;
        }
        if t.starts_with("##") || t.starts_with("- ") {
            if in_para {
                break;
            }
            continue;
        }
        in_para = true;
        lines.push(t);
    }

    if lines.is_empty() {
        return None;
    }

    let joined = lines.join(" ");
    let char_count = joined.chars().count();
    if char_count > EXCERPT_MAX_CHARS {
        let truncated: String = joined.chars().take(EXCERPT_MAX_CHARS).collect();
        Some(format!("{truncated}\u{2026}"))
    } else {
        Some(joined)
    }
}

#[allow(clippy::fn_params_excessive_bools)]
fn matches_search_filters(
    card: &SearchableCard,
    query: Option<&str>,
    status_filter: Option<Status>,
    domain: Option<&str>,
    verified_before: Option<NaiveDate>,
    verified_after: Option<NaiveDate>,
    or_mode: bool,
) -> bool {
    // Text query filter
    if let Some(q) = query {
        let content_lower = card.content.to_lowercase();
        let keywords: Vec<&str> = q.split_whitespace().collect();
        if !keywords.is_empty() {
            let matched = if or_mode {
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
    if let Some(d) = domain
        && card.domain != d
    {
        return false;
    }
    // verified_before filter
    if let Some(before) = verified_before
        && let Some(lv) = &card.last_verified
        && let Ok(lv_date) = NaiveDate::parse_from_str(lv, "%Y-%m-%d")
        && lv_date >= before
    {
        return false;
    }
    // verified_after filter
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
