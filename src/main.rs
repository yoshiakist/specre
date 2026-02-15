// @specre 01KHFFCX8BCDAYP8YHG0J65H0E
mod cli;
mod commands;
mod config;
mod template;
mod ulid;

use clap::Parser;
use cli::{Cli, Commands};
use std::process;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init(args) => commands::init::execute(args),
        Commands::New(args) => commands::new::execute(args),
        Commands::Index => commands::index::execute(),
        Commands::Status(args) => commands::status::execute(args),
        Commands::Trace(args) => commands::trace::execute(args),
        Commands::Orphans => commands::orphans::execute(),
        Commands::Tag(args) => commands::tag::execute(args),
        Commands::Coverage(args) => commands::coverage::execute(args),
        Commands::HealthCheck => commands::health_check::execute(),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
