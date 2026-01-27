mod cli;
mod config;
mod rules;
mod auth;
mod executor;

use clap::Parser;
use colored::Colorize;
use cli::{Cli, Commands};
use rules::{RulesEngine, RiskLevel, default_rules};
use auth::{Authenticator, ConfirmAuth};
use executor::ShellExecutor;

fn main() {
    let cli = Cli::parse();
    let engine = RulesEngine::new(default_rules());

    match cli.command {
        Commands::Check { command } => {
            run_check(&engine, &command, cli.verbose);
        }
        Commands::Exec { command } => {
            run_exec(&engine, &command, cli.verbose);
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

fn run_check(engine: &RulesEngine, command: &str, verbose: bool) {
    let result = engine.evaluate(command);

    let level_colored = match result.level {
        RiskLevel::Allow => "ALLOW".green(),
        RiskLevel::Low => "LOW".cyan(),
        RiskLevel::Medium => "MEDIUM".yellow(),
        RiskLevel::High => "HIGH".red(),
        RiskLevel::Critical => "CRITICAL".red().bold(),
    };

    println!("{} {}", "Risk:".bold(), level_colored);

    if verbose {
        if let Some(cat) = &result.category {
            println!("{} {}", "Category:".bold(), cat);
        }
        if let Some(reason) = &result.reason {
            println!("{} {}", "Reason:".bold(), reason);
        }
        if let Some(pattern) = &result.matched_pattern {
            println!("{} {}", "Pattern:".bold(), pattern);
        }
    }

    // Exit with appropriate code
    let exit_code = match result.level {
        RiskLevel::Allow => 0,
        RiskLevel::Low => 1,
        RiskLevel::Medium => 2,
        RiskLevel::High => 3,
        RiskLevel::Critical => 4,
    };
    std::process::exit(exit_code);
}

fn run_exec(engine: &RulesEngine, command: &str, verbose: bool) {
    let result = engine.evaluate(command);

    if verbose {
        let level_colored = match result.level {
            RiskLevel::Allow => "ALLOW".green(),
            RiskLevel::Low => "LOW".cyan(),
            RiskLevel::Medium => "MEDIUM".yellow(),
            RiskLevel::High => "HIGH".red(),
            RiskLevel::Critical => "CRITICAL".red().bold(),
        };
        println!("{} {}", "Risk:".bold(), level_colored);
    }

    // Check if auth is required
    let needs_auth = !matches!(result.level, RiskLevel::Allow);

    if needs_auth {
        let auth = ConfirmAuth::new();

        // Show warning for high-risk commands
        if matches!(result.level, RiskLevel::High | RiskLevel::Critical) {
            println!("{}", "⚠️  High-risk operation detected!".red().bold());
            if let Some(reason) = &result.reason {
                println!("{} {}", "Reason:".bold(), reason);
            }
        }

        match auth.authenticate(command) {
            Ok(true) => {
                // Proceed with execution
            }
            Ok(false) | Err(_) => {
                println!("{}", "Operation cancelled.".yellow());
                std::process::exit(1);
            }
        }
    }

    // Execute the command
    let executor = ShellExecutor::new();
    match executor.execute(command) {
        Ok(status) => {
            let code = status.code().unwrap_or(1);
            std::process::exit(code);
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            std::process::exit(1);
        }
    }
}
