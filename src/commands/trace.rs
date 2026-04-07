// @specre 01KJ4NJW28F072X64SS68P3N08
// @specre 01KHB48DYZDN8GHXPX7MSYJ1NZ
// @specre 01KHG0A2V4YXE918WMJCY7WFE8
use crate::card::to_forward_slash;
use crate::cli::TraceArgs;
use crate::config;
use crate::error::SpecreError;
use crate::parser::{extract_marker_ulid, parse_frontmatter};
use crate::scanner::{collect_all_files, collect_md_files, compile_exclude_patterns};
use crate::ulid;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;

#[derive(Serialize)]
struct TraceByUlidOutput {
    specre: Option<String>,
    source_refs: Vec<SourceRefOutput>,
}

#[derive(Serialize)]
struct SourceRefOutput {
    file: String,
    line: usize,
}

#[derive(Serialize)]
struct TraceByFileOutput {
    file: String,
    specres: Vec<SpecreRefOutput>,
}

#[derive(Serialize)]
struct SpecreRefOutput {
    id: String,
    path: Option<String>,
}

/// # Errors
///
/// Returns [`SpecreError`] on config, I/O, or serialization failure, or
/// [`SpecreError::NonZeroExit`] when no trace results are found.
pub fn execute(args: &TraceArgs, json: bool) -> Result<(), SpecreError> {
    if ulid::is_valid(&args.query) {
        trace_by_ulid(&args.query, json)
    } else {
        trace_by_file(&args.query, json)
    }
}

fn trace_by_ulid(ulid: &str, json: bool) -> Result<(), SpecreError> {
    let config = config::load()?;
    let specre_dir = Path::new(&config.specre_dir);

    // Find specre file with matching id
    let mut specre_path: Option<String> = None;
    if specre_dir.exists() {
        collect_md_files(specre_dir, &mut |path| {
            if specre_path.is_some() {
                return;
            }
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: failed to read '{}': {e}", path.display());
                    return;
                }
            };
            if let Ok(fm) = parse_frontmatter(&content)
                && fm.id == ulid
            {
                specre_path = Some(to_forward_slash(path).into_owned());
            }
        });
    }

    // Find source references
    let exclude_set = compile_exclude_patterns(config.exclude_patterns.as_deref());
    let mut source_refs: Vec<(Rc<str>, usize)> = Vec::new();
    for dir_str in &config.source_dirs {
        let dir = Path::new(dir_str);
        if !dir.exists() {
            continue;
        }
        collect_all_files(
            dir,
            config.target_extensions.as_deref(),
            exclude_set.as_ref(),
            &mut |path| {
                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Warning: failed to read '{}': {e}", path.display());
                        return;
                    }
                };
                let rel_path: Rc<str> = Rc::from(to_forward_slash(path).as_ref());
                for (line_num, line) in content.lines().enumerate() {
                    if let Some(found_ulid) = extract_marker_ulid(line)
                        && found_ulid == ulid
                    {
                        source_refs.push((Rc::clone(&rel_path), line_num + 1));
                    }
                }
            },
        );
    }
    source_refs.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let nothing_found = specre_path.is_none() && source_refs.is_empty();

    if json {
        let output = TraceByUlidOutput {
            specre: specre_path,
            source_refs: source_refs
                .into_iter()
                .map(|(file, line)| SourceRefOutput {
                    file: file.to_string(),
                    line,
                })
                .collect(),
        };
        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{json_str}");
    } else {
        // Print output
        println!("Specre:");
        match &specre_path {
            Some(p) => println!("  {p}"),
            None => println!("  (not found)"),
        }

        println!();
        println!("Source references:");
        if source_refs.is_empty() {
            println!("  (none)");
        } else {
            for (file, line) in &source_refs {
                println!("  {file}:{line}");
            }
        }
    }

    // Exit with error if nothing found at all
    if nothing_found {
        return Err(SpecreError::NonZeroExit);
    }

    Ok(())
}

fn trace_by_file(file_path: &str, json: bool) -> Result<(), SpecreError> {
    let file_path = file_path.replace('\\', "/");
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(SpecreError::InvalidArgument(format!(
            "file not found: {file_path}"
        )));
    }

    let content = fs::read_to_string(path).map_err(|e| SpecreError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    // Extract all marker ULIDs from the file (preserving order)
    let mut ulids: Vec<String> = Vec::new();
    for line in content.lines() {
        if let Some(ulid) = extract_marker_ulid(line)
            && !ulids.iter().any(|u| u == ulid)
        {
            ulids.push(ulid.to_string());
        }
    }

    let config = config::load()?;
    let specre_dir = Path::new(&config.specre_dir);

    // Build ULID → specre path map
    let mut ulid_to_path: HashMap<String, String> = HashMap::new();
    if specre_dir.exists() {
        collect_md_files(specre_dir, &mut |path| {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: failed to read '{}': {e}", path.display());
                    return;
                }
            };
            if let Ok(fm) = parse_frontmatter(&content) {
                ulid_to_path.insert(fm.id, to_forward_slash(path).into_owned());
            }
        });
    }

    if json {
        let output = TraceByFileOutput {
            file: file_path.clone(),
            specres: ulids
                .iter()
                .map(|u| SpecreRefOutput {
                    id: u.clone(),
                    path: ulid_to_path.get(u).cloned(),
                })
                .collect(),
        };
        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{json_str}");
    } else {
        // Print output
        println!("File: {file_path}");
        println!();
        println!("Specres:");
        if ulids.is_empty() {
            println!("  (none)");
        } else {
            for ulid in &ulids {
                match ulid_to_path.get(ulid) {
                    Some(specre_path) => println!("  {ulid}  {specre_path}"),
                    None => println!("  {ulid}  (not found)"),
                }
            }
        }
    }

    if ulids.is_empty() {
        return Err(SpecreError::NonZeroExit);
    }

    Ok(())
}
