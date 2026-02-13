// @specre 01KHB48EYB9686YYQMYFYQ5R1Z
// @specre 01KHB48EES4FR5TFV6ZP2W3MGT
// @specre 01KHB48DYZDN8GHXPX7MSYJ1NZ
// @specre 01KHAN6JE712ZAKXPP97854PKJ
// @specre 01KHAKAYN5WPTDVR99D5Q5TMJE
// @specre 01KHAGG8NQQ7RSNYZ6SWBCYH3N
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
