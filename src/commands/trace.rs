// @specre 01KHB48DYZDN8GHXPX7MSYJ1NZ
use crate::cli::TraceArgs;
use crate::commands::index::{
    collect_all_files, collect_md_files, extract_marker_ulid, parse_frontmatter, to_forward_slash,
};
use crate::config;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn is_valid_ulid(s: &str) -> bool {
    s.len() == 26 && s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

pub fn execute(args: TraceArgs) -> Result<(), String> {
    if is_valid_ulid(&args.query) {
        trace_by_ulid(&args.query)
    } else {
        trace_by_file(&args.query)
    }
}

fn trace_by_ulid(ulid: &str) -> Result<(), String> {
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
            if let Some(fm) = parse_frontmatter(&content) {
                if fm.id == ulid {
                    specre_path = Some(to_forward_slash(path));
                }
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
        collect_all_files(dir, &mut |path| {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => return,
            };
            let rel_path = to_forward_slash(path);
            for (line_num, line) in content.lines().enumerate() {
                if let Some(found_ulid) = extract_marker_ulid(line) {
                    if found_ulid == ulid {
                        source_refs.push((rel_path.clone(), line_num + 1));
                    }
                }
            }
        });
    }
    source_refs.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

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

    // Exit with error if nothing found at all
    if specre_path.is_none() && source_refs.is_empty() {
        return Err(String::new());
    }

    Ok(())
}

fn trace_by_file(file_path: &str) -> Result<(), String> {
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
        if let Some(ulid) = extract_marker_ulid(line) {
            if !ulids.iter().any(|u| u == ulid) {
                ulids.push(ulid.to_string());
            }
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

    if ulids.is_empty() {
        return Err(String::new());
    }

    Ok(())
}
