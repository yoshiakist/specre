// @specre 01KHAGG8NQQ7RSNYZ6SWBCYH3N
// @specre 01KHFD5R1G3C5R34XMQXQTTMM9
// @specre 01KHG0A2V4YXE918WMJCY7WFE8
use crate::cli::InitArgs;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct InitOutput {
    specre_dir: String,
    config_file: String,
}

const CONFIG_FILE: &str = "specre.toml";

pub fn execute(args: InitArgs, json: bool) -> Result<(), String> {
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
        if !json {
            println!("Created {}/", args.specre_dir);
        }
    } else if !json {
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
    let ext_line = match &args.ext {
        Some(exts) => {
            let ext_toml: Vec<String> = exts.iter().map(|s| format!("\"{s}\"")).collect();
            format!("target_extensions = [{}]\n", ext_toml.join(", "))
        }
        None => String::new(),
    };
    let config_content = format!(
        "specre_dir = \"{}\"\nsource_dirs = [{}]\n{language_line}{ext_line}",
        args.specre_dir,
        source_dirs_toml.join(", ")
    );

    fs::write(config_path, &config_content)
        .map_err(|e| format!("Failed to write {CONFIG_FILE}: {e}"))?;

    if json {
        let output = InitOutput {
            specre_dir: args.specre_dir,
            config_file: CONFIG_FILE.to_string(),
        };
        let json_str = serde_json::to_string_pretty(&output)
            .map_err(|e| format!("Failed to serialize: {e}"))?;
        println!("{json_str}");
    } else {
        println!("Created {CONFIG_FILE}");
    }

    Ok(())
}
