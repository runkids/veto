//! PIN authentication using Argon2 hashing
//!
//! PIN is stored as an Argon2 hash in the system keychain.
//! This provides secure storage without exposing the actual PIN.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use colored::Colorize;
use dialoguer::Password;

use super::keyring::SecureKeyring;
use super::{AuthError, AuthResult, Authenticator};

/// PIN authenticator using Argon2 hash verification
pub struct PinAuth;

impl PinAuth {
    pub fn new() -> Self {
        Self
    }

    /// Set a new PIN (stores hash in keychain)
    pub fn set_pin(pin: &str) -> Result<(), AuthError> {
        if pin.len() < 4 {
            return Err(AuthError::Failed(
                "PIN must be at least 4 characters".to_string(),
            ));
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let hash = argon2
            .hash_password(pin.as_bytes(), &salt)
            .map_err(|e| AuthError::Failed(format!("Failed to hash PIN: {}", e)))?;

        SecureKeyring::set_pin(hash.to_string().as_str(), salt.as_str())
            .map_err(|e| AuthError::Failed(format!("Failed to store PIN: {}", e)))?;

        Ok(())
    }

    /// Delete stored PIN
    pub fn delete_pin() -> Result<(), AuthError> {
        SecureKeyring::delete_pin()
            .map_err(|e| AuthError::Failed(format!("Failed to delete PIN: {}", e)))
    }

    /// Verify a PIN directly (for non-interactive use)
    pub fn verify_direct(&self, pin: &str) -> Result<bool, AuthError> {
        self.verify_pin(pin)
    }

    /// Verify a PIN against the stored hash
    fn verify_pin(&self, pin: &str) -> Result<bool, AuthError> {
        let stored_hash = SecureKeyring::get_pin_hash()
            .map_err(|_| AuthError::NotAvailable("PIN not configured".to_string()))?;

        let parsed_hash = PasswordHash::new(&stored_hash)
            .map_err(|e| AuthError::Failed(format!("Invalid stored hash: {}", e)))?;

        match Argon2::default().verify_password(pin.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

impl Default for PinAuth {
    fn default() -> Self {
        Self::new()
    }
}

impl Authenticator for PinAuth {
    fn is_available(&self) -> bool {
        SecureKeyring::has_pin()
    }

    fn authenticate(&self, command: &str) -> AuthResult {
        eprintln!("{} {}", "Command:".yellow(), command);
        eprintln!("{}", "PIN required for this operation".red());

        let entered = Password::new()
            .with_prompt("Enter PIN")
            .interact()
            .map_err(|_| AuthError::Cancelled)?;

        if self.verify_pin(&entered)? {
            Ok(true)
        } else {
            Err(AuthError::Failed("Invalid PIN".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Integration tests for PIN verification require keychain access
    // and are marked as ignored for CI

    #[test]
    #[ignore = "requires system keychain"]
    fn test_set_and_verify_pin() {
        // Set PIN
        PinAuth::set_pin("1234").expect("Failed to set PIN");

        // Verify correct PIN
        let auth = PinAuth::new();
        assert!(auth.is_available());
        assert!(auth.verify_pin("1234").expect("Verification failed"));

        // Verify wrong PIN
        assert!(!auth.verify_pin("0000").expect("Verification failed"));

        // Cleanup
        PinAuth::delete_pin().expect("Failed to delete PIN");
    }
}
