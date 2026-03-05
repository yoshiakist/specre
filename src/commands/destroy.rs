// @specre 01KJYMAV7G01B743W72WAG9RGN
use crate::config;
use crate::error::SpecreError;
use crate::parser::extract_marker_ulid;
use crate::scanner::{collect_all_files, compile_exclude_patterns};
use std::fs;
use std::io::{self, BufRead, Write as IoWrite};
use std::path::Path;

const CONFIG_FILE: &str = "specre.toml";
const GLOSSARY_FILE: &str = "glossary.toml";

/// Returns `true` if `line` is a specre marker line inserted by `specre tag`.
///
/// All three conditions must hold:
/// 1. No leading whitespace (starts at column 0).
/// 2. [`extract_marker_ulid`] returns `Some(_)` — valid 26-char ULID, not
///    preceded by a quote character.
/// 3. The prefix text before `@specre `, when trimmed, contains no embedded
///    space — accepting comment prefixes like `//`, `#`, `/*`, `<!--` while
///    rejecting prose like `# See @specre …`.
fn is_marker_line(line: &str) -> bool {
    // Condition 1: no leading whitespace
    if line.starts_with(|c: char| c.is_whitespace()) {
        return false;
    }
    // Condition 2: valid ULID marker (not in a string literal)
    if extract_marker_ulid(line).is_none() {
        return false;
    }
    // Condition 3: prefix before "@specre " has no embedded space
    line.find("@specre ")
        .is_some_and(|pos| !line[..pos].trim().contains(' '))
}

/// Removes every marker line (as defined by [`is_marker_line`]) from `path`.
/// Returns the number of lines removed. The file is rewritten only if at
/// least one line was removed.
///
/// # Errors
///
/// Returns [`SpecreError::Io`] on write failure. Read failures are signalled
/// by the caller via a stderr warning instead of being treated as fatal.
fn remove_markers_from_file(path: &Path) -> Result<usize, SpecreError> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == io::ErrorKind::InvalidData => return Ok(0),
        Err(e) => {
            eprintln!("Warning: failed to read '{}': {e}", path.display());
            return Ok(0);
        }
    };

    let original_line_count = content.lines().count();
    let filtered: Vec<&str> = content
        .lines()
        .filter(|line| !is_marker_line(line))
        .collect();
    let removed = original_line_count - filtered.len();

    if removed == 0 {
        return Ok(0);
    }

    // Preserve trailing newline if the original had one
    let new_content = if content.ends_with('\n') {
        format!("{}\n", filtered.join("\n"))
    } else {
        filtered.join("\n")
    };

    fs::write(path, new_content).map_err(|e| SpecreError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(removed)
}

/// Reads a `[y/N]` answer from stdin. Returns `true` only for `y` or `Y`.
/// Empty input (Enter) and everything else returns `false`.
fn prompt_yes_no(question: &str) -> bool {
    print!("{question} [y/N]: ");
    io::stdout().flush().ok();

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).ok();
    matches!(line.trim(), "y" | "Y")
}

/// # Errors
///
/// Returns [`SpecreError`] on config load failure or I/O errors during
/// deletion/rewriting.
pub fn execute() -> Result<(), SpecreError> {
    let config = config::load()?;

    println!(
        "This will remove all @specre markers from your source files and delete {CONFIG_FILE}."
    );

    let delete_specre_dir = prompt_yes_no(&format!(
        "Also delete the specre cards directory '{}'?",
        config.specre_dir
    ));

    // Strip @specre markers from all source files
    let exclude_set = compile_exclude_patterns(config.exclude_patterns.as_deref());
    let mut modified_files: Vec<String> = Vec::new();

    for dir_str in &config.source_dirs {
        let dir = Path::new(dir_str);
        if !dir.exists() {
            continue;
        }
        collect_all_files(
            dir,
            config.target_extensions.as_deref(),
            exclude_set.as_ref(),
            &mut |path| match remove_markers_from_file(path) {
                Ok(0) => {}
                Ok(_) => modified_files.push(path.display().to_string()),
                Err(e) => eprintln!("Warning: {e}"),
            },
        );
    }

    for file in &modified_files {
        println!("Cleaned  {file}");
    }

    // Delete specre.toml
    let config_path = Path::new(CONFIG_FILE);
    fs::remove_file(config_path).map_err(|e| SpecreError::Io {
        path: config_path.to_path_buf(),
        source: e,
    })?;
    println!("Deleted  {CONFIG_FILE}");

    // Delete glossary.toml if it exists
    let glossary_path = Path::new(GLOSSARY_FILE);
    if glossary_path.exists() {
        fs::remove_file(glossary_path).map_err(|e| SpecreError::Io {
            path: glossary_path.to_path_buf(),
            source: e,
        })?;
        println!("Deleted  {GLOSSARY_FILE}");
    }

    // Optionally delete the specre cards directory
    if delete_specre_dir {
        let specre_dir = Path::new(&config.specre_dir);
        if specre_dir.exists() {
            fs::remove_dir_all(specre_dir).map_err(|e| SpecreError::Io {
                path: specre_dir.to_path_buf(),
                source: e,
            })?;
            println!("Deleted  {}/", config.specre_dir);
        }
    }

    println!("Done. To remove the specre binary, run: cargo uninstall specre");
    Ok(())
}
