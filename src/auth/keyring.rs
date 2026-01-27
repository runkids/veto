//! System Keychain access wrapper with file-based fallback
//!
//! Stores secrets securely using the system keychain:
//! - macOS: Keychain
//! - Linux: Secret Service (GNOME Keyring, KWallet)
//! - Windows: Credential Manager
//!
//! When system keychain is unavailable (e.g., Docker containers),
//! falls back to encrypted file storage using AES-GCM.

use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use keyring::Entry;
use pbkdf2::pbkdf2_hmac_array;
use rand::RngCore;
use sha2::Sha256;
use thiserror::Error;

/// Service name for all veto secrets
const SERVICE_NAME: &str = "veto";

/// Salt for key derivation (hardcoded, combined with machine-id for security)
const KEY_DERIVATION_SALT: &[u8] = b"veto-keyring-fallback-v1";

/// PBKDF2 iterations
const PBKDF2_ITERATIONS: u32 = 100_000;

/// Nonce size for AES-GCM
const NONCE_SIZE: usize = 12;

/// Keychain key names
pub mod keys {
    pub const PIN_HASH: &str = "veto.pin.hash";
    pub const PIN_SALT: &str = "veto.pin.salt";
    pub const TOTP_SECRET: &str = "veto.totp.secret";
    pub const TELEGRAM_TOKEN: &str = "veto.telegram.token";
}

#[derive(Error, Debug)]
pub enum KeyringError {
    #[error("Secret not found: {0}")]
    NotFound(String),
    #[error("Keyring access error: {0}")]
    AccessError(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("IO error: {0}")]
    IoError(String),
}

pub type KeyringResult<T> = Result<T, KeyringError>;

/// Cached backend type detection
static BACKEND: OnceLock<KeyringBackend> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq)]
enum KeyringBackend {
    System,
    File,
}

/// Detect which backend to use (cached)
fn detect_backend() -> KeyringBackend {
    *BACKEND.get_or_init(|| {
        // Try to create an entry and set/get a test value
        let test_key = "veto.backend.test";
        let test_value = "test";

        let entry = match Entry::new(SERVICE_NAME, test_key) {
            Ok(e) => e,
            Err(_) => return KeyringBackend::File,
        };

        // Try to set
        if entry.set_password(test_value).is_err() {
            return KeyringBackend::File;
        }

        // Try to get
        let result = entry.get_password();

        // Clean up
        let _ = entry.delete_credential();

        match result {
            Ok(v) if v == test_value => KeyringBackend::System,
            _ => KeyringBackend::File,
        }
    })
}

/// File-based encrypted keyring for fallback storage
struct FileKeyring;

impl FileKeyring {
    /// Get the secrets directory path
    fn secrets_dir() -> KeyringResult<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            KeyringError::IoError("Cannot determine home directory".to_string())
        })?;
        Ok(home.join(".veto").join("secrets"))
    }

    /// Get path for a specific key
    fn key_path(key: &str) -> KeyringResult<PathBuf> {
        let dir = Self::secrets_dir()?;
        // Sanitize key name for filesystem
        let filename = key.replace('.', "_") + ".enc";
        Ok(dir.join(filename))
    }

    /// Get machine-id for key derivation
    fn get_machine_id() -> String {
        // Try /etc/machine-id (Linux)
        if let Ok(id) = fs::read_to_string("/etc/machine-id") {
            let id = id.trim();
            if !id.is_empty() {
                return id.to_string();
            }
        }

        // Try /var/lib/dbus/machine-id (older Linux)
        if let Ok(id) = fs::read_to_string("/var/lib/dbus/machine-id") {
            let id = id.trim();
            if !id.is_empty() {
                return id.to_string();
            }
        }

        // macOS: use hostname + serial from IOPlatformSerialNumber
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = std::process::Command::new("ioreg")
                .args(["-rd1", "-c", "IOPlatformExpertDevice"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = output_str.lines().find(|l| l.contains("IOPlatformSerialNumber")) {
                    if let Some(serial) = line.split('"').nth(3) {
                        return serial.to_string();
                    }
                }
            }
        }

        // Fallback to hostname
        hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "veto-default-machine".to_string())
    }

    /// Derive encryption key from machine-id
    fn derive_key() -> [u8; 32] {
        let machine_id = Self::get_machine_id();
        let password = format!("{}-{}", machine_id, SERVICE_NAME);

        pbkdf2_hmac_array::<Sha256, 32>(
            password.as_bytes(),
            KEY_DERIVATION_SALT,
            PBKDF2_ITERATIONS,
        )
    }

    /// Encrypt data
    fn encrypt(plaintext: &[u8]) -> KeyringResult<Vec<u8>> {
        let key = Self::derive_key();
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| KeyringError::EncryptionError(e.to_string()))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| KeyringError::EncryptionError(e.to_string()))?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }

    /// Decrypt data
    fn decrypt(data: &[u8]) -> KeyringResult<Vec<u8>> {
        if data.len() < NONCE_SIZE {
            return Err(KeyringError::EncryptionError(
                "Data too short".to_string(),
            ));
        }

        let key = Self::derive_key();
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| KeyringError::EncryptionError(e.to_string()))?;

        let nonce = Nonce::from_slice(&data[..NONCE_SIZE]);
        let ciphertext = &data[NONCE_SIZE..];

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| KeyringError::EncryptionError(format!("Decryption failed: {}", e)))
    }

    /// Get a secret
    fn get(key: &str) -> KeyringResult<String> {
        let path = Self::key_path(key)?;

        let data = fs::read(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                KeyringError::NotFound(key.to_string())
            } else {
                KeyringError::IoError(e.to_string())
            }
        })?;

        let plaintext = Self::decrypt(&data)?;
        String::from_utf8(plaintext)
            .map_err(|e| KeyringError::AccessError(format!("Invalid UTF-8: {}", e)))
    }

    /// Set a secret
    fn set(key: &str, value: &str) -> KeyringResult<()> {
        let dir = Self::secrets_dir()?;
        fs::create_dir_all(&dir).map_err(|e| KeyringError::IoError(e.to_string()))?;

        // Set restrictive permissions on the directory
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o700);
            let _ = fs::set_permissions(&dir, perms);
        }

        let path = Self::key_path(key)?;
        let encrypted = Self::encrypt(value.as_bytes())?;

        fs::write(&path, &encrypted).map_err(|e| KeyringError::IoError(e.to_string()))?;

        // Set restrictive permissions on the file
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            let _ = fs::set_permissions(&path, perms);
        }

        Ok(())
    }

    /// Delete a secret
    fn delete(key: &str) -> KeyringResult<()> {
        let path = Self::key_path(key)?;

        fs::remove_file(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                KeyringError::NotFound(key.to_string())
            } else {
                KeyringError::IoError(e.to_string())
            }
        })
    }

    /// Check if a secret exists
    fn exists(key: &str) -> bool {
        Self::key_path(key)
            .map(|p| p.exists())
            .unwrap_or(false)
    }
}

/// System keyring wrapper
struct SystemKeyring;

impl SystemKeyring {
    fn get(key: &str) -> KeyringResult<String> {
        let entry = Entry::new(SERVICE_NAME, key)
            .map_err(|e| KeyringError::AccessError(e.to_string()))?;

        entry.get_password().map_err(|e| match e {
            keyring::Error::NoEntry => KeyringError::NotFound(key.to_string()),
            _ => KeyringError::AccessError(e.to_string()),
        })
    }

    fn set(key: &str, value: &str) -> KeyringResult<()> {
        let entry = Entry::new(SERVICE_NAME, key)
            .map_err(|e| KeyringError::AccessError(e.to_string()))?;

        entry
            .set_password(value)
            .map_err(|e| KeyringError::AccessError(e.to_string()))?;

        // Verify the write actually succeeded (macOS Keychain can silently fail)
        match Self::get(key) {
            Ok(v) if v == value => Ok(()),
            Ok(_) => Err(KeyringError::AccessError(
                "Keychain write verification failed: value mismatch".to_string(),
            )),
            Err(_) => Err(KeyringError::AccessError(
                "Keychain write verification failed: cannot read back".to_string(),
            )),
        }
    }

    fn delete(key: &str) -> KeyringResult<()> {
        let entry = Entry::new(SERVICE_NAME, key)
            .map_err(|e| KeyringError::AccessError(e.to_string()))?;

        entry.delete_credential().map_err(|e| match e {
            keyring::Error::NoEntry => KeyringError::NotFound(key.to_string()),
            _ => KeyringError::AccessError(e.to_string()),
        })
    }

    fn exists(key: &str) -> bool {
        Self::get(key).is_ok()
    }
}

/// Keyring wrapper for secure secret storage
/// Automatically uses system keyring or file-based fallback
pub struct SecureKeyring;

impl SecureKeyring {
    /// Get a secret from the keychain
    pub fn get(key: &str) -> KeyringResult<String> {
        match detect_backend() {
            KeyringBackend::System => {
                // Try system first, fall back to file if it fails
                SystemKeyring::get(key).or_else(|_| FileKeyring::get(key))
            }
            KeyringBackend::File => FileKeyring::get(key),
        }
    }

    /// Set a secret in the keychain
    pub fn set(key: &str, value: &str) -> KeyringResult<()> {
        match detect_backend() {
            KeyringBackend::System => {
                // Try system first, fall back to file if it fails
                match SystemKeyring::set(key, value) {
                    Ok(()) => Ok(()),
                    Err(_) => FileKeyring::set(key, value),
                }
            }
            KeyringBackend::File => FileKeyring::set(key, value),
        }
    }

    /// Delete a secret from the keychain
    pub fn delete(key: &str) -> KeyringResult<()> {
        match detect_backend() {
            KeyringBackend::System => {
                // Try to delete from both backends
                let sys_result = SystemKeyring::delete(key);
                let file_result = FileKeyring::delete(key);
                // Return Ok if either succeeded
                if sys_result.is_ok() || file_result.is_ok() {
                    Ok(())
                } else {
                    // Return the file error (more likely to be meaningful)
                    file_result
                }
            }
            KeyringBackend::File => FileKeyring::delete(key),
        }
    }

    /// Check if a secret exists
    pub fn exists(key: &str) -> bool {
        match detect_backend() {
            KeyringBackend::System => {
                SystemKeyring::exists(key) || FileKeyring::exists(key)
            }
            KeyringBackend::File => FileKeyring::exists(key),
        }
    }

    /// Get the current backend type (for diagnostics)
    pub fn backend_name() -> &'static str {
        match detect_backend() {
            KeyringBackend::System => "system",
            KeyringBackend::File => "file",
        }
    }

    // === PIN specific helpers ===

    /// Get stored PIN hash
    pub fn get_pin_hash() -> KeyringResult<String> {
        Self::get(keys::PIN_HASH)
    }


    /// Store PIN hash and salt
    pub fn set_pin(hash: &str, salt: &str) -> KeyringResult<()> {
        Self::set(keys::PIN_HASH, hash)?;
        Self::set(keys::PIN_SALT, salt)
    }

    /// Check if PIN is configured
    pub fn has_pin() -> bool {
        Self::exists(keys::PIN_HASH) && Self::exists(keys::PIN_SALT)
    }

    /// Delete PIN
    pub fn delete_pin() -> KeyringResult<()> {
        // Ignore NotFound errors
        let _ = Self::delete(keys::PIN_HASH);
        let _ = Self::delete(keys::PIN_SALT);
        Ok(())
    }

    // === TOTP specific helpers ===

    /// Get stored TOTP secret
    pub fn get_totp_secret() -> KeyringResult<String> {
        Self::get(keys::TOTP_SECRET)
    }

    /// Store TOTP secret
    pub fn set_totp_secret(secret: &str) -> KeyringResult<()> {
        Self::set(keys::TOTP_SECRET, secret)
    }

    /// Check if TOTP is configured
    pub fn has_totp() -> bool {
        Self::exists(keys::TOTP_SECRET)
    }

    /// Delete TOTP secret
    pub fn delete_totp() -> KeyringResult<()> {
        Self::delete(keys::TOTP_SECRET)
    }

    // === Telegram specific helpers ===

    /// Get stored Telegram bot token
    pub fn get_telegram_token() -> KeyringResult<String> {
        Self::get(keys::TELEGRAM_TOKEN)
    }

    /// Store Telegram bot token
    pub fn set_telegram_token(token: &str) -> KeyringResult<()> {
        Self::set(keys::TELEGRAM_TOKEN, token)
    }

    /// Check if Telegram is configured
    pub fn has_telegram() -> bool {
        Self::exists(keys::TELEGRAM_TOKEN)
    }

    /// Delete Telegram token
    pub fn delete_telegram() -> KeyringResult<()> {
        Self::delete(keys::TELEGRAM_TOKEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a working keychain
    // They are marked as ignored by default to avoid CI issues

    #[test]
    #[ignore = "requires system keychain"]
    fn test_set_get_delete() {
        let test_key = "veto.test.key";
        let test_value = "test_secret_value";

        // Set
        SecureKeyring::set(test_key, test_value).expect("Failed to set");

        // Get
        let retrieved = SecureKeyring::get(test_key).expect("Failed to get");
        assert_eq!(retrieved, test_value);

        // Exists
        assert!(SecureKeyring::exists(test_key));

        // Delete
        SecureKeyring::delete(test_key).expect("Failed to delete");
        assert!(!SecureKeyring::exists(test_key));
    }

    #[test]
    fn test_file_keyring_encrypt_decrypt() {
        let plaintext = b"test secret data";
        let encrypted = FileKeyring::encrypt(plaintext).expect("Encrypt failed");
        let decrypted = FileKeyring::decrypt(&encrypted).expect("Decrypt failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_file_keyring_different_nonce() {
        let plaintext = b"test data";
        let encrypted1 = FileKeyring::encrypt(plaintext).expect("Encrypt failed");
        let encrypted2 = FileKeyring::encrypt(plaintext).expect("Encrypt failed");
        // Same plaintext should produce different ciphertext (random nonce)
        assert_ne!(encrypted1, encrypted2);
    }
}
