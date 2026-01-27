//! TOTP (Time-based One-Time Password) authentication
//!
//! Compatible with Google Authenticator, Authy, and other TOTP apps.
//! Uses RFC 6238 standard with SHA1, 6 digits, 30 second period.

use colored::Colorize;
use dialoguer::Input;
use totp_rs::{Algorithm, Secret, TOTP};

use super::keyring::SecureKeyring;
use super::{AuthError, AuthResult, Authenticator};

/// Default issuer name for TOTP
const DEFAULT_ISSUER: &str = "veto";

/// TOTP authenticator
pub struct TotpAuth;

impl TotpAuth {
    pub fn new() -> Self {
        Self
    }

    /// Generate a new TOTP secret and store it
    /// Returns the secret and otpauth URL for QR code generation
    pub fn setup(account: &str, issuer: Option<&str>) -> Result<SetupResult, AuthError> {
        let secret = Secret::generate_secret();
        let secret_base32 = secret.to_encoded().to_string();

        let issuer_str = issuer.unwrap_or(DEFAULT_ISSUER);

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes().map_err(|e| AuthError::Failed(e.to_string()))?,
            Some(issuer_str.to_string()),
            account.to_string(),
        )
        .map_err(|e| AuthError::Failed(format!("Failed to create TOTP: {}", e)))?;

        let otpauth_url = totp.get_url();

        // Store secret in keychain
        SecureKeyring::set_totp_secret(&secret_base32)
            .map_err(|e| AuthError::Failed(format!("Failed to store TOTP secret: {}", e)))?;

        Ok(SetupResult {
            secret: secret_base32,
            otpauth_url,
        })
    }

    /// Delete stored TOTP secret
    pub fn delete() -> Result<(), AuthError> {
        SecureKeyring::delete_totp()
            .map_err(|e| AuthError::Failed(format!("Failed to delete TOTP: {}", e)))
    }

    /// Verify a TOTP code (static method for setup verification)
    /// Does not prompt for input - use this when you already have the code
    pub fn verify(code: &str) -> Result<bool, AuthError> {
        let totp = Self::create_totp()?;
        Ok(totp.check_current(code).unwrap_or(false))
    }

    /// Create TOTP instance from stored secret
    fn create_totp() -> Result<TOTP, AuthError> {
        let secret_base32 = SecureKeyring::get_totp_secret()
            .map_err(|_| AuthError::NotAvailable("TOTP not configured".to_string()))?;

        let secret = Secret::Encoded(secret_base32);
        let secret_bytes = secret
            .to_bytes()
            .map_err(|e| AuthError::Failed(format!("Invalid TOTP secret: {}", e)))?;

        TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            Some(DEFAULT_ISSUER.to_string()),
            "veto".to_string(),
        )
        .map_err(|e| AuthError::Failed(format!("Failed to create TOTP: {}", e)))
    }

    /// Verify a TOTP code
    fn verify_code(&self, code: &str) -> Result<bool, AuthError> {
        let totp = Self::create_totp()?;

        // Check current and adjacent time windows for clock skew tolerance
        Ok(totp.check_current(code).unwrap_or(false))
    }
}

impl Default for TotpAuth {
    fn default() -> Self {
        Self::new()
    }
}

impl Authenticator for TotpAuth {
    fn is_available(&self) -> bool {
        SecureKeyring::has_totp()
    }

    fn authenticate(&self, _command: &str) -> AuthResult {
        println!("{}", "TOTP verification required".red());

        let code: String = Input::new()
            .with_prompt("Enter 6-digit code")
            .validate_with(|input: &String| {
                if input.len() == 6 && input.chars().all(|c| c.is_ascii_digit()) {
                    Ok(())
                } else {
                    Err("Code must be exactly 6 digits")
                }
            })
            .interact_text()
            .map_err(|_| AuthError::Cancelled)?;

        match self.verify_code(&code) {
            Ok(true) => {
                println!("{}", "✓ TOTP verified".green());
                Ok(true)
            }
            Ok(false) => {
                Err(AuthError::Failed("Invalid TOTP code".to_string()))
            }
            Err(e) => {
                Err(e)
            }
        }
    }
}

/// Result from TOTP setup
pub struct SetupResult {
    /// Base32 encoded secret (for manual entry)
    pub secret: String,
    /// otpauth:// URL for QR code generation
    pub otpauth_url: String,
}

impl SetupResult {
    /// Generate QR code as a string (terminal display)
    pub fn generate_qr_terminal(&self) -> Result<String, AuthError> {
        use qrcode::QrCode;
        use qrcode::render::unicode;

        let code = QrCode::new(&self.otpauth_url)
            .map_err(|e| AuthError::Failed(format!("Failed to generate QR code: {}", e)))?;

        Ok(code.render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires system keychain"]
    fn test_setup_and_verify() {
        // Setup
        let result = TotpAuth::setup("test@example.com", None)
            .expect("Failed to setup TOTP");

        assert!(!result.secret.is_empty());
        assert!(result.otpauth_url.starts_with("otpauth://totp/"));

        // Verify using static method
        let auth = TotpAuth::new();
        assert!(auth.is_available());

        // Cleanup
        TotpAuth::delete().expect("Failed to delete TOTP");
    }
}
