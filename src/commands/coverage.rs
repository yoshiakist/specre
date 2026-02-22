// @specre 01KHFEA9QVV4A127VCRJY97A68
// @specre 01KHG0A2V4YXE918WMJCY7WFE8
use crate::cli::CoverageArgs;
use crate::config;
use crate::error::SpecreError;
use crate::scanner::{SourceScanResult, scan_source_markers};
use serde::Serialize;

/// Maximum number of uncovered files shown in output.
const UNCOVERED_DISPLAY_LIMIT: usize = 30;

#[derive(Serialize)]
struct CoverageOutput {
    total: usize,
    tagged: usize,
    coverage: f64,
    uncovered: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uncovered_total: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    truncated: Option<bool>,
}

#[derive(Debug)]
pub struct CoverageResult {
    pub total: usize,
    pub tagged: usize,
    pub uncovered: Vec<String>,
}

/// Derives a [`CoverageResult`] by consuming a [`SourceScanResult`], avoiding clones.
#[must_use]
pub fn coverage_from_scan(scan: SourceScanResult) -> CoverageResult {
    CoverageResult {
        total: scan.total,
        tagged: scan.tagged,
        uncovered: scan.uncovered,
    }
}

/// Derives a [`CoverageResult`] by borrowing a [`SourceScanResult`].
///
/// Use this when the scan is shared with other consumers (e.g., `health-check`).
#[must_use]
pub fn coverage_from_scan_ref(scan: &SourceScanResult) -> CoverageResult {
    CoverageResult {
        total: scan.total,
        tagged: scan.tagged,
        uncovered: scan.uncovered.clone(),
    }
}

#[must_use]
pub fn compute_coverage(
    source_dirs: &[String],
    target_extensions: Option<&[String]>,
) -> CoverageResult {
    let scan = scan_source_markers(source_dirs, target_extensions);
    coverage_from_scan(scan)
}

/// # Errors
///
/// Returns [`SpecreError`] on config or serialization failure.
pub fn execute(args: CoverageArgs, json: bool) -> Result<(), SpecreError> {
    let config = config::load()?;

    let extensions = args.ext.or(config.target_extensions);

    let result = compute_coverage(&config.source_dirs, extensions.as_deref());

    let uncovered_total = result.uncovered.len();
    let is_truncated = uncovered_total > UNCOVERED_DISPLAY_LIMIT;

    if json {
        let coverage_ratio = if result.total == 0 {
            0.0
        } else {
            result.tagged as f64 / result.total as f64
        };
        let mut uncovered = result.uncovered;
        if is_truncated {
            uncovered.truncate(UNCOVERED_DISPLAY_LIMIT);
        }
        let output = CoverageOutput {
            total: result.total,
            tagged: result.tagged,
            coverage: coverage_ratio,
            uncovered,
            uncovered_total: is_truncated.then_some(uncovered_total),
            truncated: is_truncated.then_some(true),
        };
        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{json_str}");
    } else {
        if result.total == 0 {
            println!("Coverage: 0/0 files (N/A)");
        } else {
            let pct = result.tagged as f64 / result.total as f64 * 100.0;
            println!(
                "Coverage: {}/{} files ({:.1}%)",
                result.tagged, result.total, pct
            );
        }

        if !result.uncovered.is_empty() {
            println!();
            println!("Uncovered files:");
            for path in result.uncovered.iter().take(UNCOVERED_DISPLAY_LIMIT) {
                println!("  {path}");
            }
            if is_truncated {
                println!("  ... ({UNCOVERED_DISPLAY_LIMIT} of {uncovered_total} shown)");
            }
        }
    }

    Ok(())
}
