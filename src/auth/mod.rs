//! Authentication module
//!
//! Provides various authentication methods for veto:
//! - confirm: Simple y/n confirmation
//! - pin: PIN code with Argon2 hashing
//! - totp: Time-based OTP (Google Authenticator compatible)
//! - touchid: macOS Touch ID (platform-specific)
//! - telegram: Async Telegram bot approval
//! - challenge: Challenge-response for preventing AI replay attacks

mod confirm;
mod pin;
mod totp;
mod touchid;
mod telegram;
mod dialog;
pub mod challenge;
pub mod keyring;
pub mod manager;

pub use confirm::*;
pub use pin::*;
pub use totp::*;
pub use touchid::*;
pub use telegram::*;
pub use dialog::*;
pub use manager::{AuthManager, AuthenticatorFactory};
pub use challenge::{Challenge, notify_challenge, verify_response};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Authentication cancelled by user")]
    Cancelled,
    #[error("Authentication failed: {0}")]
    Failed(String),
    #[error("Authentication method not available: {0}")]
    NotAvailable(String),
    #[error("Authentication timeout")]
    Timeout,
}

pub type AuthResult = Result<bool, AuthError>;

/// Context information for authentication prompts
#[derive(Debug, Clone, Default)]
pub struct AuthContext {
    /// Working directory where the AI tool is running
    pub cwd: Option<String>,
    /// Session ID (conversation identifier)
    pub session_id: Option<String>,
    /// Tool name (e.g., "Write", "Edit", "Bash")
    pub tool_name: Option<String>,
    /// File path being operated on (for file operations)
    pub file_path: Option<String>,
}

impl AuthContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_tool_name(mut self, tool_name: impl Into<String>) -> Self {
        self.tool_name = Some(tool_name.into());
        self
    }

    pub fn with_file_path(mut self, file_path: impl Into<String>) -> Self {
        self.file_path = Some(file_path.into());
        self
    }

    /// Format context for display in dialog/touchid prompts
    pub fn format_for_display(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref tool) = self.tool_name {
            parts.push(format!("Tool: {}", tool));
        }

        if let Some(ref path) = self.file_path {
            parts.push(format!("File: {}", path));
        }

        if let Some(ref cwd) = self.cwd {
            // Shorten home directory
            let display_cwd = if let Some(home) = dirs::home_dir() {
                if let Some(home_str) = home.to_str() {
                    cwd.replace(home_str, "~")
                } else {
                    cwd.clone()
                }
            } else {
                cwd.clone()
            };
            parts.push(format!("Dir: {}", display_cwd));
        }

        if let Some(ref session) = self.session_id {
            // Show truncated session ID
            let short_id = if session.len() > 8 {
                &session[..8]
            } else {
                session
            };
            parts.push(format!("Session: {}...", short_id));
        }

        parts.join("\\n")
    }
}

/// Synchronous authenticator trait
pub trait Authenticator: Send + Sync {
    fn is_available(&self) -> bool;
    fn authenticate(&self, command: &str) -> AuthResult;

    /// Authenticate with additional context information
    fn authenticate_with_context(&self, command: &str, _context: &AuthContext) -> AuthResult {
        // Default implementation ignores context
        self.authenticate(command)
    }
}

/// Platform-default auth method (avoid "confirm" as the default)
pub fn default_auth_method() -> &'static str {
    if cfg!(target_os = "macos") {
        "touchid"
    } else {
        "pin"
    }
}
