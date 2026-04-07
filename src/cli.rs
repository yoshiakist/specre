// @specre 01KHFFCX8BCDAYP8YHG0J65H0E
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "specre",
    version,
    about = "Atomic specification cards for AI-agent-friendly development"
)]
pub struct Cli {
    /// Output in JSON format (machine-readable)
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize specre in a project
    Init(InitArgs),

    /// Create a new specre card from a template
    New(NewArgs),

    /// Generate index.json and per-domain _INDEX.md
    Index,

    /// Report specre counts by status and flag stale `last_verified` dates
    Status(StatusArgs),

    /// Bidirectional traceability lookup by ULID or file path
    Trace(TraceArgs),

    /// Detect unlinked specres or dangling markers
    Orphans,

    /// Insert a @specre marker into a source file
    Tag(TagArgs),

    /// Report the percentage of source files covered by @specre tags
    Coverage(CoverageArgs),

    /// Comprehensive health check for AI agent preflight
    HealthCheck,

    /// Search specre cards by text query and structured filters
    Search(SearchArgs),

    /// Start the MCP server (stdio transport)
    Mcp,

    /// Remove all specre artifacts from the project (markers, config files)
    Destroy,

    /// Detect drift between specre cards and implementation code
    Drift(DriftArgs),

    /// Update `last_verified` to today for confirmed specres
    Verify(VerifyArgs),

    /// Transition a stable specre back to in-development
    Reopen(ReopenArgs),
}

#[derive(Debug, Args)]
pub struct SearchArgs {
    /// Free-text substring to match against card content (case-insensitive)
    pub query: Option<String>,

    /// Filter by status (draft, in-development, stable, deprecated)
    #[arg(long)]
    pub status: Option<String>,

    /// Filter by domain (top-level directory under `specre_dir`)
    #[arg(long)]
    pub domain: Option<String>,

    /// Include only specres whose `last_verified` is before this date (YYYY-MM-DD)
    #[arg(long)]
    pub verified_before: Option<String>,

    /// Include only specres whose `last_verified` is on or after this date (YYYY-MM-DD)
    #[arg(long)]
    pub verified_after: Option<String>,

    /// Use OR logic for multi-keyword queries (default is AND)
    #[arg(long)]
    pub or: bool,

    /// Return at most N results, bypassing the truncation threshold
    #[arg(long)]
    pub limit: Option<usize>,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Directory where specre cards are stored
    #[arg(long, default_value = "docs/specres")]
    pub specre_dir: String,

    /// Directories to scan for @specre markers (comma-separated)
    #[arg(long, default_value = "src", value_delimiter = ',')]
    pub source_dirs: Vec<String>,

    /// Language for specre card templates (e.g., "en", "ja")
    #[arg(long)]
    pub language: Option<String>,

    /// Target file extensions for source scanning (comma-separated, e.g., "rs,ts,js")
    #[arg(long, value_delimiter = ',')]
    pub ext: Option<Vec<String>>,
}

#[derive(Debug, Args)]
pub struct StatusArgs {
    /// Number of days after which a stable specre's `last_verified` is considered stale
    #[arg(long, default_value_t = 30)]
    pub threshold: u32,
}

#[derive(Debug, Args)]
pub struct TraceArgs {
    /// ULID (26 uppercase alphanumeric characters) or file path to look up
    pub query: String,
}

#[derive(Debug, Args)]
pub struct TagArgs {
    /// ULID to insert as a marker (26 uppercase alphanumeric characters)
    pub ulid: String,

    /// Path to the source file where the marker will be inserted
    pub file: String,
}

#[derive(Debug, Args)]
pub struct CoverageArgs {
    /// Target file extensions to filter by (comma-separated, e.g., "rs,ts")
    #[arg(long, value_delimiter = ',')]
    pub ext: Option<Vec<String>>,
}

#[derive(Debug, Args)]
pub struct NewArgs {
    /// Target directory where the specre file will be created
    pub target_dir: String,

    /// Name of the specre (used as filename and front-matter name field)
    #[arg(long, default_value = "untitled")]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// ULIDs of specres to verify
    #[arg(conflicts_with_all = ["domain", "file"])]
    pub ulids: Vec<String>,

    /// Verify all specres in a domain
    #[arg(long, conflicts_with = "file")]
    pub domain: Option<String>,

    /// Verify all specres linked to a source file
    #[arg(long)]
    pub file: Option<String>,
}

#[derive(Debug, Args)]
pub struct ReopenArgs {
    /// ULID of the specre to reopen
    pub ulid: String,
}

#[derive(Debug, Args)]
pub struct DriftArgs {
    /// Target: ULID, specre path, or omit for project-wide check
    pub target: Option<String>,

    /// Buffer period before reporting drift (e.g., 0d, 7d, 30d)
    #[arg(long)]
    pub grace: Option<String>,

    /// Filter by status (default: stable)
    #[arg(long)]
    pub status: Option<String>,

    /// Filter by domain name
    #[arg(long)]
    pub domain: Option<String>,
}
