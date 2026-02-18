// @specre 01KHFEA9QVV4A127VCRJY97A68
// @specre 01KHG0A2V4YXE918WMJCY7WFE8
use crate::cli::CoverageArgs;
use crate::config;
use crate::error::SpecreError;
use crate::scanner::{SourceScanResult, scan_source_markers};
use serde::Serialize;

#[derive(Serialize)]
struct CoverageOutput {
    total: usize,
    tagged: usize,
    coverage: f64,
    uncovered: Vec<String>,
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

    if json {
        let coverage_ratio = if result.total == 0 {
            0.0
        } else {
            result.tagged as f64 / result.total as f64
        };
        let output = CoverageOutput {
            total: result.total,
            tagged: result.tagged,
            coverage: coverage_ratio,
            uncovered: result.uncovered,
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
            for path in &result.uncovered {
                println!("  {path}");
            }
        }
    }

    Ok(())
}
