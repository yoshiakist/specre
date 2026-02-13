use crate::cli::NewArgs;
use crate::template;
use crate::ulid;
use std::fs;
use std::path::Path;

pub fn execute(args: NewArgs) -> Result<(), String> {
    let target = Path::new(&args.target_dir);

    if target.is_file() {
        return Err(format!("'{}' is a file, not a directory", target.display()));
    }

    let file_name = format!("{}.md", args.name);
    let file_path = target.join(&file_name);

    if file_path.exists() {
        return Err(format!("'{}' already exists", file_path.display()));
    }

    if !target.exists() {
        fs::create_dir_all(target)
            .map_err(|e| format!("Failed to create directory '{}': {e}", target.display()))?;
    }

    let id = ulid::generate();
    let content = template::render(&id, &args.name);

    fs::write(&file_path, &content)
        .map_err(|e| format!("Failed to write '{}': {e}", file_path.display()))?;

    println!("{}", file_path.display());

    Ok(())
}
