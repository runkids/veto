use colored::Colorize;

use crate::auth::keyring::SecureKeyring;
use crate::commands::is_claude_configured;
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

    // Check keyring
    println!();
    println!("{}", "Keyring Status:".bold());
    print!("  Backend: ");
    println!("{}", SecureKeyring::backend_name().cyan());

    print!("  PIN configured: ");
    if SecureKeyring::has_pin() {
        println!("{}", "yes".green());
    } else {
        println!("{}", "no".dimmed());
    }

    print!("  TOTP configured: ");
    if SecureKeyring::has_totp() {
        println!("{}", "yes".green());
    } else {
        println!("{}", "no".dimmed());
    }

    print!("  Telegram configured: ");
    if SecureKeyring::has_telegram() {
        println!("{}", "yes".green());
    } else {
        println!("{}", "no".dimmed());
    }

    // Test keyring write/read
    print!("  Keyring test: ");
    let test_key = "veto.doctor.test";
    let test_value = "doctor-test-value";
    match SecureKeyring::set(test_key, test_value) {
        Ok(_) => {
            match SecureKeyring::get(test_key) {
                Ok(v) if v == test_value => {
                    let _ = SecureKeyring::delete(test_key);
                    println!("{} (write/read OK)", "✓".green());
                }
                Ok(_) => {
                    let _ = SecureKeyring::delete(test_key);
                    println!("{} (read mismatch)", "✗".red());
                    all_ok = false;
                }
                Err(e) => {
                    println!("{} (read failed: {})", "✗".red(), e);
                    all_ok = false;
                }
            }
        }
        Err(e) => {
            println!("{} (write failed: {})", "✗".red(), e);
            all_ok = false;
        }
    }

    // Claude Code integration
    println!();
    println!("{}", "Claude Code Integration:".bold());
    let claude_settings = dirs::home_dir()
        .map(|h| h.join(".claude").join("settings.json"));

    print!("  settings.json: ");
    match claude_settings {
        Some(path) if path.exists() => {
            if is_claude_configured() {
                println!("{} veto hooks configured", "✓".green());
            } else {
                println!("{} exists but no veto hooks (run `veto setup claude`)", "○".yellow());
            }
        }
        Some(_) => {
            println!("{} not found (optional)", "○".yellow());
        }
        None => {
            println!("{} cannot determine path", "○".yellow());
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
