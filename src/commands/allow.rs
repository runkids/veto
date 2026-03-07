use chrono::{DateTime, Duration, Utc};
use colored::Colorize;
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::loader::get_config_dir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllowOnceEntry {
    pub command: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub one_shot: bool,
}

pub fn allow_once_path() -> PathBuf {
    get_config_dir().join("allow-once.json")
}

pub fn load_allow_once() -> Vec<AllowOnceEntry> {
    let path = allow_once_path();
    if !path.exists() {
        return Vec::new();
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save_allow_once(entries: &[AllowOnceEntry]) -> Result<(), String> {
    let path = allow_once_path();
    let json =
        serde_json::to_string_pretty(entries).map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

/// Check if a command matches any allow-once entry.
/// Consumes one-shot entries and removes expired entries on match.
pub fn check_allow_once(command: &str) -> bool {
    let mut entries = load_allow_once();
    let now = Utc::now();

    // Remove expired entries
    entries.retain(|e| match e.expires_at {
        Some(exp) => exp > now,
        None => true,
    });

    let mut matched = false;
    let mut updated = Vec::new();

    for entry in entries {
        if !matched && glob_match_simple(&entry.command, command) {
            matched = true;
            // Consume one-shot entries (don't keep them)
            if !entry.one_shot {
                updated.push(entry);
            }
        } else {
            updated.push(entry);
        }
    }

    if matched {
        let _ = save_allow_once(&updated);
    }

    matched
}

/// Add an allow-once entry.
/// If `ttl_seconds` is None, creates a one-shot entry (consumed on first use).
/// If `ttl_seconds` is Some, creates a time-limited entry.
pub fn add_allow_once(command: &str, ttl_seconds: Option<i64>) -> Result<(), String> {
    let mut entries = load_allow_once();

    let (expires_at, one_shot) = match ttl_seconds {
        Some(secs) => (Some(Utc::now() + Duration::seconds(secs)), false),
        None => (None, true),
    };

    entries.push(AllowOnceEntry {
        command: command.to_string(),
        expires_at,
        one_shot,
    });

    save_allow_once(&entries)
}

/// Remove expired entries from allow-once list. Returns count of entries removed.
pub fn clean_allow_once() -> Result<usize, String> {
    let entries = load_allow_once();
    let now = Utc::now();
    let before = entries.len();

    let remaining: Vec<AllowOnceEntry> = entries
        .into_iter()
        .filter(|e| match e.expires_at {
            Some(exp) => exp > now,
            None => true, // one-shot entries without expiry are kept
        })
        .collect();

    let removed = before - remaining.len();
    save_allow_once(&remaining)?;
    Ok(removed)
}

/// Parse a human-readable TTL string into seconds.
/// Supported formats: "60s", "30m", "1h", "24h", "7d"
pub fn parse_ttl(ttl: &str) -> Result<i64, String> {
    if ttl.is_empty() {
        return Err("TTL string cannot be empty".to_string());
    }

    let (num_str, suffix) = ttl.split_at(ttl.len() - 1);
    let value: i64 = num_str
        .parse()
        .map_err(|_| format!("Invalid TTL format: '{}'", ttl))?;

    match suffix {
        "s" => Ok(value),
        "m" => Ok(value * 60),
        "h" => Ok(value * 3600),
        "d" => Ok(value * 86400),
        _ => Err(format!("Unknown TTL suffix '{}' in '{}'. Use s/m/h/d.", suffix, ttl)),
    }
}

/// Simple glob matching, consistent with the engine's approach.
pub fn glob_match_simple(pattern: &str, text: &str) -> bool {
    if pattern.contains('*') {
        if let Ok(pat) = Pattern::new(pattern) {
            return pat.matches(text);
        }
        // Fallback: simple contains
        let core = pattern.trim_matches('*');
        return text.contains(core);
    }
    text == pattern
}

/// Add pattern to allowlist TOML file
pub fn add_to_allowlist(pattern: &str, global: bool) -> Result<(), Box<dyn std::error::Error>> {
    let path = if global {
        get_config_dir().join("allowlist.toml")
    } else {
        std::path::PathBuf::from(".veto/allowlist.toml")
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };

    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .unwrap_or_else(|_| toml_edit::DocumentMut::new());

    if !doc.contains_key("whitelist") {
        doc["whitelist"] = toml_edit::Item::Table(toml_edit::Table::new());
    }

    let whitelist = doc["whitelist"].as_table_mut().unwrap();
    if !whitelist.contains_key("commands") {
        whitelist["commands"] = toml_edit::value(toml_edit::Array::new());
    }

    let commands = whitelist["commands"].as_array_mut().unwrap();

    let already_exists = commands.iter().any(|v| v.as_str() == Some(pattern));
    if already_exists {
        println!("{} \"{}\" already in allowlist", "Skip:".yellow(), pattern);
        return Ok(());
    }

    commands.push(pattern);
    std::fs::write(&path, doc.to_string())?;

    let layer = if global { "global" } else { "project" };
    println!(
        "{} \"{}\" → {} allowlist",
        "Added:".green().bold(),
        pattern,
        layer
    );
    Ok(())
}

/// Remove pattern from allowlist TOML file
pub fn remove_from_allowlist(
    pattern: &str,
    global: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = if global {
        get_config_dir().join("allowlist.toml")
    } else {
        std::path::PathBuf::from(".veto/allowlist.toml")
    };

    if !path.exists() {
        println!("{} allowlist file not found", "Error:".red());
        return Ok(());
    }

    let content = std::fs::read_to_string(&path)?;
    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| format!("Failed to parse allowlist: {}", e))?;

    if let Some(whitelist) = doc.get_mut("whitelist").and_then(|w| w.as_table_mut()) {
        if let Some(commands) = whitelist.get_mut("commands").and_then(|c| c.as_array_mut()) {
            let before = commands.len();
            commands.retain(|v| v.as_str() != Some(pattern));
            if commands.len() < before {
                std::fs::write(&path, doc.to_string())?;
                let layer = if global { "global" } else { "project" };
                println!(
                    "{} \"{}\" from {} allowlist",
                    "Removed:".green().bold(),
                    pattern,
                    layer
                );
                return Ok(());
            }
        }
    }

    println!("{} \"{}\" not found in allowlist", "Error:".red(), pattern);
    Ok(())
}

/// List all allowlist entries with source labels
pub fn list_allowlist() {
    let global_path = get_config_dir().join("allowlist.toml");
    let project_path = std::path::PathBuf::from(".veto/allowlist.toml");

    let mut found = false;

    // Global
    if let Some(wl) = crate::config::loader::read_allowlist_file_pub(&global_path) {
        if !wl.commands.is_empty() {
            println!("{}", "Global (~/.veto/allowlist.toml):".bold());
            for cmd in &wl.commands {
                println!("  {}", cmd);
            }
            found = true;
        }
    }

    // Project
    if let Some(wl) = crate::config::loader::read_allowlist_file_pub(&project_path) {
        if !wl.commands.is_empty() {
            if found {
                println!();
            }
            println!("{}", "Project (.veto/allowlist.toml):".bold());
            for cmd in &wl.commands {
                println!("  {}", cmd);
            }
            found = true;
        }
    }

    // Allow-once
    let once_entries = load_allow_once();
    let now = chrono::Utc::now();
    let active: Vec<_> = once_entries
        .iter()
        .filter(|e| match e.expires_at {
            Some(exp) => now <= exp,
            None => true,
        })
        .collect();

    if !active.is_empty() {
        if found {
            println!();
        }
        println!("{}", "Temporary (allow-once):".bold());
        for entry in &active {
            let label = if entry.one_shot {
                "one-shot".to_string()
            } else if let Some(exp) = entry.expires_at {
                format!("expires {}", exp.format("%Y-%m-%d %H:%M UTC"))
            } else {
                "permanent".to_string()
            };
            println!("  {} ({})", entry.command, label.dimmed());
        }
        found = true;
    }

    if !found {
        println!("{}", "No allowlist entries found.".dimmed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match_simple() {
        assert!(glob_match_simple("git push*", "git push origin main"));
        assert!(glob_match_simple("ls", "ls"));
        assert!(!glob_match_simple("ls", "cat"));
    }

    #[test]
    fn test_allow_once_entry_serialization() {
        let entry = AllowOnceEntry {
            command: "git push".to_string(),
            expires_at: None,
            one_shot: true,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: AllowOnceEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.command, "git push");
        assert!(parsed.one_shot);
    }

    #[test]
    fn test_parse_ttl() {
        assert_eq!(parse_ttl("30m").unwrap(), 1800);
        assert_eq!(parse_ttl("1h").unwrap(), 3600);
        assert_eq!(parse_ttl("24h").unwrap(), 86400);
        assert_eq!(parse_ttl("7d").unwrap(), 604800);
        assert_eq!(parse_ttl("60s").unwrap(), 60);
        assert!(parse_ttl("abc").is_err());
        assert!(parse_ttl("").is_err());
    }
}
