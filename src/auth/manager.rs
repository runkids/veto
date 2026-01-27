//! Authentication manager - handles auth method selection and chain execution

use super::{AuthError, AuthResult, Authenticator};
use crate::config::{AuthConfig, AuthMethod, RiskLevel};

/// Async authenticator trait for methods like Telegram
#[async_trait::async_trait]
pub trait AsyncAuthenticator: Send + Sync {
    fn is_available(&self) -> bool;
    async fn authenticate_async(&self, command: &str) -> AuthResult;
}

/// Bridge to run async authenticators synchronously
pub struct AsyncAuthBridge<T: AsyncAuthenticator> {
    inner: T,
}

impl<T: AsyncAuthenticator> AsyncAuthBridge<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: AsyncAuthenticator + 'static> Authenticator for AsyncAuthBridge<T> {
    fn is_available(&self) -> bool {
        self.inner.is_available()
    }

    fn authenticate(&self, command: &str) -> AuthResult {
        // Create a new runtime for the async operation
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| AuthError::Failed(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(self.inner.authenticate_async(command))
    }
}

/// Authentication manager - selects and executes authentication methods
pub struct AuthManager {
    config: AuthConfig,
}

impl AuthManager {
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }

    /// Get authentication method for a risk level
    pub fn get_methods_for_level(&self, level: &RiskLevel) -> Vec<String> {
        let level_key = match level {
            RiskLevel::Allow => return vec![],  // No auth needed
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        };

        // Check levels config first
        if let Some(levels) = &self.config.levels {
            if let Some(method) = levels.get(level_key) {
                return match method {
                    AuthMethod::Single(m) => vec![m.clone()],
                    // For backwards compatibility, array configs use first element only
                    AuthMethod::Multiple(ms) => ms.first().cloned().map(|m| vec![m]).unwrap_or_default(),
                };
            }
        }

        // Fall back to default
        if let Some(default) = &self.config.default {
            vec![default.clone()]
        } else {
            vec!["confirm".to_string()]
        }
    }
}

/// Factory for creating authenticators by name
pub struct AuthenticatorFactory;

impl AuthenticatorFactory {
    /// Check if an auth method is available
    pub fn is_available(method: &str) -> bool {
        match method {
            "confirm" => true,
            "pin" => super::keyring::SecureKeyring::has_pin(),
            "totp" => super::keyring::SecureKeyring::has_totp(),
            "touchid" => cfg!(target_os = "macos"),
            "telegram" => {
                super::keyring::SecureKeyring::has_telegram()
            }
            _ => false,
        }
    }

    /// List all configured auth methods
    pub fn list_configured() -> Vec<(&'static str, bool)> {
        vec![
            ("confirm", true),
            ("pin", super::keyring::SecureKeyring::has_pin()),
            ("totp", super::keyring::SecureKeyring::has_totp()),
            ("touchid", cfg!(target_os = "macos")),
            ("telegram", super::keyring::SecureKeyring::has_telegram()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_get_methods_for_level() {
        let mut levels = HashMap::new();
        levels.insert("low".to_string(), AuthMethod::Single("confirm".to_string()));
        levels.insert("high".to_string(), AuthMethod::Multiple(vec![
            "pin".to_string(),
            "totp".to_string(),
        ]));

        let config = AuthConfig {
            default: Some("confirm".to_string()),
            levels: Some(levels),
            ..Default::default()
        };

        let manager = AuthManager::new(config);

        assert_eq!(manager.get_methods_for_level(&RiskLevel::Allow), Vec::<String>::new());
        assert_eq!(manager.get_methods_for_level(&RiskLevel::Low), vec!["confirm"]);
        assert_eq!(manager.get_methods_for_level(&RiskLevel::High), vec!["pin", "totp"]);
        // Medium not configured, should fall back to default
        assert_eq!(manager.get_methods_for_level(&RiskLevel::Medium), vec!["confirm"]);
    }
}
