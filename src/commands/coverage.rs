// @specre 01KHFEA9QVV4A127VCRJY97A68
use crate::cli::CoverageArgs;
use crate::commands::index::{collect_all_files, extract_marker_ulid, to_forward_slash};
use crate::config;
use std::fs;
use std::path::Path;

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
                Err(_) => {
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

pub fn execute(args: CoverageArgs) -> Result<(), String> {
    let config = config::load()?;

    let extensions = args
        .ext
        .or(config.target_extensions);

    let result = compute_coverage(&config.source_dirs, extensions.as_deref());

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

    Ok(())
}
