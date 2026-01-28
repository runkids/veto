use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Allow,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Allow => write!(f, "ALLOW"),
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::High => write!(f, "HIGH"),
            RiskLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub category: String,
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub paths: Vec<String>,
    pub reason: Option<String>,
    /// Enable challenge-response mechanism for this rule
    /// When true, requires challenge code sent via notification
    #[serde(default)]
    pub challenge: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Whitelist {
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Rules {
    #[serde(default)]
    pub critical: Vec<Rule>,
    #[serde(default)]
    pub high: Vec<Rule>,
    #[serde(default)]
    pub medium: Vec<Rule>,
    #[serde(default)]
    pub low: Vec<Rule>,
    #[serde(default)]
    pub whitelist: Whitelist,
}

#[derive(Debug, Clone)]
pub struct RiskResult {
    pub level: RiskLevel,
    pub category: Option<String>,
    pub reason: Option<String>,
    pub matched_pattern: Option<String>,
    /// Whether this rule requires challenge-response authentication
    pub challenge: bool,
}
