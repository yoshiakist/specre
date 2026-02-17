// @specre 01KHB48DYZDN8GHXPX7MSYJ1NZ
use crate::status::Status;

pub struct Frontmatter {
    pub id: String,
    pub name: String,
    pub status: Status,
    pub last_verified: Option<String>,
}

pub fn parse_frontmatter(content: &str) -> Option<Frontmatter> {
    let content = content.trim_start_matches('\u{feff}'); // BOM
    if !content.starts_with("---") {
        return None;
    }
    let after_first = &content[3..];
    let end = after_first.find("\n---")?;
    let block = &after_first[..end];

    #[derive(serde::Deserialize)]
    struct RawFrontmatter {
        id: String,
        name: String,
        status: Status,
        last_verified: Option<String>,
    }

    let raw: RawFrontmatter = serde_yaml::from_str(block).ok()?;

    Some(Frontmatter {
        id: raw.id,
        name: raw.name,
        status: raw.status,
        last_verified: raw.last_verified,
    })
}

/// Extracts a ULID from a `@specre` marker on a line.
/// Returns None if the marker appears inside a string literal
/// (preceded by a quote character `"` or `'` on the same line).
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
