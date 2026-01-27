use std::path::PathBuf;
use thiserror::Error;

use super::Config;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found: {0}")]
    NotFound(PathBuf),
    #[error("Failed to read config: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
}

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("veto")
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
default = "confirm"

[auth.touchid]
enabled = true
prompt = "Verify operation"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.auth.is_some());
        let auth = config.auth.unwrap();
        assert_eq!(auth.default, Some("confirm".to_string()));
        assert!(auth.touchid.is_some());
        assert!(auth.touchid.unwrap().enabled);
    }
}
