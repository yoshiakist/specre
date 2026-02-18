// @specre 01KHK7MFZJZ12XFPQE4RHCBHQN
// @specre 01KHQJG96BS5STGSENPNDHEH1H
// @specre 01KHQKZ5M6N304YYJNW8VDKT4W
// @specre 01KHQKZ5VKTHSD483ZWK0RYPR9
// @specre 01KHQKZ633JHVDK0WADPPVP3CM
// @specre 01KHQKZ6AAMY6Y6AQB3VDVSF6Z
// @specre 01KHQKZ6H8FB46ESFXB03N85AN
// @specre 01KHQKZ6RE7Z3WEDZ54ZKHM6BM
// @specre 01KHQKZ6ZHSZX3GR2D7DS23XTE

use crate::card::{self, extract_domain, to_forward_slash};
use crate::commands::coverage::coverage_from_scan;
use crate::commands::orphans::{compute_orphans, orphans_from_scan};
use crate::commands::tag::comment_syntax;
use crate::parser::{extract_marker_ulid, parse_frontmatter};
use crate::scanner::{collect_md_files, scan_source_markers};
use crate::status::Status;
use crate::{config, template, ulid};
use chrono::{NaiveDate, Utc};
use rmcp::{
    ErrorData as McpError,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content},
    schemars, tool, tool_router,
};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct SpecreMcpServer {
    pub(crate) specre_dir: PathBuf,
    pub(crate) tool_router: ToolRouter<Self>,
}

// ---------------------------------------------------------------------------
// Request structs
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct NewToolRequest {
    /// Directory where the specre file will be created (e.g., "docs/specres/auth")
    target_dir: String,
    /// Specre name describing the behavior (default: "untitled")
    name: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct TagToolRequest {
    /// ULID to insert as a @specre marker (26 uppercase alphanumeric characters)
    ulid: String,
    /// Path to the source file where the marker will be inserted
    file: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct StatusToolRequest {
    /// Number of days after which a stable specre's `last_verified` is stale (default: 30)
    threshold: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct TraceToolRequest {
    /// ULID (26 uppercase alphanumeric characters) or file path to look up
    query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SearchToolRequest {
    /// Free-text substring to match against card content (case-insensitive, multi-keyword AND by default)
    query: Option<String>,
    /// Filter by status: draft, in-development, stable, deprecated
    status: Option<String>,
    /// Filter by domain (top-level directory under `specre_dir`)
    domain: Option<String>,
    /// Include only specres verified before this date (YYYY-MM-DD)
    verified_before: Option<String>,
    /// Include only specres verified on or after this date (YYYY-MM-DD)
    verified_after: Option<String>,
    /// Use OR logic for multi-keyword queries (default: false)
    or: Option<bool>,
    /// Return at most N results
    limit: Option<usize>,
}

// ---------------------------------------------------------------------------
// Tool implementations
// ---------------------------------------------------------------------------

#[tool_router]
impl SpecreMcpServer {
    pub fn new(specre_dir: PathBuf) -> Self {
        Self {
            specre_dir,
            tool_router: Self::tool_router(),
        }
    }

    // === new ===

    #[tool(name = "new", description = "Create a new specre card from a template, auto-generating a ULID")]
    #[allow(clippy::unused_self)]
    fn new_card(
        &self,
        Parameters(req): Parameters<NewToolRequest>,
    ) -> Result<CallToolResult, McpError> {
        let name = req.name.unwrap_or_else(|| "untitled".to_string());
        let target = Path::new(&req.target_dir);

        if target.is_file() {
            return Ok(CallToolResult::error(vec![Content::text(
                format!("'{}' is a file, not a directory", target.display()),
            )]));
        }

        let file_name = format!("{name}.md");
        let file_path = target.join(&file_name);

        if file_path.exists() {
            return Ok(CallToolResult::error(vec![Content::text(
                format!("'{}' already exists", file_path.display()),
            )]));
        }

        if !target.exists() {
            fs::create_dir_all(target).map_err(|e| {
                McpError::internal_error(
                    "failed to create directory",
                    Some(serde_json::json!({ "error": e.to_string() })),
                )
            })?;
        }

        let language = config::load_language();
        let id = ulid::generate();
        let content = template::render(&id, &name, &language);

        fs::write(&file_path, &content).map_err(|e| {
            McpError::internal_error(
                "failed to write specre card",
                Some(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

        let result = serde_json::json!({
            "id": id,
            "path": to_forward_slash(&file_path).as_ref(),
        });

        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    // === tag ===

    #[tool(name = "tag", description = "Insert a @specre marker into a source file at line 1")]
    #[allow(clippy::unused_self)]
    fn tag_file(
        &self,
        Parameters(req): Parameters<TagToolRequest>,
    ) -> Result<CallToolResult, McpError> {
        if !ulid::is_valid(&req.ulid) {
            return Ok(CallToolResult::error(vec![Content::text(
                "invalid ULID format. Expected 26 uppercase alphanumeric characters.",
            )]));
        }

        let file_path = Path::new(&req.file);

        if !file_path.exists() {
            return Ok(CallToolResult::error(vec![Content::text(
                format!("file not found: {}", to_forward_slash(file_path)),
            )]));
        }

        if file_path.is_dir() {
            return Ok(CallToolResult::error(vec![Content::text(
                format!(
                    "'{}' is a directory, not a file",
                    to_forward_slash(file_path)
                ),
            )]));
        }

        let content = fs::read_to_string(file_path).map_err(|e| {
            McpError::internal_error(
                "failed to read file",
                Some(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

        // Check if marker already exists
        let marker_pattern = format!("@specre {}", req.ulid);
        if content.contains(&marker_pattern) {
            let line = content
                .lines()
                .position(|l| l.contains(&marker_pattern))
                .map_or(1, |n| n + 1);

            let result = serde_json::json!({
                "id": req.ulid,
                "file": to_forward_slash(file_path).as_ref(),
                "line": line,
            });
            return Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]));
        }

        // Determine comment syntax from file extension
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let Some((prefix, suffix)) = comment_syntax(ext) else {
            return Ok(CallToolResult::error(vec![Content::text(
                format!("unsupported file extension '.{ext}' — comment syntax is unknown"),
            )]));
        };

        let marker_line = format!("{prefix}@specre {}{suffix}\n", req.ulid);
        let new_content = format!("{marker_line}{content}");

        fs::write(file_path, &new_content).map_err(|e| {
            McpError::internal_error(
                "failed to write file",
                Some(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

        let result = serde_json::json!({
            "id": req.ulid,
            "file": to_forward_slash(file_path).as_ref(),
            "line": 1,
        });

        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    // === index ===

    #[tool(
        name = "index",
        description = "Regenerate index.json and per-domain _INDEX.md files"
    )]
    #[allow(clippy::unused_self)]
    fn run_index(&self) -> Result<CallToolResult, McpError> {
        let cfg = load_config()?;
        let specre_dir = Path::new(&cfg.specre_dir);
        let cards = card::scan_specre_cards(specre_dir, &cfg.specre_dir);
        let scan = scan_source_markers(&cfg.source_dirs, cfg.target_extensions.as_deref());

        // Build source_refs array for index.json
        let source_refs: Vec<serde_json::Value> = scan
            .all_markers
            .iter()
            .map(|m| {
                serde_json::json!({
                    "specre_id": m.ulid,
                    "file": m.file,
                    "line": m.line,
                })
            })
            .collect();

        let index = serde_json::json!({
            "version": 1,
            "generated_at": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "specres": cards,
            "source_refs": source_refs,
        });

        // Write index.json
        let index_path = specre_dir.join("index.json");
        let index_json = serde_json::to_string_pretty(&index).map_err(|e| {
            McpError::internal_error(
                "failed to serialize index",
                Some(serde_json::json!({ "error": e.to_string() })),
            )
        })?;
        write_file(&index_path, &index_json)?;

        // Write per-domain _INDEX.md
        let specre_count = cards.len();
        let md_files = generate_domain_indexes(specre_dir, &cfg.specre_dir, &cards)?;

        let result = serde_json::json!({
            "index_file": format!("{}/index.json", cfg.specre_dir),
            "specre_count": specre_count,
            "source_ref_count": scan.all_markers.len(),
            "index_md_files": md_files,
        });

        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    // === status ===

    #[tool(
        name = "status",
        description = "Report specre counts by status and flag stale last_verified dates"
    )]
    #[allow(clippy::unused_self)]
    fn run_status(
        &self,
        Parameters(req): Parameters<StatusToolRequest>,
    ) -> Result<CallToolResult, McpError> {
        let cfg = load_config()?;
        let specre_dir = Path::new(&cfg.specre_dir);
        let threshold = req.threshold.unwrap_or(30);
        let today = Utc::now().date_naive();

        let mut draft = 0u32;
        let mut in_development = 0u32;
        let mut stable = 0u32;
        let mut deprecated = 0u32;
        let mut stale_entries: Vec<serde_json::Value> = Vec::new();

        if specre_dir.exists() {
            collect_md_files(specre_dir, &mut |path| {
                let Ok(content) = fs::read_to_string(path) else {
                    return;
                };
                let Ok(fm) = parse_frontmatter(&content) else {
                    return;
                };
                match fm.status {
                    Status::Draft => draft += 1,
                    Status::InDevelopment => in_development += 1,
                    Status::Stable => {
                        stable += 1;
                        let reason = fm.last_verified.as_ref().map_or_else(
                            || Some("no last_verified".to_string()),
                            |date_str| {
                                NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_or_else(
                                    |_| Some("invalid last_verified".to_string()),
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
                            stale_entries.push(serde_json::json!({
                                "name": fm.name,
                                "path": to_forward_slash(path).as_ref(),
                                "reason": reason,
                            }));
                        }
                    }
                    Status::Deprecated => deprecated += 1,
                }
            });
        }

        let total = draft + in_development + stable + deprecated;
        let result = serde_json::json!({
            "summary": {
                "draft": draft,
                "in_development": in_development,
                "stable": stable,
                "deprecated": deprecated,
                "total": total,
            },
            "stale": stale_entries,
        });

        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    // === trace ===

    #[tool(
        name = "trace",
        description = "Bidirectional traceability lookup by ULID or file path"
    )]
    #[allow(clippy::unused_self)]
    fn run_trace(
        &self,
        Parameters(req): Parameters<TraceToolRequest>,
    ) -> Result<CallToolResult, McpError> {
        if ulid::is_valid(&req.query) {
            trace_by_ulid(&req.query)
        } else {
            trace_by_file(&req.query)
        }
    }

    // === orphans ===

    #[tool(
        name = "orphans",
        description = "Detect unlinked specre cards and dangling @specre markers"
    )]
    #[allow(clippy::unused_self)]
    fn run_orphans(&self) -> Result<CallToolResult, McpError> {
        let cfg = load_config()?;
        let result = compute_orphans(
            &cfg.specre_dir,
            &cfg.source_dirs,
            cfg.target_extensions.as_deref(),
        );

        let json = serde_json::to_value(&result).map_err(|e| {
            McpError::internal_error(
                "failed to serialize orphans result",
                Some(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

        Ok(CallToolResult::success(vec![Content::text(
            json.to_string(),
        )]))
    }

    // === search ===

    #[tool(
        name = "search",
        description = "Search specre cards by text query and structured filters"
    )]
    #[allow(clippy::unused_self)]
    fn run_search(
        &self,
        Parameters(req): Parameters<SearchToolRequest>,
    ) -> Result<CallToolResult, McpError> {
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

    // === coverage ===

    #[tool(
        name = "coverage",
        description = "Report the percentage of source files covered by @specre tags"
    )]
    #[allow(clippy::unused_self)]
    fn run_coverage(&self) -> Result<CallToolResult, McpError> {
        let cfg = load_config()?;
        let cov = crate::commands::coverage::compute_coverage(
            &cfg.source_dirs,
            cfg.target_extensions.as_deref(),
        );

        let coverage_ratio = if cov.total == 0 {
            0.0
        } else {
            cov.tagged as f64 / cov.total as f64
        };

        let result = serde_json::json!({
            "total": cov.total,
            "tagged": cov.tagged,
            "coverage": coverage_ratio,
            "uncovered": cov.uncovered,
        });

        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    // === health-check ===

    #[tool(
        name = "health-check",
        description = "Comprehensive health check for AI agent preflight"
    )]
    #[allow(clippy::unused_self)]
    fn run_health_check(&self) -> Result<CallToolResult, McpError> {
        let cfg = load_config()?;

        let hc = cfg.health_check.as_ref();
        let t_coverage = hc.and_then(|h| h.coverage).unwrap_or(0.90);
        let t_orphans = hc.and_then(|h| h.orphans).unwrap_or(5);
        let t_index_age = hc.and_then(|h| h.index_age_hours).unwrap_or(24.0);

        // Single scan for both coverage and orphans
        let scan = scan_source_markers(&cfg.source_dirs, cfg.target_extensions.as_deref());

        let cov = coverage_from_scan(&scan);
        let coverage_ratio = if cov.total == 0 {
            0.0
        } else {
            ((cov.tagged as f64 / cov.total as f64) * 100.0).round() / 100.0
        };

        let orphan_result = orphans_from_scan(&cfg.specre_dir, &scan);
        let orphan_count = orphan_result.orphan_count() + orphan_result.dangling_count();

        let index_age_hours = get_index_age_hours(&cfg.specre_dir);

        let coverage_ok = coverage_ratio >= t_coverage;
        let orphans_ok = orphan_count <= t_orphans;
        let index_ok = index_age_hours.is_some_and(|age| age <= t_index_age);
        let healthy = coverage_ok && orphans_ok && index_ok;

        let result = serde_json::json!({
            "healthy": healthy,
            "coverage": coverage_ratio,
            "orphans": orphan_count,
            "index_age_hours": index_age_hours,
            "thresholds": {
                "coverage": t_coverage,
                "orphans": t_orphans,
                "index_age_hours": t_index_age,
            },
        });

        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn load_config() -> Result<config::Config, McpError> {
    config::load().map_err(|e| {
        McpError::internal_error(
            "failed to load specre.toml",
            Some(serde_json::json!({ "error": e.to_string() })),
        )
    })
}

fn write_file(path: &Path, content: &str) -> Result<(), McpError> {
    fs::write(path, content).map_err(|e| {
        McpError::internal_error(
            "failed to write file",
            Some(serde_json::json!({ "error": e.to_string(), "path": path.display().to_string() })),
        )
    })
}

fn generate_domain_indexes(
    specre_dir: &Path,
    specre_dir_str: &str,
    cards: &[card::SpecreCard],
) -> Result<Vec<String>, McpError> {
    let mut by_domain: BTreeMap<String, Vec<&card::SpecreCard>> = BTreeMap::new();
    for entry in cards {
        by_domain
            .entry(entry.domain.clone())
            .or_default()
            .push(entry);
    }

    let prefix = if specre_dir_str.ends_with('/') {
        specre_dir_str.to_string()
    } else {
        format!("{specre_dir_str}/")
    };

    let mut generated = Vec::new();
    for (domain, entries) in &by_domain {
        let domain_dir = specre_dir.join(domain);
        let index_path = domain_dir.join("_INDEX.md");
        let domain_prefix = format!("{prefix}{domain}/");

        let mut md = format!(
            "# {domain}\n\n| Name | Status | Last Verified |\n|------|--------|---------------|\n"
        );
        for entry in entries {
            let rel = entry
                .path
                .strip_prefix(&domain_prefix)
                .unwrap_or(&entry.path);
            let lv = entry.last_verified.as_deref().unwrap_or("-");
            let _ = writeln!(
                md,
                "| [{}]({rel}) | {} | {lv} |",
                entry.name, entry.status
            );
        }

        write_file(&index_path, &md)?;

        let rel_path =
            to_forward_slash(&PathBuf::from(specre_dir_str).join(domain).join("_INDEX.md"))
                .into_owned();
        generated.push(rel_path);
    }

    Ok(generated)
}

fn trace_by_ulid(ulid_str: &str) -> Result<CallToolResult, McpError> {
    let cfg = load_config()?;
    let specre_dir = Path::new(&cfg.specre_dir);

    // Find specre card with matching id
    let cards = card::scan_specre_cards(specre_dir, &cfg.specre_dir);
    let specre_path = cards.iter().find(|c| c.id == ulid_str).map(|c| &c.path);

    // Find source references
    let scan = scan_source_markers(&cfg.source_dirs, cfg.target_extensions.as_deref());
    let source_refs: Vec<serde_json::Value> = scan
        .all_markers
        .iter()
        .filter(|m| m.ulid == ulid_str)
        .map(|m| serde_json::json!({ "file": m.file, "line": m.line }))
        .collect();

    let result = serde_json::json!({
        "specre": specre_path,
        "source_refs": source_refs,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn trace_by_file(file_path: &str) -> Result<CallToolResult, McpError> {
    let normalized = file_path.replace('\\', "/");
    let path = Path::new(&normalized);

    if !path.exists() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "file not found: {normalized}"
        ))]));
    }

    let content = fs::read_to_string(path).map_err(|e| {
        McpError::internal_error(
            "failed to read file",
            Some(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // Extract all marker ULIDs (deduplicated, preserving order)
    let mut ulids: Vec<String> = Vec::new();
    for line in content.lines() {
        if let Some(found) = extract_marker_ulid(line)
            && !ulids.iter().any(|u| u == found)
        {
            ulids.push(found.to_string());
        }
    }

    // Resolve ULID → specre card path
    let cfg = load_config()?;
    let specre_dir = Path::new(&cfg.specre_dir);
    let cards = card::scan_specre_cards(specre_dir, &cfg.specre_dir);

    let specres: Vec<serde_json::Value> = ulids
        .iter()
        .map(|u| {
            let card_path = cards.iter().find(|c| c.id == *u).map(|c| c.path.as_str());
            serde_json::json!({ "id": u, "path": card_path })
        })
        .collect();

    let result = serde_json::json!({
        "file": normalized,
        "specres": specres,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn get_index_age_hours(specre_dir: &str) -> Option<f64> {
    let index_path = Path::new(specre_dir).join("index.json");
    let content = fs::read_to_string(&index_path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
    let generated_at = parsed["generated_at"].as_str()?;
    let generated = chrono::DateTime::parse_from_rfc3339(generated_at).ok()?;
    let age = Utc::now().signed_duration_since(generated);
    let hours = age.num_minutes() as f64 / 60.0;
    Some((hours * 10.0).round() / 10.0)
}

fn parse_date_filter(date: &str, field: &str) -> Result<NaiveDate, McpError> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| {
        McpError::invalid_params(
            format!("invalid {field} date: '{date}'. Expected YYYY-MM-DD."),
            None,
        )
    })
}

// ---------------------------------------------------------------------------
// Search helpers
// ---------------------------------------------------------------------------

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
