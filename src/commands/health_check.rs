// @specre 01KHFGVXWP100JXYBZTRJGMB9H
use crate::commands::coverage::coverage_from_scan;
use crate::commands::index::scan_source_markers;
use crate::commands::orphans::orphans_from_scan;
use crate::config;
use crate::error::SpecreError;
use chrono::Utc;
use serde::Serialize;
use std::fs;

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

fn get_index_age_hours(specre_dir: &str) -> Option<f64> {
    let index_path = std::path::Path::new(specre_dir).join("index.json");
    let content = match fs::read_to_string(&index_path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            eprintln!(
                "Warning: failed to read '{}': {e}",
                index_path.display()
            );
            return None;
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "Warning: failed to parse '{}': {e}",
                index_path.display()
            );
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
    // Round to one decimal place
    Some((hours * 10.0).round() / 10.0)
}

pub fn execute() -> Result<(), SpecreError> {
    let cfg = config::load()?;

    let hc = cfg.health_check.as_ref();
    let threshold_coverage = hc.and_then(|h| h.coverage).unwrap_or(0.90);
    let threshold_orphans = hc.and_then(|h| h.orphans).unwrap_or(5);
    let threshold_index_age = hc.and_then(|h| h.index_age_hours).unwrap_or(24.0);

    // Single scan for both coverage and orphans
    let scan = scan_source_markers(&cfg.source_dirs, cfg.target_extensions.as_deref());

    // Coverage
    let cov = coverage_from_scan(&scan);
    let coverage_ratio = if cov.total == 0 {
        0.0
    } else {
        // Round to two decimal places
        ((cov.tagged as f64 / cov.total as f64) * 100.0).round() / 100.0
    };

    // Orphans
    let orphan_result = orphans_from_scan(&cfg.specre_dir, &scan);
    let orphan_count = orphan_result.orphan_count() + orphan_result.dangling_count();

    // Index freshness
    let index_age_hours = get_index_age_hours(&cfg.specre_dir);

    // Healthy check
    let coverage_ok = coverage_ratio >= threshold_coverage;
    let orphans_ok = orphan_count <= threshold_orphans;
    let index_ok = index_age_hours
        .map(|age| age <= threshold_index_age)
        .unwrap_or(false);
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
