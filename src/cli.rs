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
pub struct NewArgs {
    /// Target directory where the specre file will be created
    pub target_dir: String,

    /// Name of the specre (used as filename and front-matter name field)
    #[arg(long, default_value = "untitled")]
    pub name: String,
}
