// @specre 01JMBJK7QRVX3N4P5G6H8W9Y0Z
// @specre 01KHDF9WHR5HFM4RQCF6HS3KCC
use crate::cli::NewArgs;
use crate::config;
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

    let language = config::load_language();
    let id = ulid::generate();
    let content = template::render(&id, &args.name, &language);

    fs::write(&file_path, &content)
        .map_err(|e| format!("Failed to write '{}': {e}", file_path.display()))?;

    println!("{}", file_path.display());

    Ok(())
}
