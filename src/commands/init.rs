use std::fs;
use std::path::PathBuf;
use colored::Colorize;

use crate::config::get_config_dir;

const DEFAULT_CONFIG: &str = r#"# Veto Configuration
# AI operation guardian - verify before execute

[auth]
# Default authentication method: "confirm", "pin", "touchid", "telegram", "totp"
default = "confirm"

# Authentication levels for different risk levels
# [auth.levels]
# low = "confirm"
# medium = "confirm"
# high = "pin"
# critical = ["pin", "confirm"]

# TouchID configuration (macOS only)
# [auth.touchid]
# enabled = true
# prompt = "Authorize veto operation"

# PIN configuration
# [auth.pin]
# enabled = true
# hash = ""  # Set via `veto config set-pin`
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

    println!("{}", "Initializing veto configuration...".bold());

    // Create config directory
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
        println!("  {} {}", "Created".green(), config_dir.display());
    }

    // Write config.toml
    if write_file_if_needed(&config_path, DEFAULT_CONFIG, force)? {
        println!("  {} {}", "Created".green(), config_path.display());
    } else {
        println!("  {} {} (use --force to overwrite)", "Exists".yellow(), config_path.display());
    }

    // Write rules.toml
    if write_file_if_needed(&rules_path, DEFAULT_RULES, force)? {
        println!("  {} {}", "Created".green(), rules_path.display());
    } else {
        println!("  {} {} (use --force to overwrite)", "Exists".yellow(), rules_path.display());
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
