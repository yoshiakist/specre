// @specre 01KJ4NJW28F072X64SS68P3N08
// @specre 01KHFGVXWP100JXYBZTRJGMB9H
use crate::card;
use crate::commands::coverage::coverage_from_scan_ref;
use crate::commands::orphans::orphans_from_scan_ref;
use crate::config;
use crate::error::SpecreError;
use crate::scanner::{SourceScanResult, scan_source_markers};
use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct HealthCheckResult {
    healthy: bool,
    coverage: f64,
    orphans: usize,
    index_age_hours: Option<f64>,
    thresholds: Thresholds,
}

#[derive(Serialize)]
struct Thresholds {
    coverage: f64,
    orphans: usize,
    index_age_hours: f64,
}

struct IndexInfo {
    age_hours: f64,
    json: serde_json::Value,
}

fn load_index(specre_dir: &str) -> Option<IndexInfo> {
    let index_path = Path::new(specre_dir).join("index.json");
    let content = match fs::read_to_string(&index_path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            eprintln!("Warning: failed to read '{}': {e}", index_path.display());
            return None;
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Warning: failed to parse '{}': {e}", index_path.display());
            return None;
        }
    };
    let generated_at = parsed["generated_at"].as_str()?;
    let generated = match chrono::DateTime::parse_from_rfc3339(generated_at) {
        Ok(dt) => dt,
        Err(e) => {
            eprintln!(
                "Warning: invalid generated_at in '{}': {e}",
                index_path.display()
            );
            return None;
        }
    };
    let age = Utc::now().signed_duration_since(generated);
    let hours = age.num_minutes() as f64 / 60.0;
    let age_hours = (hours * 10.0).round() / 10.0;
    Some(IndexInfo {
        age_hours,
        json: parsed,
    })
}

/// Checks whether the existing `index.json` content matches what would be
/// regenerated from the current specre cards and source markers.
fn is_index_content_current(
    cfg: &config::Config,
    scan: &SourceScanResult,
    existing_json: &serde_json::Value,
) -> bool {
    // Build fresh specre cards
    let specre_dir = Path::new(&cfg.specre_dir);
    let fresh_specres = card::scan_specre_cards(specre_dir, &cfg.specre_dir);

    let Ok(fresh_specres_value) = serde_json::to_value(&fresh_specres) else {
        return false;
    };

    // Build fresh source_refs from scan results (same shape as index.rs SourceRef)
    let fresh_source_refs: Vec<serde_json::Value> = scan
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
    let fresh_source_refs_value = serde_json::Value::Array(fresh_source_refs);

    // Compare with existing index content
    existing_json.get("specres") == Some(&fresh_specres_value)
        && existing_json.get("source_refs") == Some(&fresh_source_refs_value)
}

/// # Errors
///
/// Returns [`SpecreError`] on config or serialization failure, or
/// [`SpecreError::NonZeroExit`] when health checks fail.
pub fn execute() -> Result<(), SpecreError> {
    let cfg = config::load()?;

    let hc = cfg.health_check.as_ref();
    let threshold_coverage = hc.and_then(|h| h.coverage).unwrap_or(0.90);
    let threshold_orphans = hc.and_then(|h| h.orphans).unwrap_or(5);
    let threshold_index_age = hc.and_then(|h| h.index_age_hours).unwrap_or(24.0);

    // Single scan for both coverage and orphans
    let scan = scan_source_markers(
        &cfg.source_dirs,
        cfg.target_extensions.as_deref(),
        cfg.exclude_patterns.as_deref(),
    );

    // Coverage
    let cov = coverage_from_scan_ref(&scan);
    let coverage_ratio = if cov.total == 0 {
        0.0
    } else {
        // Round to two decimal places
        ((cov.tagged as f64 / cov.total as f64) * 100.0).round() / 100.0
    };

    // Orphans
    let orphan_result = orphans_from_scan_ref(&cfg.specre_dir, &scan);
    let orphan_count = orphan_result.orphan_count() + orphan_result.dangling_count();

    // Index freshness (two-stage: timestamp check, then content comparison)
    let index_info = load_index(&cfg.specre_dir);
    let index_age_hours = index_info.as_ref().map(|i| i.age_hours);

    // Healthy check
    let coverage_ok = coverage_ratio >= threshold_coverage;
    let orphans_ok = orphan_count <= threshold_orphans;
    let index_ok = match &index_info {
        Some(info) if info.age_hours <= threshold_index_age => true,
        Some(info) => is_index_content_current(&cfg, &scan, &info.json),
        None => false,
    };
    let healthy = coverage_ok && orphans_ok && index_ok;

    let result = HealthCheckResult {
        healthy,
        coverage: coverage_ratio,
        orphans: orphan_count,
        index_age_hours,
        thresholds: Thresholds {
            coverage: threshold_coverage,
            orphans: threshold_orphans,
            index_age_hours: threshold_index_age,
        },
    };

    let json = serde_json::to_string_pretty(&result)?;
    println!("{json}");

    if healthy {
        Ok(())
    } else {
        Err(SpecreError::NonZeroExit)
    }
}
