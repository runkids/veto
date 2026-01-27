mod cli;
mod config;
mod rules;
mod auth;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { command } => {
            println!("Checking: {}", command);
        }
        Commands::Exec { command } => {
            println!("Executing: {}", command);
        }
        Commands::Init { force } => {
            println!("Init (force={})", force);
        }
        Commands::Doctor => {
            println!("Running doctor...");
        }
        Commands::Shell => {
            println!("Starting shell...");
        }
    }
}
