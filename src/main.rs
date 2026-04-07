// @specre 01KHFFCX8BCDAYP8YHG0J65H0E
pub mod card;
mod cli;
mod commands;
mod config;
pub mod error;
pub mod parser;
pub mod scanner;
mod status;
mod template;
mod ulid;

use clap::Parser;
use cli::{Cli, Commands};
use error::SpecreError;
use std::process;

fn main() {
    let cli = Cli::parse();
    let json = cli.json;

    let result = match cli.command {
        Commands::Init(args) => commands::init::execute(args, json),
        Commands::New(ref args) => commands::new::execute(args, json),
        Commands::Index => commands::index::execute(json),
        Commands::Status(ref args) => commands::status::execute(args, json),
        Commands::Trace(ref args) => commands::trace::execute(args, json),
        Commands::Orphans => commands::orphans::execute(json),
        Commands::Tag(args) => commands::tag::execute(args, json),
        Commands::Coverage(args) => commands::coverage::execute(args, json),
        Commands::HealthCheck => commands::health_check::execute(),
        Commands::Search(ref args) => commands::search::execute(args),
        Commands::Mcp => commands::mcp::execute(),
        Commands::Destroy => commands::destroy::execute(),
        Commands::Drift(ref args) => commands::drift::execute(args, json),
        Commands::Verify(ref args) => commands::verify::execute(args, json),
        Commands::Reopen(ref args) => commands::reopen::execute(args, json),
    };

    match result {
        Ok(()) => {}
        Err(SpecreError::NonZeroExit) => process::exit(1),
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}
