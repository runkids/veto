use std::fs;
use std::path::PathBuf;
use colored::Colorize;
use dialoguer::Confirm;

use crate::config::get_config_dir;
use crate::commands::{run_auth_command, AuthCommands};
use crate::auth::keyring::SecureKeyring;

const DEFAULT_CONFIG_MACOS: &str = r#"# Veto Configuration
# AI operation guardian - verify before execute

[auth]
# Default authentication method: "confirm", "pin", "touchid", "telegram", "totp", "dialog"
#
# For Claude Code hooks, recommended methods:
#   - touchid: Touch ID prompt (macOS, works directly in hook)
#   - dialog:  macOS dialog box (works directly in hook)
#   - totp:    Time-based OTP (AI asks in chat, retry with VETO_TOTP=<code>)
#   - pin:     PIN code (AI asks in chat, retry with VETO_PIN=<code>)
#   - confirm: Simple yes/no (AI asks in chat, retry with VETO_CONFIRM=yes)
#   - telegram: Telegram bot approval (works directly in hook)
#
default = "touchid"

# Authentication levels for different risk levels
# [auth.levels]
# low = "pin"
# medium = "pin"
# high = "pin"
# critical = "telegram"

# TouchID configuration (macOS only)
# [auth.touchid]
# enabled = true
# prompt = "Authorize veto operation"

# PIN configuration - set via: veto auth set-pin
# [auth.pin]
# enabled = true

# TOTP configuration - set via: veto auth setup-totp
# [auth.totp]
# enabled = true

# Telegram configuration - set via: veto auth setup-telegram
# [auth.telegram]
# chat_id = "123456789"
# timeout_seconds = 60
"#;

const DEFAULT_CONFIG_OTHER: &str = r#"# Veto Configuration
# AI operation guardian - verify before execute

[auth]
# Default authentication method: "confirm", "pin", "touchid", "telegram", "totp", "dialog"
#
# For Claude Code hooks, recommended methods:
#   - touchid: Touch ID prompt (macOS, works directly in hook)
#   - dialog:  macOS dialog box (works directly in hook)
#   - totp:    Time-based OTP (AI asks in chat, retry with VETO_TOTP=<code>)
#   - pin:     PIN code (AI asks in chat, retry with VETO_PIN=<code>)
#   - confirm: Simple yes/no (AI asks in chat, retry with VETO_CONFIRM=yes)
#   - telegram: Telegram bot approval (works directly in hook)
#
default = "pin"

# Authentication levels for different risk levels
# [auth.levels]
# low = "pin"
# medium = "pin"
# high = "pin"
# critical = "telegram"

# TouchID configuration (macOS only)
# [auth.touchid]
# enabled = true
# prompt = "Authorize veto operation"

# PIN configuration - set via: veto auth set-pin
# [auth.pin]
# enabled = true

# TOTP configuration - set via: veto auth setup-totp
# [auth.totp]
# enabled = true

# Telegram configuration - set via: veto auth setup-telegram
# [auth.telegram]
# chat_id = "123456789"
# timeout_seconds = 60
"#;

const DEFAULT_RULES: &str = r#"# Veto Rules Configuration
# Define custom rules for command risk evaluation

# Whitelist - commands that are always allowed
[whitelist]
commands = [
    "ls*",
    "pwd",
    "echo *",
    "cat *",
    "head *",
    "tail *",
    "grep *",
    "cargo build*",
    "cargo test*",
    "npm run*",
    "git status*",
    "git log*",
    "git diff*",
]

# Critical risk commands
# [[critical]]
# category = "custom-critical"
# patterns = ["dangerous-command*"]
# reason = "Custom critical operation"

# High risk commands
# [[high]]
# category = "custom-high"
# patterns = ["risky-command*"]
# reason = "Custom high-risk operation"

# Medium risk commands
# [[medium]]
# category = "custom-medium"
# patterns = ["moderate-command*"]
# reason = "Custom medium-risk operation"
"#;

pub fn run_init(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir();
    let config_path = config_dir.join("config.toml");
    let rules_path = config_dir.join("rules.toml");
    let default_config = if cfg!(target_os = "macos") {
        DEFAULT_CONFIG_MACOS
    } else {
        DEFAULT_CONFIG_OTHER
    };

    println!("{}", "Initializing veto configuration...".bold());

    // Create config directory
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
        println!("  {} {}", "Created".green(), config_dir.display());
    }

    // Write config.toml
    let created_config = if write_file_if_needed(&config_path, default_config, force)? {
        println!("  {} {}", "Created".green(), config_path.display());
        true
    } else {
        println!("  {} {} (use --force to overwrite)", "Exists".yellow(), config_path.display());
        false
    };

    // Write rules.toml
    if write_file_if_needed(&rules_path, DEFAULT_RULES, force)? {
        println!("  {} {}", "Created".green(), rules_path.display());
    } else {
        println!("  {} {} (use --force to overwrite)", "Exists".yellow(), rules_path.display());
    }

    if cfg!(target_os = "linux") && created_config && !SecureKeyring::has_pin() {
        println!();
        println!("{}", "PIN is the default auth method on Linux.".cyan());
        let should_set = Confirm::new()
            .with_prompt("Set a PIN now?")
            .default(true)
            .interact()
            .unwrap_or(false);

        if should_set {
            if let Err(e) = run_auth_command(AuthCommands::SetPin) {
                eprintln!("{} {}", "Failed to set PIN:".red(), e);
            }
        } else {
            println!("You can set it later with: veto auth set-pin");
        }
    }

    println!();
    println!("{}", "Configuration initialized!".green().bold());
    println!("Edit {} to customize settings.", config_path.display());

    Ok(())
}

fn write_file_if_needed(path: &PathBuf, content: &str, force: bool) -> Result<bool, std::io::Error> {
    if path.exists() && !force {
        return Ok(false);
    }
    fs::write(path, content)?;
    Ok(true)
}
