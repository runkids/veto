use std::path::PathBuf;
use thiserror::Error;

use super::Config;
use crate::rules::{Rules, default_rules};

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Failed to edit config: {0}")]
    EditError(String),
}

pub fn get_config_dir() -> PathBuf {
    // Use VETO_HOME or ~/.veto
    std::env::var("VETO_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".veto")
        })
}

pub fn load_config() -> Result<Config, ConfigError> {
    let config_path = get_config_dir().join("config.toml");

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

/// Load rules from ~/.veto/rules.toml and merge with defaults
/// User rules take priority (checked first)
pub fn load_rules() -> Rules {
    let rules_path = get_config_dir().join("rules.toml");
    let defaults = default_rules();

    if !rules_path.exists() {
        return defaults;
    }

    let content = match std::fs::read_to_string(&rules_path) {
        Ok(c) => c,
        Err(_) => return defaults,
    };

    let user_rules: Rules = match toml::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[veto] Warning: Failed to parse rules.toml: {}", e);
            return defaults;
        }
    };

    // Merge: user rules first (higher priority), then defaults
    Rules {
        critical: [user_rules.critical, defaults.critical].concat(),
        high: [user_rules.high, defaults.high].concat(),
        medium: [user_rules.medium, defaults.medium].concat(),
        low: [user_rules.low, defaults.low].concat(),
        whitelist: crate::rules::Whitelist {
            commands: [user_rules.whitelist.commands, defaults.whitelist.commands].concat(),
            paths: [user_rules.whitelist.paths, defaults.whitelist.paths].concat(),
        },
    }
}

/// Update Telegram configuration in config.toml
/// This preserves existing config and only updates/adds the telegram section
pub fn update_telegram_config(chat_id: &str, timeout_seconds: Option<u32>) -> Result<(), ConfigError> {
    let config_path = get_config_dir().join("config.toml");

    let content = if config_path.exists() {
        std::fs::read_to_string(&config_path)?
    } else {
        String::new()
    };

    // Parse as toml_edit to preserve formatting and comments
    let mut doc = content.parse::<toml_edit::DocumentMut>()
        .map_err(|e| ConfigError::EditError(e.to_string()))?;

    // Ensure [auth] section exists
    if !doc.contains_key("auth") {
        doc["auth"] = toml_edit::Item::Table(toml_edit::Table::new());
    }

    // Get or create [auth.telegram] section
    let auth = doc["auth"].as_table_mut().unwrap();
    if !auth.contains_key("telegram") {
        auth["telegram"] = toml_edit::Item::Table(toml_edit::Table::new());
    }

    let telegram = auth["telegram"].as_table_mut().unwrap();
    telegram["enabled"] = toml_edit::value(true);
    telegram["chat_id"] = toml_edit::value(chat_id);
    if let Some(timeout) = timeout_seconds {
        telegram["timeout_seconds"] = toml_edit::value(timeout as i64);
    } else if !telegram.contains_key("timeout_seconds") {
        telegram["timeout_seconds"] = toml_edit::value(60i64);
    }

    std::fs::write(&config_path, doc.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_empty_config() {
        let config = Config::default();
        assert!(config.auth.is_none());
    }

    #[test]
    fn test_parse_config_toml() {
        let toml_str = r#"
[auth]
default = "pin"

[auth.touchid]
enabled = true
prompt = "Verify operation"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.auth.is_some());
        let auth = config.auth.unwrap();
        assert_eq!(auth.default, Some("pin".to_string()));
        assert!(auth.touchid.is_some());
        assert!(auth.touchid.unwrap().enabled);
    }
}
