use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "veto")]
#[command(about = "AI operation guardian - verify before execute")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress output (exit code only)
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check command risk level (no execute)
    Check {
        /// Command to check
        command: String,
    },
    /// Verify and execute command
    Exec {
        /// Command to execute
        command: String,
    },
    /// Initialize config files
    Init {
        /// Overwrite existing config
        #[arg(long)]
        force: bool,
    },
    /// Verify installation and config
    Doctor,
    /// Start interactive shell wrapper
    Shell,
}
