mod confirm;
mod pin;

pub use confirm::*;
pub use pin::*;

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

pub trait Authenticator {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn authenticate(&self, command: &str) -> AuthResult;
}
