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
const GLOSSARY_FILE: &str = "glossary.toml";

const GLOSSARY_SAMPLE: &str = r#"# specre glossary — Project vocabulary for search suggestions
#
# Add terms that describe your project's domains, operations, and concepts.
# `specre search` uses this glossary to suggest query refinements when
# results are empty or too broad.
#
# Customize this list with your project's domain-specific vocabulary:
#   - Domain nouns: "user", "order", "payment", "notification"
#   - Operations: "create", "update", "delete", "approve", "reject"
#   - Technical concepts: "authentication", "authorization", "validation"

terms = [
  "user",
  "system",
  "create",
  "update",
  "delete",
  "find",
  "list",
]
"#;

/// # Errors
///
/// Returns [`SpecreError`] if already initialized, or on I/O / serialization failure.
pub fn execute(args: InitArgs, json: bool) -> Result<(), SpecreError> {
    let config_path = Path::new(CONFIG_FILE);

    if config_path.exists() {
        return Err(SpecreError::AlreadyInitialized);
    }

    let InitArgs {
        specre_dir,
        source_dirs,
        language,
        ext,
    } = args;

    let specre_dir_path = Path::new(&specre_dir);
    let dir_already_existed = specre_dir_path.exists();

    if !dir_already_existed {
        fs::create_dir_all(specre_dir_path).map_err(|e| SpecreError::Io {
            path: specre_dir_path.to_path_buf(),
            source: e,
        })?;
        if !json {
            println!("Created {specre_dir}/");
        }
    } else if !json {
        println!("Exists  {specre_dir}/");
    }

    let config = SpecreConfig {
        specre_dir: specre_dir.clone(),
        source_dirs,
        language,
        target_extensions: ext,
    };
    let config_content = toml::to_string(&config).map_err(SpecreError::ConfigSerialize)?;

    fs::write(config_path, &config_content).map_err(|e| SpecreError::Io {
        path: config_path.to_path_buf(),
        source: e,
    })?;

    if !json {
        println!("Created {CONFIG_FILE}");
    }

    // Write glossary.toml (only if it doesn't already exist)
    let glossary_path = Path::new(GLOSSARY_FILE);
    let glossary_already_existed = glossary_path.exists();

    if !glossary_already_existed {
        fs::write(glossary_path, GLOSSARY_SAMPLE).map_err(|e| SpecreError::Io {
            path: glossary_path.to_path_buf(),
            source: e,
        })?;
    }
    if !json {
        if glossary_already_existed {
            println!("Exists  {GLOSSARY_FILE}");
        } else {
            println!("Created {GLOSSARY_FILE}");
        }
    }

    if json {
        let output = InitOutput {
            specre_dir,
            config_file: CONFIG_FILE.to_string(),
        };
        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{json_str}");
    }

    Ok(())
}
