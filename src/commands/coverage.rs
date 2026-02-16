// @specre 01KHFEA9QVV4A127VCRJY97A68
// @specre 01KHG0A2V4YXE918WMJCY7WFE8
use crate::cli::CoverageArgs;
use crate::commands::index::{collect_all_files, extract_marker_ulid, to_forward_slash};
use crate::config;
use crate::error::SpecreError;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct CoverageOutput {
    total: usize,
    tagged: usize,
    coverage: f64,
    uncovered: Vec<String>,
}

pub struct CoverageResult {
    pub total: usize,
    pub tagged: usize,
    pub uncovered: Vec<String>,
}

pub fn compute_coverage(
    source_dirs: &[String],
    target_extensions: Option<&[String]>,
) -> CoverageResult {
    let mut total = 0usize;
    let mut tagged = 0usize;
    let mut uncovered = Vec::new();

    for dir_str in source_dirs {
        let dir = Path::new(dir_str);
        if !dir.exists() {
            continue;
        }
        collect_all_files(dir, target_extensions, &mut |path| {
            total += 1;
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: failed to read '{}': {e}", path.display());
                    uncovered.push(to_forward_slash(path));
                    return;
                }
            };
            let has_marker = content
                .lines()
                .any(|line| extract_marker_ulid(line).is_some());
            if has_marker {
                tagged += 1;
            } else {
                uncovered.push(to_forward_slash(path));
            }
        });
    }

    uncovered.sort();

    CoverageResult {
        total,
        tagged,
        uncovered,
    }
}

pub fn execute(args: CoverageArgs, json: bool) -> Result<(), SpecreError> {
    let config = config::load()?;

    let extensions = args
        .ext
        .or(config.target_extensions);

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
            println!("Coverage: {}/{} files ({:.1}%)", result.tagged, result.total, pct);
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
