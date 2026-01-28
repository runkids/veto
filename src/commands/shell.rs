//! Interactive shell wrapper with veto protection
//!
//! Every command is evaluated for risk and requires authentication
//! based on the configured risk level mappings.

use std::env;
use std::path::PathBuf;

use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::auth::{
    Authenticator, ConfirmAuth, PinAuth, TotpAuth, TouchIdAuth, TelegramAuth,
    AuthManager, manager::AsyncAuthBridge,
};
use crate::config::loader::load_config;
use crate::executor::ShellExecutor;
use crate::rules::{RulesEngine, RiskLevel, default_rules};

/// History file location
fn history_file() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("veto")
        .join("shell_history")
}

/// Run the interactive veto shell
pub fn run_shell() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "╔════════════════════════════════════════╗".cyan());
    println!("{}", "║         Veto Protected Shell           ║".cyan());
    println!("{}", "║  Every command is risk-evaluated       ║".cyan());
    println!("{}", "║  Ctrl+C to cancel, 'exit' to quit      ║".cyan());
    println!("{}", "╚════════════════════════════════════════╝".cyan());
    println!();

    let mut rl = DefaultEditor::new()?;

    // Load history
    let history_path = history_file();
    if let Some(parent) = history_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = rl.load_history(&history_path);

    let engine = RulesEngine::new(default_rules());
    let executor = ShellExecutor::new();

    loop {
        // Build prompt with current directory
        let cwd = env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "?".to_string());

        let prompt = format!("{} {} ", "veto".green().bold(), shorten_path(&cwd).blue());

        match rl.readline(&prompt) {
            Ok(line) => {
                let command = line.trim();

                // Skip empty lines
                if command.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(command);

                // Handle built-in commands
                if handle_builtin(command) {
                    continue;
                }

                // Check for exit
                if command == "exit" || command == "quit" {
                    println!("{}", "Goodbye!".cyan());
                    break;
                }

                // Evaluate risk
                let result = engine.evaluate(command);

                // Show risk level
                let level_display = match result.level {
                    RiskLevel::Allow => "ALLOW".green(),
                    RiskLevel::Low => "LOW".cyan(),
                    RiskLevel::Medium => "MEDIUM".yellow(),
                    RiskLevel::High => "HIGH".red(),
                    RiskLevel::Critical => "CRITICAL".red().bold(),
                };

                if !matches!(result.level, RiskLevel::Allow) {
                    println!("{} {}", "Risk:".dimmed(), level_display);
                    if let Some(reason) = &result.reason {
                        println!("{} {}", "Reason:".dimmed(), reason);
                    }
                }

                // Check if auth is required
                let needs_auth = !matches!(result.level, RiskLevel::Allow);

                if needs_auth {
                    // Get auth methods for this risk level
                    let methods = get_auth_methods(&result.level);

                    // Run authentication
                    match run_auth_chain(&methods, command) {
                        Ok(()) => {
                            // Authentication passed
                        }
                        Err(e) => {
                            let msg = e.to_string();
                            if msg.contains("cancelled") || msg.contains("Cancelled") {
                                println!("{}", "Cancelled.".dimmed());
                            } else {
                                println!("{} {}", "Denied:".red(), e);
                            }
                            continue;
                        }
                    }
                }

                // Execute the command
                match executor.execute(command) {
                    Ok(status) => {
                        if !status.success() {
                            if let Some(code) = status.code() {
                                println!("{} {}", "Exit:".dimmed(), code);
                            }
                        }
                    }
                    Err(e) => {
                        println!("{} {}", "Error:".red(), e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "Goodbye!".cyan());
                break;
            }
            Err(err) => {
                println!("{} {:?}", "Error:".red(), err);
                break;
            }
        }
    }

    // Save history
    let _ = rl.save_history(&history_path);

    Ok(())
}

/// Handle built-in shell commands
fn handle_builtin(command: &str) -> bool {
    let parts: Vec<&str> = command.split_whitespace().collect();

    match parts.first() {
        Some(&"cd") => {
            let target = parts.get(1).map(|s| *s).unwrap_or("~");
            let path = if target == "~" || target.starts_with("~/") {
                dirs::home_dir()
                    .map(|home| {
                        if target == "~" {
                            home
                        } else {
                            home.join(&target[2..])
                        }
                    })
                    .unwrap_or_else(|| PathBuf::from(target))
            } else {
                PathBuf::from(target)
            };

            if let Err(e) = env::set_current_dir(&path) {
                println!("{} {}: {}", "cd:".red(), path.display(), e);
            }
            true
        }
        Some(&"pwd") => {
            match env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(e) => println!("{} {}", "pwd:".red(), e),
            }
            true
        }
        Some(&"help") => {
            println!("{}", "Veto Shell Commands:".bold());
            println!("  {}      Change directory", "cd <dir>".cyan());
            println!("  {}          Print working directory", "pwd".cyan());
            println!("  {}         Show this help", "help".cyan());
            println!("  {}         Exit the shell", "exit".cyan());
            println!();
            println!("{}", "Shortcuts:".bold());
            println!("  {}       Cancel current operation", "Ctrl+C".cyan());
            println!("  {}       Exit the shell", "Ctrl+D".cyan());
            println!();
            println!("{}", "All other commands are risk-evaluated before execution.".dimmed());
            true
        }
        _ => false,
    }
}

/// Shorten path for display
fn shorten_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();
        if path.starts_with(&home_str) {
            return format!("~{}", &path[home_str.len()..]);
        }
    }
    path.to_string()
}

/// Get authentication methods for a risk level
fn get_auth_methods(level: &RiskLevel) -> Vec<String> {
    let config = match load_config() {
        Ok(c) => c,
        Err(_) => return vec![crate::auth::default_auth_method().to_string()],
    };

    let auth_config = match config.auth {
        Some(a) => a,
        None => return vec![crate::auth::default_auth_method().to_string()],
    };

    let manager = AuthManager::new(auth_config);
    manager.get_methods_for_level(&(*level).into())
}

/// Run authentication chain
fn run_auth_chain(methods: &[String], command: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config().unwrap_or_default();

    for method in methods {
        let authenticated = match method.as_str() {
            "confirm" => {
                let auth = ConfirmAuth::new();
                auth.authenticate(command)?
            }
            "pin" => {
                let auth = PinAuth::new();
                if !auth.is_available() {
                    return Err("PIN not configured".into());
                }
                auth.authenticate(command)?
            }
            "totp" => {
                let auth = TotpAuth::new();
                if !auth.is_available() {
                    return Err("TOTP not configured".into());
                }
                auth.authenticate(command)?
            }
            "touchid" => {
                let auth = TouchIdAuth::new();
                if !auth.is_available() {
                    return Err("Touch ID unavailable".into());
                }
                auth.authenticate(command)?
            }
            "telegram" => {
                let chat_id = config
                    .auth
                    .as_ref()
                    .and_then(|a| a.telegram.as_ref())
                    .and_then(|t| t.chat_id.as_ref())
                    .ok_or("Telegram not configured")?;

                let timeout = config
                    .auth
                    .as_ref()
                    .and_then(|a| a.telegram.as_ref())
                    .and_then(|t| t.timeout_seconds)
                    .unwrap_or(60);

                let auth = TelegramAuth::new(chat_id).with_timeout(timeout as u64);
                let bridge = AsyncAuthBridge::new(auth);
                bridge.authenticate(command)?
            }
            _ => {
                return Err(format!("Unknown auth method: {}", method).into());
            }
        };

        if !authenticated {
            return Err("Authentication cancelled".into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shorten_path() {
        // Test with a path that doesn't start with home
        assert_eq!(shorten_path("/usr/bin"), "/usr/bin");
    }

    #[test]
    fn test_handle_builtin_help() {
        assert!(handle_builtin("help"));
    }

    #[test]
    fn test_handle_builtin_unknown() {
        assert!(!handle_builtin("ls -la"));
    }
}
