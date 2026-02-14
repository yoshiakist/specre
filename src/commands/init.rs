// @specre 01KHAGG8NQQ7RSNYZ6SWBCYH3N
use crate::cli::InitArgs;
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "specre.toml";

pub fn execute(args: InitArgs) -> Result<(), String> {
    let config_path = Path::new(CONFIG_FILE);

    if config_path.exists() {
        return Err(format!(
            "{CONFIG_FILE} already exists. This project is already initialized."
        ));
    }

    let specre_dir = Path::new(&args.specre_dir);
    let dir_already_existed = specre_dir.exists();

    if !dir_already_existed {
        fs::create_dir_all(specre_dir)
            .map_err(|e| format!("Failed to create directory '{}': {e}", specre_dir.display()))?;
        println!("Created {}/", args.specre_dir);
    } else {
        println!("Exists  {}/", args.specre_dir);
    }

    let source_dirs_toml: Vec<String> = args
        .source_dirs
        .iter()
        .map(|s| format!("\"{s}\""))
        .collect();
    let language_line = match &args.language {
        Some(lang) => format!("language = \"{lang}\"\n"),
        None => String::new(),
    };
    let config_content = format!(
        "specre_dir = \"{}\"\nsource_dirs = [{}]\n{language_line}",
        args.specre_dir,
        source_dirs_toml.join(", ")
    );

    fs::write(config_path, &config_content)
        .map_err(|e| format!("Failed to write {CONFIG_FILE}: {e}"))?;

    println!("Created {CONFIG_FILE}");

    Ok(())
}
