//! Authentication module
//!
//! Provides various authentication methods for veto:
//! - confirm: Simple y/n confirmation
//! - pin: PIN code with Argon2 hashing
//! - totp: Time-based OTP (Google Authenticator compatible)
//! - touchid: macOS Touch ID (platform-specific)
//! - telegram: Async Telegram bot approval

mod confirm;
mod pin;
mod totp;
mod touchid;
mod telegram;
mod dialog;
pub mod keyring;
pub mod manager;

pub use confirm::*;
pub use pin::*;
pub use totp::*;
pub use touchid::*;
pub use telegram::*;
pub use dialog::*;
pub use manager::{AuthManager, AuthenticatorFactory};

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

/// Synchronous authenticator trait
pub trait Authenticator: Send + Sync {
    fn is_available(&self) -> bool;
    fn authenticate(&self, command: &str) -> AuthResult;
}
