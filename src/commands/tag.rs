use crate::cli::TagArgs;
use std::fs;
use std::path::Path;

fn is_valid_ulid(s: &str) -> bool {
    s.len() == 26 && s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

fn comment_syntax(ext: &str) -> (&'static str, &'static str) {
    match ext {
        // // style
        "rs" | "js" | "ts" | "jsx" | "tsx" | "java" | "c" | "cpp" | "h" | "hpp" | "cs" | "go"
        | "swift" => ("// ", ""),
        // # style
        "rb" | "py" | "sh" | "bash" | "zsh" | "yaml" | "yml" | "toml" => ("# ", ""),
        // /* */ style
        "css" | "scss" | "sass" | "less" => ("/* ", " */"),
        // <!-- --> style
        "html" | "htm" | "xml" | "svg" => ("<!-- ", " -->"),
        // -- style
        "sql" => ("-- ", ""),
        // default
        _ => ("// ", ""),
    }
}

fn to_forward_slash(s: &str) -> String {
    s.replace('\\', "/")
}

pub fn execute(args: TagArgs) -> Result<(), String> {
    if !is_valid_ulid(&args.ulid) {
        return Err(
            "invalid ULID format. Expected 26 uppercase alphanumeric characters.".to_string(),
        );
    }

    let file_path = Path::new(&args.file);

    if !file_path.exists() {
        return Err(format!("file not found: {}", to_forward_slash(&args.file)));
    }

    if file_path.is_dir() {
        return Err(format!(
            "'{}' is a directory, not a file",
            to_forward_slash(&args.file)
        ));
    }

    let content =
        fs::read_to_string(file_path).map_err(|e| format!("Failed to read '{}': {e}", args.file))?;

    // Check if marker already exists
    let marker_pattern = format!("@specre {}", args.ulid);
    if content.contains(&marker_pattern) {
        println!(
            "Marker already exists in {}",
            to_forward_slash(&args.file)
        );
        return Ok(());
    }

    // Determine comment syntax from file extension
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let (prefix, suffix) = comment_syntax(ext);

    let marker_line = format!("{prefix}@specre {}{suffix}\n", args.ulid);
    let new_content = format!("{marker_line}{content}");

    fs::write(file_path, &new_content)
        .map_err(|e| format!("Failed to write '{}': {e}", args.file))?;

    println!(
        "Tagged {} with {}",
        to_forward_slash(&args.file),
        args.ulid
    );

    Ok(())
}
