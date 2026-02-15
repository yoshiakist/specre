// @specre 01KHB48DYZDN8GHXPX7MSYJ1NZ
// @specre 01KHG0A2V4YXE918WMJCY7WFE8
use crate::cli::TraceArgs;
use crate::commands::index::{
    collect_all_files, collect_md_files, extract_marker_ulid, parse_frontmatter, to_forward_slash,
};
use crate::config;
use crate::ulid;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

pub fn execute(args: TraceArgs, json: bool) -> Result<(), String> {
    if ulid::is_valid(&args.query) {
        trace_by_ulid(&args.query, json)
    } else {
        trace_by_file(&args.query, json)
    }
}

fn trace_by_ulid(ulid: &str, json: bool) -> Result<(), String> {
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
                Err(_) => return,
            };
            if let Some(fm) = parse_frontmatter(&content)
                && fm.id == ulid
            {
                specre_path = Some(to_forward_slash(path));
            }
        });
    }

    // Find source references
    let mut source_refs: Vec<(String, usize)> = Vec::new();
    for dir_str in &config.source_dirs {
        let dir = Path::new(dir_str);
        if !dir.exists() {
            continue;
        }
        collect_all_files(dir, config.target_extensions.as_deref(), &mut |path| {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => return,
            };
            let rel_path = to_forward_slash(path);
            for (line_num, line) in content.lines().enumerate() {
                if let Some(found_ulid) = extract_marker_ulid(line)
                    && found_ulid == ulid
                {
                    source_refs.push((rel_path.clone(), line_num + 1));
                }
            }
        });
    }
    source_refs.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    if json {
        let output = TraceByUlidOutput {
            specre: specre_path.clone(),
            source_refs: source_refs
                .iter()
                .map(|(file, line)| SourceRefOutput {
                    file: file.clone(),
                    line: *line,
                })
                .collect(),
        };
        let json_str = serde_json::to_string_pretty(&output)
            .map_err(|e| format!("Failed to serialize: {e}"))?;
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
    if specre_path.is_none() && source_refs.is_empty() {
        return Err(String::new());
    }

    Ok(())
}

fn trace_by_file(file_path: &str, json: bool) -> Result<(), String> {
    let file_path = file_path.replace('\\', "/");
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(format!("file not found: {file_path}"));
    }

    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read '{file_path}': {e}"))?;

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
                Err(_) => return,
            };
            if let Some(fm) = parse_frontmatter(&content) {
                ulid_to_path.insert(fm.id, to_forward_slash(path));
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
        let json_str = serde_json::to_string_pretty(&output)
            .map_err(|e| format!("Failed to serialize: {e}"))?;
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
        return Err(String::new());
    }

    Ok(())
}
