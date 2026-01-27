use colored::Colorize;

use crate::config::{get_config_dir, load_config};

pub fn run_doctor() {
    println!("{}", "Veto Doctor".bold());
    println!("{}", "===========".bold());
    println!();

    let mut all_ok = true;

    // Check config directory
    let config_dir = get_config_dir();
    print!("Config directory: ");
    if config_dir.exists() {
        println!("{} {}", "✓".green(), config_dir.display());
    } else {
        println!("{} {} (run `veto init`)", "✗".red(), config_dir.display());
        all_ok = false;
    }

    // Check config.toml
    let config_path = config_dir.join("config.toml");
    print!("Config file: ");
    if config_path.exists() {
        println!("{} {}", "✓".green(), config_path.display());

        // Try to parse config
        print!("Config valid: ");
        match load_config() {
            Ok(_) => println!("{}", "✓".green()),
            Err(e) => {
                println!("{} {}", "✗".red(), e);
                all_ok = false;
            }
        }
    } else {
        println!("{} not found (run `veto init`)", "✗".red());
        all_ok = false;
    }

    // Check rules.toml
    let rules_path = config_dir.join("rules.toml");
    print!("Rules file: ");
    if rules_path.exists() {
        println!("{} {}", "✓".green(), rules_path.display());
    } else {
        println!("{} not found (optional)", "○".yellow());
    }

    // Check shell
    print!("Shell: ");
    match std::env::var("SHELL") {
        Ok(shell) => println!("{} {}", "✓".green(), shell),
        Err(_) => {
            println!("{} SHELL not set, using /bin/bash", "○".yellow());
        }
    }

    // Summary
    println!();
    if all_ok {
        println!("{}", "All checks passed!".green().bold());
    } else {
        println!("{}", "Some checks failed. Run `veto init` to fix.".yellow());
    }
}
