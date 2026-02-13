use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "specre",
    version,
    about = "Atomic specification cards for AI-agent-friendly development"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize specre in a project
    Init(InitArgs),

    /// Create a new specre card from a template
    New(NewArgs),

    /// Generate index.json and per-domain INDEX.md
    Index,

    /// Report specre counts by status and flag stale last_verified dates
    Status(StatusArgs),

    /// Bidirectional traceability lookup by ULID
    Trace(TraceArgs),

    /// Detect unlinked specres or dangling markers
    Orphans,
}

#[derive(Args)]
pub struct InitArgs {
    /// Directory where specre cards are stored
    #[arg(long, default_value = "docs/specres")]
    pub specre_dir: String,

    /// Directories to scan for @specre markers (comma-separated)
    #[arg(long, default_value = "src", value_delimiter = ',')]
    pub source_dirs: Vec<String>,
}

#[derive(Args)]
pub struct StatusArgs {
    /// Number of days after which a stable specre's last_verified is considered stale
    #[arg(long, default_value_t = 30)]
    pub threshold: u32,
}

#[derive(Args)]
pub struct TraceArgs {
    /// ULID to look up (26 uppercase alphanumeric characters)
    pub ulid: String,
}

#[derive(Args)]
pub struct NewArgs {
    /// Target directory where the specre file will be created
    pub target_dir: String,

    /// Name of the specre (used as filename and front-matter name field)
    #[arg(long, default_value = "untitled")]
    pub name: String,
}
