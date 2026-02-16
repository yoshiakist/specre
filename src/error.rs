use std::path::PathBuf;

#[derive(Debug)]
pub enum SpecreError {
    /// specre.toml already exists (init の重複実行)
    AlreadyInitialized,
    /// specre.toml が見つからない
    ConfigNotFound,
    /// specre.toml のパースに失敗
    ConfigParse(toml::de::Error),
    /// ファイルシステム操作の失敗（パス付き）
    Io { path: PathBuf, source: std::io::Error },
    /// JSON シリアライズの失敗
    Serialize(serde_json::Error),
    /// バリデーションエラー（引数の不正など）
    InvalidArgument(String),
}

impl std::fmt::Display for SpecreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyInitialized => {
                write!(f, "specre.toml already exists. This project is already initialized.")
            }
            Self::ConfigNotFound => {
                write!(f, "specre.toml not found. Run 'specre init' first.")
            }
            Self::ConfigParse(e) => write!(f, "Failed to parse specre.toml: {e}"),
            Self::Io { path, source } => {
                write!(f, "Failed to access '{}': {source}", path.display())
            }
            Self::Serialize(e) => write!(f, "Failed to serialize: {e}"),
            Self::InvalidArgument(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SpecreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ConfigParse(e) => Some(e),
            Self::Io { source, .. } => Some(source),
            Self::Serialize(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for SpecreError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialize(e)
    }
}
