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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AuthMethod {
    Single(String),
    Multiple(Vec<String>),
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
