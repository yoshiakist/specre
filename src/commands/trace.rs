use crate::cli::TraceArgs;
use crate::commands::index::{
    collect_all_files, collect_md_files, parse_frontmatter, to_forward_slash,
};
use crate::config;
use std::fs;
use std::path::Path;

fn is_valid_ulid(s: &str) -> bool {
    s.len() == 26 && s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

pub fn execute(args: TraceArgs) -> Result<(), String> {
    if !is_valid_ulid(&args.ulid) {
        return Err(
            "invalid ULID format. Expected 26 uppercase alphanumeric characters.".to_string(),
        );
    }

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
                if fm.id == args.ulid {
                    specre_path = Some(to_forward_slash(path));
                }
            }
        });
    }

    // Find source references
    let mut source_refs: Vec<(String, usize)> = Vec::new();
    let marker = format!("@specre {}", args.ulid);
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
                if line.contains(&marker) {
                    source_refs.push((rel_path.clone(), line_num + 1));
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
