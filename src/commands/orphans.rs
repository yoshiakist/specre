// @specre 01KHB48EES4FR5TFV6ZP2W3MGT
// @specre 01KHG0A2V4YXE918WMJCY7WFE8
use crate::card;
use crate::scanner::{scan_source_markers, SourceScanResult};
use crate::config;
use crate::error::SpecreError;
use crate::status::Status;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;

#[derive(Serialize)]
pub struct DanglingMarkerDetail {
    pub file: String,
    pub line: usize,
    #[serde(rename = "id")]
    pub ulid: String,
}

#[derive(Serialize)]
pub struct OrphanResult {
    pub orphan_specres: Vec<String>,
    pub dangling_markers: Vec<DanglingMarkerDetail>,
}

impl OrphanResult {
    #[must_use]
    pub const fn orphan_count(&self) -> usize {
        self.orphan_specres.len()
    }

    #[must_use]
    pub const fn dangling_count(&self) -> usize {
        self.dangling_markers.len()
    }
}

/// Derives an [`OrphanResult`] from a pre-computed [`SourceScanResult`].
#[must_use]
pub fn orphans_from_scan(specre_dir: &str, scan: &SourceScanResult) -> OrphanResult {
    let specre_dir_path = Path::new(specre_dir);
    let cards = card::scan_specre_cards(specre_dir_path, specre_dir);

    // Find dangling markers (markers with no matching specre)
    // Scope specre_ids borrow so cards can be consumed below
    let mut dangling: Vec<DanglingMarkerDetail> = {
        let specre_ids: HashSet<&str> = cards.iter().map(|c| c.id.as_str()).collect();
        scan.all_markers
            .iter()
            .filter(|m| !specre_ids.contains(m.ulid.as_str()))
            .map(|m| DanglingMarkerDetail {
                file: m.file.clone(),
                line: m.line,
                ulid: m.ulid.clone(),
            })
            .collect()
    };
    dangling.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));

    // Find orphan specres (non-deprecated specres with no source markers)
    // into_iter() moves c.path instead of cloning
    let mut orphan_paths: Vec<String> = cards
        .into_iter()
        .filter(|c| c.status != Status::Deprecated && !scan.marker_ulids.contains(c.id.as_str()))
        .map(|c| c.path)
        .collect();
    orphan_paths.sort();

    OrphanResult {
        orphan_specres: orphan_paths,
        dangling_markers: dangling,
    }
}

#[must_use]
pub fn compute_orphans(
    specre_dir: &str,
    source_dirs: &[String],
    target_extensions: Option<&[String]>,
) -> OrphanResult {
    let scan = scan_source_markers(source_dirs, target_extensions);
    orphans_from_scan(specre_dir, &scan)
}

/// # Errors
///
/// Returns [`SpecreError`] on config or serialization failure, or
/// [`SpecreError::NonZeroExit`] when orphans or dangling markers are detected.
pub fn execute(json: bool) -> Result<(), SpecreError> {
    let config = config::load()?;
    let result = compute_orphans(
        &config.specre_dir,
        &config.source_dirs,
        config.target_extensions.as_deref(),
    );

    if json {
        let json_str = serde_json::to_string_pretty(&result)?;
        println!("{json_str}");

        if result.orphan_specres.is_empty() && result.dangling_markers.is_empty() {
            return Ok(());
        }
        return Err(SpecreError::NonZeroExit);
    }

    if result.orphan_specres.is_empty() && result.dangling_markers.is_empty() {
        println!("No orphans or dangling markers found.");
        return Ok(());
    }

    if !result.orphan_specres.is_empty() {
        println!("Orphan specres (no source markers):");
        for path in &result.orphan_specres {
            println!("  {path}");
        }
    }

    if !result.orphan_specres.is_empty() && !result.dangling_markers.is_empty() {
        println!();
    }

    if !result.dangling_markers.is_empty() {
        println!("Dangling markers (no matching specre):");
        for d in &result.dangling_markers {
            println!("  {}:{}  {}", d.file, d.line, d.ulid);
        }
    }

    Err(SpecreError::NonZeroExit)
}
