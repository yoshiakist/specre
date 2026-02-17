// @specre 01KHB48DYZDN8GHXPX7MSYJ1NZ
use crate::status::Status;
use std::fmt;

pub struct Frontmatter {
    pub id: String,
    pub name: String,
    pub status: Status,
    pub last_verified: Option<String>,
}

/// Describes why front-matter parsing failed, preserving diagnostic
/// information so that callers can tell the user *what* to fix.
#[derive(Debug)]
pub enum FrontmatterError {
    /// The content does not start with `---`.
    NoOpeningDelimiter,
    /// An opening `---` was found but the closing `---` is missing.
    NoClosingDelimiter,
    /// The YAML block could not be deserialized (missing field, invalid
    /// status value, syntax error, etc.).
    Yaml(serde_yaml::Error),
}

impl fmt::Display for FrontmatterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoOpeningDelimiter => write!(f, "missing opening '---' delimiter"),
            Self::NoClosingDelimiter => write!(f, "missing closing '---' delimiter"),
            Self::Yaml(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for FrontmatterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Yaml(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(serde::Deserialize)]
struct RawFrontmatter {
    id: String,
    name: String,
    status: Status,
    last_verified: Option<String>,
}

/// Parses YAML front-matter delimited by `---` from specre card content.
///
/// # Errors
///
/// Returns [`FrontmatterError`] if delimiters are missing or the YAML is invalid.
pub fn parse_frontmatter(content: &str) -> Result<Frontmatter, FrontmatterError> {
    let content = content.trim_start_matches('\u{feff}'); // BOM
    if !content.starts_with("---") {
        return Err(FrontmatterError::NoOpeningDelimiter);
    }
    let after_first = &content[3..];
    let end = after_first
        .find("\n---")
        .ok_or(FrontmatterError::NoClosingDelimiter)?;
    let block = &after_first[..end];

    let raw: RawFrontmatter = serde_yaml::from_str(block).map_err(FrontmatterError::Yaml)?;

    Ok(Frontmatter {
        id: raw.id,
        name: raw.name,
        status: raw.status,
        last_verified: raw.last_verified,
    })
}

/// Extracts a ULID from a `@specre` marker on a line.
/// Returns None if the marker appears inside a string literal
/// (preceded by a quote character `"` or `'` on the same line).
#[must_use]
pub fn extract_marker_ulid(line: &str) -> Option<&str> {
    let pos = line.find("@specre ")?;
    let before = &line[..pos];
    if before.contains('"') || before.contains('\'') {
        return None;
    }
    let after = &line[pos + 8..];
    if after.len() < 26 {
        return None;
    }
    let candidate = &after[..26];
    if candidate
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        Some(candidate)
    } else {
        None
    }
}
