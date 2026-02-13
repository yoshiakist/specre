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
    /// Create a new specre card from a template
    New(NewArgs),
}

#[derive(Args)]
pub struct NewArgs {
    /// Target directory where the specre file will be created
    pub target_dir: String,

    /// Name of the specre (used as filename and front-matter name field)
    #[arg(long, default_value = "untitled")]
    pub name: String,
}
