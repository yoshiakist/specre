// @specre 01KHAGG8NQQ7RSNYZ6SWBCYH3N
// @specre 01KHFD5R1G3C5R34XMQXQTTMM9
// @specre 01KHG0A2V4YXE918WMJCY7WFE8
use crate::cli::InitArgs;
use crate::error::SpecreError;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct SpecreConfig {
    specre_dir: String,
    source_dirs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_extensions: Option<Vec<String>>,
}

#[derive(Serialize)]
struct InitOutput {
    specre_dir: String,
    config_file: String,
}

const CONFIG_FILE: &str = "specre.toml";

pub fn execute(args: InitArgs, json: bool) -> Result<(), SpecreError> {
    let config_path = Path::new(CONFIG_FILE);

    if config_path.exists() {
        return Err(SpecreError::AlreadyInitialized);
    }

    let specre_dir = Path::new(&args.specre_dir);
    let dir_already_existed = specre_dir.exists();

    if !dir_already_existed {
        fs::create_dir_all(specre_dir).map_err(|e| SpecreError::Io {
            path: specre_dir.to_path_buf(),
            source: e,
        })?;
        if !json {
            println!("Created {}/", args.specre_dir);
        }
    } else if !json {
        println!("Exists  {}/", args.specre_dir);
    }

    let config = SpecreConfig {
        specre_dir: args.specre_dir.clone(),
        source_dirs: args.source_dirs.clone(),
        language: args.language.clone(),
        target_extensions: args.ext.clone(),
    };
    let config_content =
        toml::to_string(&config).map_err(|e| SpecreError::Runtime(Box::new(e)))?;

    fs::write(config_path, &config_content).map_err(|e| SpecreError::Io {
        path: config_path.to_path_buf(),
        source: e,
    })?;

    if json {
        let output = InitOutput {
            specre_dir: args.specre_dir,
            config_file: CONFIG_FILE.to_string(),
        };
        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{json_str}");
    } else {
        println!("Created {CONFIG_FILE}");
    }

    Ok(())
}
