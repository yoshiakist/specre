// @specre 01KHB48EYB9686YYQMYFYQ5R1Z
// @specre 01KHB48EES4FR5TFV6ZP2W3MGT
// @specre 01KHFTCYJN8YJMW2RNHJTAQV49
// @specre 01KHJ98T83DPJGMEFH9HAXXAZ1
use crate::parser::parse_frontmatter;
use crate::scanner::collect_md_files;
use crate::status::Status;
use serde::Serialize;
use std::borrow::Cow;
use std::fs;
use std::path::Path;

/// Unified representation of a specre card's metadata.
///
/// This is the single source of truth for specre card data,
/// replacing per-command struct definitions (`SpecreEntry`,
/// `CardData`, `ScannedCard`) with one shared type.
#[derive(Debug, Serialize)]
pub struct SpecreCard {
    pub id: String,
    pub name: String,
    pub status: Status,
    pub domain: String,
    pub path: String,
    pub last_verified: Option<String>,
}

/// Converts a `Path` to a forward-slash-separated string.
///
/// On Windows, backslashes are replaced with forward slashes.
/// On Unix, the path string is returned as-is.
#[must_use]
pub fn to_forward_slash(path: &Path) -> Cow<'_, str> {
    let s = path.to_string_lossy();
    if s.contains('\\') {
        Cow::Owned(s.replace('\\', "/"))
    } else {
        s
    }
}

/// Extracts the domain (first path component after `specre_dir`)
/// from a forward-slash-separated relative path.
///
/// ```text
/// extract_domain("docs/specres/cli/foo.md", "docs/specres") => "cli"
/// ```
#[must_use]
pub fn extract_domain<'a>(rel_path: &'a str, specre_dir: &str) -> &'a str {
    let prefix = if specre_dir.ends_with('/') {
        specre_dir.to_string()
    } else {
        format!("{specre_dir}/")
    };
    let after = rel_path.strip_prefix(&prefix).unwrap_or(rel_path);
    after.split('/').next().unwrap_or("unknown")
}

/// Scans the specre directory and returns all valid cards with metadata.
///
/// Cards are sorted by ID. Files with malformed front-matter are
/// skipped with a warning printed to stderr.
#[must_use]
pub fn scan_specre_cards(specre_dir: &Path, specre_dir_str: &str) -> Vec<SpecreCard> {
    let mut cards = Vec::new();
    if !specre_dir.exists() {
        return cards;
    }
    collect_md_files(specre_dir, &mut |path| {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: failed to read '{}': {e}", path.display());
                return;
            }
        };
        match parse_frontmatter(&content) {
            Ok(fm) => {
                let rel_path = to_forward_slash(path);
                let domain = extract_domain(&rel_path, specre_dir_str).to_owned();
                cards.push(SpecreCard {
                    id: fm.id,
                    name: fm.name,
                    status: fm.status,
                    domain,
                    path: rel_path.into_owned(),
                    last_verified: fm.last_verified,
                });
            }
            Err(e) => {
                eprintln!("Warning: skipping '{}': {e}", path.display());
            }
        }
    });
    cards.sort_by(|a, b| a.id.cmp(&b.id));
    cards
}
