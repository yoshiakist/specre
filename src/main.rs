// @specre 01KHFFCX8BCDAYP8YHG0J65H0E
mod cli;
mod commands;
mod config;
pub mod error;
mod template;
mod ulid;

use clap::Parser;
use cli::{Cli, Commands};
use std::process;

fn main() {
    let cli = Cli::parse();
    let json = cli.json;

    let result = match cli.command {
        Commands::Init(args) => commands::init::execute(args, json).map_err(|e| e.to_string()),
        Commands::New(args) => commands::new::execute(args, json).map_err(|e| e.to_string()),
        Commands::Index => commands::index::execute(json),
        Commands::Status(args) => commands::status::execute(args, json),
        Commands::Trace(args) => commands::trace::execute(args, json),
        Commands::Orphans => commands::orphans::execute(json),
        Commands::Tag(args) => commands::tag::execute(args, json),
        Commands::Coverage(args) => commands::coverage::execute(args, json),
        Commands::HealthCheck => commands::health_check::execute(),
        Commands::Search(args) => commands::search::execute(args),
        Commands::Mcp => commands::mcp::execute(),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
