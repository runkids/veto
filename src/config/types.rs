use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Allow,
    Low,
    Medium,
    High,
    Critical,
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
pub struct TotpConfig {
    pub enabled: bool,
    pub secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub auth: Option<AuthConfig>,
}
