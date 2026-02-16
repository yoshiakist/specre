// @specre 01KHB48EYB9686YYQMYFYQ5R1Z
// @specre 01KHB48EES4FR5TFV6ZP2W3MGT
// @specre 01KHB48DYZDN8GHXPX7MSYJ1NZ
// @specre 01KHAN6JE712ZAKXPP97854PKJ
// @specre 01KHAKAYN5WPTDVR99D5Q5TMJE
// @specre 01KHAGG8NQQ7RSNYZ6SWBCYH3N
// @specre 01KHDF9WHR5HFM4RQCF6HS3KCC
// @specre 01KHFD5R1G3C5R34XMQXQTTMM9
use crate::error::SpecreError;
use serde::Deserialize;
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "specre.toml";

#[derive(Deserialize)]
pub struct Config {
    pub specre_dir: String,
    pub source_dirs: Vec<String>,
    pub language: Option<String>,
    pub target_extensions: Option<Vec<String>>,
    pub health_check: Option<HealthCheckConfig>,
    pub search: Option<SearchConfig>,
}

#[derive(Deserialize)]
pub struct HealthCheckConfig {
    pub coverage: Option<f64>,
    pub orphans: Option<usize>,
    pub index_age_hours: Option<f64>,
}

#[derive(Deserialize)]
pub struct SearchConfig {
    pub max_results: Option<usize>,
}

pub fn load() -> Result<Config, SpecreError> {
    let path = Path::new(CONFIG_FILE);
    if !path.exists() {
        return Err(SpecreError::ConfigNotFound);
    }
    let content = fs::read_to_string(path).map_err(|e| SpecreError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    toml::from_str(&content).map_err(SpecreError::ConfigParse)
}

pub fn load_language() -> String {
    load()
        .ok()
        .and_then(|c| c.language)
        .unwrap_or_else(|| "en".to_string())
}
