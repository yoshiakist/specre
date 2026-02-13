use serde::Deserialize;
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "specre.toml";

#[derive(Deserialize)]
pub struct Config {
    pub specre_dir: String,
    pub source_dirs: Vec<String>,
}

pub fn load() -> Result<Config, String> {
    let path = Path::new(CONFIG_FILE);
    if !path.exists() {
        return Err(format!(
            "{CONFIG_FILE} not found. Run 'specre init' first."
        ));
    }
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read {CONFIG_FILE}: {e}"))?;
    toml::from_str(&content).map_err(|e| format!("Failed to parse {CONFIG_FILE}: {e}"))
}
