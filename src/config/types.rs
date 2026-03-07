use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::rules::RiskLevel as RulesRiskLevel;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Allow,
    Low,
    Medium,
    High,
    Critical,
}

impl From<RulesRiskLevel> for RiskLevel {
    fn from(level: RulesRiskLevel) -> Self {
        match level {
            RulesRiskLevel::Allow => RiskLevel::Allow,
            RulesRiskLevel::Low => RiskLevel::Low,
            RulesRiskLevel::Medium => RiskLevel::Medium,
            RulesRiskLevel::High => RiskLevel::High,
            RulesRiskLevel::Critical => RiskLevel::Critical,
        }
    }
}

/// Authentication method - single method per risk level
/// Arrays are accepted for backwards compatibility but only first element is used
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AuthMethod {
    Single(String),
    #[serde(skip_serializing)]
    Multiple(Vec<String>), // Deprecated: only first element used
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub default: Option<String>,
    pub levels: Option<HashMap<String, AuthMethod>>,
    pub fallback: Option<HashMap<String, String>>,
    pub pin: Option<PinConfig>,
    pub touchid: Option<TouchIdConfig>,
    pub telegram: Option<TelegramConfig>,
    pub totp: Option<TotpConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TouchIdConfig {
    pub enabled: bool,
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
    pub timeout_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PinConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TotpConfig {
    pub enabled: bool,
    pub issuer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub auth: Option<AuthConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialize_minimal() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.auth.is_none());
    }

    #[test]
    fn test_config_deserialize_full() {
        let toml_str = r#"
[auth]
default = "pin"

[auth.pin]
enabled = true

[auth.totp]
enabled = true
issuer = "Veto"

[auth.touchid]
enabled = false
prompt = "Authorize veto"

[auth.telegram]
enabled = true
bot_token = "123:ABC"
chat_id = "456"
timeout_seconds = 60
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        let auth = config.auth.unwrap();
        assert_eq!(auth.default.as_deref(), Some("pin"));

        let pin = auth.pin.unwrap();
        assert!(pin.enabled);

        let totp = auth.totp.unwrap();
        assert!(totp.enabled);
        assert_eq!(totp.issuer.as_deref(), Some("Veto"));

        let touchid = auth.touchid.unwrap();
        assert!(!touchid.enabled);
        assert_eq!(touchid.prompt.as_deref(), Some("Authorize veto"));

        let telegram = auth.telegram.unwrap();
        assert!(telegram.enabled);
        assert_eq!(telegram.bot_token.as_deref(), Some("123:ABC"));
        assert_eq!(telegram.chat_id.as_deref(), Some("456"));
        assert_eq!(telegram.timeout_seconds, Some(60));
    }

    #[test]
    fn test_auth_method_single() {
        let method: AuthMethod = serde_json::from_str(r#""pin""#).unwrap();
        assert_eq!(method, AuthMethod::Single("pin".to_string()));
    }

    #[test]
    fn test_auth_method_array_compat() {
        let method: AuthMethod = serde_json::from_str(r#"["pin", "totp"]"#).unwrap();
        match method {
            AuthMethod::Multiple(v) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0], "pin");
                assert_eq!(v[1], "totp");
            }
            _ => panic!("expected AuthMethod::Multiple"),
        }
    }

    #[test]
    fn test_risk_level_from_rules() {
        assert_eq!(RiskLevel::from(RulesRiskLevel::Allow), RiskLevel::Allow);
        assert_eq!(RiskLevel::from(RulesRiskLevel::Low), RiskLevel::Low);
        assert_eq!(RiskLevel::from(RulesRiskLevel::Medium), RiskLevel::Medium);
        assert_eq!(RiskLevel::from(RulesRiskLevel::High), RiskLevel::High);
        assert_eq!(
            RiskLevel::from(RulesRiskLevel::Critical),
            RiskLevel::Critical
        );
    }

    #[test]
    fn test_risk_level_serde_roundtrip() {
        let serialized = serde_json::to_string(&RiskLevel::Critical).unwrap();
        assert_eq!(serialized, r#""critical""#);

        let deserialized: RiskLevel = serde_json::from_str(r#""allow""#).unwrap();
        assert_eq!(deserialized, RiskLevel::Allow);
    }
}
