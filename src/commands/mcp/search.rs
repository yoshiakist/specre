// @specre 01KHQKZ6H8FB46ESFXB03N85AN

use crate::commands::search::hint;
use crate::commands::search::{scan_cards, SearchableCard};
use crate::status::Status;
use chrono::NaiveDate;
use rmcp::{ErrorData as McpError, model::CallToolResult, model::Content};
use std::path::Path;

use super::helpers::{load_config, parse_date_filter};
use super::tools::SearchToolRequest;

const DEFAULT_MAX_RESULTS: usize = 10;

// ---------------------------------------------------------------------------
// Validated filters extracted from the request
// ---------------------------------------------------------------------------

struct ValidatedFilters {
    status: Option<Status>,
    verified_before: Option<NaiveDate>,
    verified_after: Option<NaiveDate>,
}

fn validate_filters(req: &SearchToolRequest) -> Result<Result<ValidatedFilters, CallToolResult>, McpError> {
    let status: Option<Status> = match &req.status {
        Some(s) => match s.parse::<Status>() {
            Ok(st) => Some(st),
            Err(_) => {
                return Ok(Err(CallToolResult::error(vec![Content::text(format!(
                    "invalid status filter: '{s}'. Expected: draft, in-development, stable, deprecated"
                ))])));
            }
        },
        None => None,
    };

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
        return Ok(Err(CallToolResult::error(vec![Content::text(
            "limit must be a positive integer",
        )])));
    }

    Ok(Ok(ValidatedFilters {
        status,
        verified_before,
        verified_after,
    }))
}

// ---------------------------------------------------------------------------
// Tool: search
// ---------------------------------------------------------------------------

pub fn execute_search(req: &SearchToolRequest) -> Result<CallToolResult, McpError> {
    let cfg = load_config()?;

    let filters = match validate_filters(req)? {
        Ok(f) => f,
        Err(err_result) => return Ok(err_result),
    };

    let specre_dir = Path::new(&cfg.specre_dir);
    let or_mode = req.or.unwrap_or(false);
    let max_results = cfg
        .search
        .as_ref()
        .and_then(|s| s.max_results)
        .unwrap_or(DEFAULT_MAX_RESULTS);

    let all_cards = scan_cards(specre_dir, &cfg.specre_dir);
    let glossary = hint::load_glossary();
    let keywords: Vec<&str> = req
        .query
        .as_deref()
        .map(|q| q.split_whitespace().collect())
        .unwrap_or_default();

    let filtered: Vec<&SearchableCard> = all_cards
        .iter()
        .filter(|c| {
            matches_search_filters(
                c,
                req.query.as_deref(),
                filters.status,
                req.domain.as_deref(),
                filters.verified_before,
                filters.verified_after,
                or_mode,
            )
        })
        .collect();

    let total = filtered.len();

    let (truncated, results, hint_value) = req.limit.map_or_else(
        || {
            if total > max_results {
                let h = hint::build_truncation_hint(&filtered, glossary.as_ref(), &keywords);
                (true, Vec::new(), serde_json::to_value(&h).ok())
            } else if total == 0
                && hint::should_show_zero_hint(&keywords, glossary.as_ref())
            {
                let h = hint::build_zero_result_hint(&all_cards, glossary.as_ref(), &keywords);
                (false, Vec::new(), serde_json::to_value(&h).ok())
            } else {
                let r = filtered.iter().map(|c| card_to_json(c)).collect();
                (false, r, None)
            }
        },
        |limit| {
            let r = filtered.iter().take(limit).map(|c| card_to_json(c)).collect();
            (false, r, None)
        },
    );

    let mut result = serde_json::json!({
        "results": results,
        "total": total,
        "truncated": truncated,
    });
    if let Some(h) = hint_value {
        result["hint"] = h;
    }

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn card_to_json(card: &SearchableCard) -> serde_json::Value {
    serde_json::json!({
        "id": card.id,
        "name": card.name,
        "status": card.status,
        "domain": card.domain,
        "path": card.path,
        "last_verified": card.last_verified,
        "excerpt": card.excerpt,
    })
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
