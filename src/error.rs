// @specre 01KHMEB8WF7BFZASE8SQHF5PR2
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
    /// コマンドは成功したが、検出結果により非ゼロ終了が必要
    NonZeroExit,
    /// TOML シリアライズの失敗（specre.toml 生成時）
    ConfigSerialize(toml::ser::Error),
    /// tokio ランタイム生成の失敗
    TokioRuntime(std::io::Error),
    /// MCP サーバー初期化の失敗
    McpInit(String),
    /// MCP サーバー実行中のタスクエラー
    McpTask(String),
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
            Self::NonZeroExit => Ok(()),
            Self::ConfigSerialize(e) => write!(f, "Failed to serialize specre.toml: {e}"),
            Self::TokioRuntime(e) => write!(f, "Failed to create tokio runtime: {e}"),
            Self::McpInit(e) => write!(f, "Failed to initialize MCP server: {e}"),
            Self::McpTask(e) => write!(f, "MCP server error: {e}"),
        }
    }
}

impl std::error::Error for SpecreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ConfigParse(e) => Some(e),
            Self::Io { source, .. } => Some(source),
            Self::Serialize(e) => Some(e),
            Self::ConfigSerialize(e) => Some(e),
            Self::TokioRuntime(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for SpecreError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialize(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    /// Produce a `toml::ser::Error` by serializing a type that fails.
    fn force_toml_ser_error() -> toml::ser::Error {
        struct Fail;
        impl serde::Serialize for Fail {
            fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom("test error"))
            }
        }
        toml::to_string(&Fail).unwrap_err()
    }

    // --- Display formatting ---

    #[test]
    fn already_initialized_display() {
        let err = SpecreError::AlreadyInitialized;
        assert_eq!(
            err.to_string(),
            "specre.toml already exists. This project is already initialized."
        );
    }

    #[test]
    fn config_not_found_display() {
        let err = SpecreError::ConfigNotFound;
        assert_eq!(
            err.to_string(),
            "specre.toml not found. Run 'specre init' first."
        );
    }

    #[test]
    fn config_parse_display() {
        let toml_err = toml::from_str::<toml::Value>("invalid [[[").unwrap_err();
        let msg = toml_err.to_string();
        let err = SpecreError::ConfigParse(toml_err);
        assert_eq!(err.to_string(), format!("Failed to parse specre.toml: {msg}"));
    }

    #[test]
    fn io_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file gone");
        let err = SpecreError::Io {
            path: PathBuf::from("/tmp/missing.toml"),
            source: io_err,
        };
        assert_eq!(
            err.to_string(),
            "Failed to access '/tmp/missing.toml': file gone"
        );
    }

    #[test]
    fn serialize_display() {
        let serde_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let msg = serde_err.to_string();
        let err = SpecreError::Serialize(serde_err);
        assert_eq!(err.to_string(), format!("Failed to serialize: {msg}"));
    }

    #[test]
    fn invalid_argument_display() {
        let err = SpecreError::InvalidArgument("bad input".into());
        assert_eq!(err.to_string(), "bad input");
    }

    #[test]
    fn config_serialize_display() {
        let toml_err = force_toml_ser_error();
        let msg = toml_err.to_string();
        let err = SpecreError::ConfigSerialize(toml_err);
        assert_eq!(
            err.to_string(),
            format!("Failed to serialize specre.toml: {msg}")
        );
    }

    #[test]
    fn tokio_runtime_display() {
        let inner = std::io::Error::other("cannot create runtime");
        let err = SpecreError::TokioRuntime(inner);
        assert_eq!(
            err.to_string(),
            "Failed to create tokio runtime: cannot create runtime"
        );
    }

    #[test]
    fn mcp_init_display() {
        let err = SpecreError::McpInit("handshake failed".into());
        assert_eq!(
            err.to_string(),
            "Failed to initialize MCP server: handshake failed"
        );
    }

    #[test]
    fn mcp_task_display() {
        let err = SpecreError::McpTask("connection lost".into());
        assert_eq!(err.to_string(), "MCP server error: connection lost");
    }

    // --- NonZeroExit ---

    #[test]
    fn non_zero_exit_display_is_empty() {
        let err = SpecreError::NonZeroExit;
        assert_eq!(err.to_string(), "");
    }

    // --- source() chain ---

    #[test]
    fn config_parse_source_returns_some() {
        let toml_err = toml::from_str::<toml::Value>("bad").unwrap_err();
        let err = SpecreError::ConfigParse(toml_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn io_source_returns_some() {
        let io_err = std::io::Error::other("oops");
        let err = SpecreError::Io {
            path: PathBuf::from("x"),
            source: io_err,
        };
        assert!(err.source().is_some());
    }

    #[test]
    fn serialize_source_returns_some() {
        let serde_err = serde_json::from_str::<serde_json::Value>("{bad}").unwrap_err();
        let err = SpecreError::Serialize(serde_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn config_serialize_source_returns_some() {
        let err = SpecreError::ConfigSerialize(force_toml_ser_error());
        assert!(err.source().is_some());
    }

    #[test]
    fn tokio_runtime_source_returns_some() {
        let inner = std::io::Error::other("err");
        let err = SpecreError::TokioRuntime(inner);
        assert!(err.source().is_some());
    }

    #[test]
    fn mcp_init_source_returns_none() {
        let err = SpecreError::McpInit("err".into());
        assert!(err.source().is_none());
    }

    #[test]
    fn mcp_task_source_returns_none() {
        let err = SpecreError::McpTask("err".into());
        assert!(err.source().is_none());
    }

    #[test]
    fn already_initialized_source_returns_none() {
        assert!(SpecreError::AlreadyInitialized.source().is_none());
    }

    #[test]
    fn config_not_found_source_returns_none() {
        assert!(SpecreError::ConfigNotFound.source().is_none());
    }

    #[test]
    fn invalid_argument_source_returns_none() {
        assert!(SpecreError::InvalidArgument("x".into()).source().is_none());
    }

    #[test]
    fn non_zero_exit_source_returns_none() {
        assert!(SpecreError::NonZeroExit.source().is_none());
    }

    // --- From<serde_json::Error> ---

    #[test]
    fn from_serde_json_error_converts_to_serialize() {
        let serde_err = serde_json::from_str::<serde_json::Value>("!!!").unwrap_err();
        let msg = serde_err.to_string();
        let specre_err: SpecreError = serde_err.into();
        assert_eq!(
            specre_err.to_string(),
            format!("Failed to serialize: {msg}")
        );
    }
}
