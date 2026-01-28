//! Challenge-Response authentication system
//!
//! Generates one-time challenge codes that are sent via notifications
//! to prevent AI agents from reusing authentication credentials.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::keyring::SecureKeyring;
use super::{AuthError, PinAuth};

/// Challenge expiration time in seconds
const CHALLENGE_EXPIRY_SECONDS: u64 = 60;

/// Challenge code length (4 digits)
const CHALLENGE_LENGTH: usize = 4;

/// Challenge data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    /// 4-digit challenge code
    pub id: String,
    /// Hash of the command this challenge is bound to
    pub command_hash: String,
    /// Unix timestamp when challenge was created
    pub created_at: u64,
    /// Whether this challenge has been used
    pub used: bool,
}

impl Challenge {
    /// Get the challenges directory path
    fn challenges_dir() -> Result<PathBuf, AuthError> {
        let home = dirs::home_dir()
            .ok_or_else(|| AuthError::Failed("Cannot determine home directory".to_string()))?;
        Ok(home.join(".veto").join("challenges"))
    }

    /// Get the path for a specific challenge file
    fn challenge_path(id: &str) -> Result<PathBuf, AuthError> {
        let dir = Self::challenges_dir()?;
        Ok(dir.join(format!("{}.json", id)))
    }

    /// Hash a command for binding
    fn hash_command(command: &str) -> String {
        let mut hasher = DefaultHasher::new();
        command.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Get current Unix timestamp
    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Generate a new challenge for a command
    pub fn generate(command: &str) -> Result<Challenge, AuthError> {
        // Generate random 4-digit code
        let mut rng = rand::thread_rng();
        let code: u32 = rng.gen_range(1000..10000);
        let id = format!("{:04}", code);

        let challenge = Challenge {
            id: id.clone(),
            command_hash: Self::hash_command(command),
            created_at: Self::now(),
            used: false,
        };

        // Ensure challenges directory exists
        let dir = Self::challenges_dir()?;
        fs::create_dir_all(&dir)
            .map_err(|e| AuthError::Failed(format!("Failed to create challenges dir: {}", e)))?;

        // Set restrictive permissions on directory
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o700);
            let _ = fs::set_permissions(&dir, perms);
        }

        // Save challenge to file
        let path = Self::challenge_path(&id)?;
        let json = serde_json::to_string_pretty(&challenge)
            .map_err(|e| AuthError::Failed(format!("Failed to serialize challenge: {}", e)))?;

        fs::write(&path, json)
            .map_err(|e| AuthError::Failed(format!("Failed to save challenge: {}", e)))?;

        Ok(challenge)
    }

    /// Load a challenge by ID
    pub fn load(id: &str) -> Result<Challenge, AuthError> {
        let path = Self::challenge_path(id)?;
        let json = fs::read_to_string(&path)
            .map_err(|_| AuthError::Failed(format!("Challenge {} not found", id)))?;

        serde_json::from_str(&json)
            .map_err(|e| AuthError::Failed(format!("Failed to parse challenge: {}", e)))
    }

    /// Mark a challenge as used and save
    pub fn mark_used(&mut self) -> Result<(), AuthError> {
        self.used = true;
        let path = Self::challenge_path(&self.id)?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| AuthError::Failed(format!("Failed to serialize challenge: {}", e)))?;

        fs::write(&path, json)
            .map_err(|e| AuthError::Failed(format!("Failed to save challenge: {}", e)))
    }

    /// Delete this challenge file
    pub fn delete(&self) -> Result<(), AuthError> {
        let path = Self::challenge_path(&self.id)?;
        fs::remove_file(&path)
            .map_err(|e| AuthError::Failed(format!("Failed to delete challenge: {}", e)))
    }

    /// Check if this challenge has expired
    pub fn is_expired(&self) -> bool {
        let now = Self::now();
        now.saturating_sub(self.created_at) > CHALLENGE_EXPIRY_SECONDS
    }

    /// Check if this challenge is valid for the given command
    pub fn is_valid_for_command(&self, command: &str) -> bool {
        self.command_hash == Self::hash_command(command)
    }

    /// Clean up expired challenges
    pub fn cleanup_expired() -> Result<usize, AuthError> {
        let dir = Self::challenges_dir()?;
        if !dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".json") {
                        let id = name.trim_end_matches(".json");
                        if let Ok(challenge) = Self::load(id) {
                            if challenge.is_expired() || challenge.used {
                                let _ = challenge.delete();
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
        Ok(count)
    }
}

/// Send challenge notification via available channels
/// Send challenge notification via available channels
/// At least one notification method must succeed
pub fn notify_challenge(challenge: &Challenge, command: &str) -> Result<(), AuthError> {
    let mut desktop_ok = false;
    let mut telegram_ok = false;

    // Try desktop notification (macOS/Linux)
    if let Ok(()) = send_desktop_notification(challenge) {
        desktop_ok = true;
    }

    // Try Telegram notification if configured
    if SecureKeyring::has_telegram() {
        if send_telegram_notification(challenge, command).is_ok() {
            telegram_ok = true;
        }
    }

    // At least one notification method must succeed
    if desktop_ok || telegram_ok {
        Ok(())
    } else {
        Err(AuthError::Failed(
            "No notification method available. Install notify-send (Linux) or configure Telegram.".to_string()
        ))
    }
}

/// Send desktop notification (macOS: osascript, Linux: notify-send)
fn send_desktop_notification(challenge: &Challenge) -> Result<(), AuthError> {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"display notification "Challenge: {}" with title "veto" sound name "Glass""#,
            challenge.id
        );

        std::process::Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| AuthError::Failed(format!("Failed to send macOS notification: {}", e)))?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        // Try notify-send (libnotify) for Linux desktop notifications
        let result = std::process::Command::new("notify-send")
            .args([
                "--urgency=critical",
                "--app-name=veto",
                "veto Challenge",
                &format!("Challenge code: {}", challenge.id),
            ])
            .output();

        match result {
            Ok(output) if output.status.success() => Ok(()),
            _ => {
                // Fallback: notify-send not available, don't print to stderr (AI can see it)
                // User must use Telegram for secure challenge on Linux without notify-send
                Err(AuthError::Failed(
                    "notify-send not available. Configure Telegram for challenge notifications.".to_string()
                ))
            }
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        // Other platforms: require Telegram
        Err(AuthError::Failed(
            "Desktop notifications not supported. Configure Telegram for challenge notifications.".to_string()
        ))
    }
}

/// Send Telegram notification (async wrapper)
fn send_telegram_notification(challenge: &Challenge, command: &str) -> Result<(), AuthError> {
    use crate::config::loader::load_config;

    let config = load_config().map_err(|e| AuthError::Failed(e.to_string()))?;

    let chat_id = config
        .auth
        .as_ref()
        .and_then(|a| a.telegram.as_ref())
        .and_then(|t| t.chat_id.as_ref())
        .ok_or_else(|| AuthError::NotAvailable("Telegram chat_id not configured".to_string()))?;

    let token = SecureKeyring::get_telegram_token()
        .map_err(|_| AuthError::NotAvailable("Telegram token not found".to_string()))?;

    // Use blocking reqwest since we're in sync context
    let client = reqwest::blocking::Client::new();
    let message = format!(
        "🔐 <b>Veto Challenge Code</b>\n\n\
         <b>Code:</b> <code>{}</code>\n\n\
         <b>Command:</b>\n<code>{}</code>\n\n\
         <i>Expires in 60 seconds</i>",
        challenge.id,
        html_escape(command)
    );

    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let params = [
        ("chat_id", chat_id.as_str()),
        ("text", &message),
        ("parse_mode", "HTML"),
    ];

    client
        .post(&url)
        .form(&params)
        .send()
        .map_err(|e| AuthError::Failed(format!("Telegram send failed: {}", e)))?;

    Ok(())
}

/// Escape HTML special characters for Telegram
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Response verification result
pub struct ChallengeVerifyResult {
    /// Whether verification succeeded
    pub success: bool,
    /// Authentication method used
    pub method: String,
    /// Error message if failed
    pub error: Option<String>,
}

/// Verify a response that contains challenge code
///
/// For PIN auth: response = PIN + challenge (e.g., "12344827" = PIN "1234" + challenge "4827")
/// For confirm auth: response = challenge only (e.g., "4827")
pub fn verify_response(
    response: &str,
    command: &str,
    auth_method: &str,
) -> ChallengeVerifyResult {
    // Clean up expired challenges first
    let _ = Challenge::cleanup_expired();

    match auth_method {
        "pin" => verify_pin_with_challenge(response, command),
        "confirm" => verify_confirm_with_challenge(response, command),
        _ => ChallengeVerifyResult {
            success: false,
            method: auth_method.to_string(),
            error: Some(format!("Unsupported auth method for challenge: {}", auth_method)),
        },
    }
}

/// Verify PIN + challenge response
/// Format: {PIN}{4-digit challenge} e.g., "12344827"
fn verify_pin_with_challenge(response: &str, command: &str) -> ChallengeVerifyResult {
    if response.len() < CHALLENGE_LENGTH + 1 {
        return ChallengeVerifyResult {
            success: false,
            method: "PIN".to_string(),
            error: Some("Response too short (need PIN + 4-digit challenge)".to_string()),
        };
    }

    // Extract challenge from last 4 characters
    let challenge_start = response.len() - CHALLENGE_LENGTH;
    let challenge_id = &response[challenge_start..];
    let pin = &response[..challenge_start];

    // Verify challenge
    match Challenge::load(challenge_id) {
        Ok(mut challenge) => {
            if challenge.used {
                return ChallengeVerifyResult {
                    success: false,
                    method: "PIN".to_string(),
                    error: Some("Challenge already used".to_string()),
                };
            }

            if challenge.is_expired() {
                let _ = challenge.delete();
                return ChallengeVerifyResult {
                    success: false,
                    method: "PIN".to_string(),
                    error: Some("Challenge expired".to_string()),
                };
            }

            if !challenge.is_valid_for_command(command) {
                return ChallengeVerifyResult {
                    success: false,
                    method: "PIN".to_string(),
                    error: Some("Challenge not valid for this command".to_string()),
                };
            }

            // Verify PIN
            let pin_auth = PinAuth::new();
            match pin_auth.verify_direct(pin) {
                Ok(true) => {
                    // Mark challenge as used
                    let _ = challenge.mark_used();
                    ChallengeVerifyResult {
                        success: true,
                        method: "PIN+challenge".to_string(),
                        error: None,
                    }
                }
                Ok(false) => ChallengeVerifyResult {
                    success: false,
                    method: "PIN".to_string(),
                    error: Some("Invalid PIN".to_string()),
                },
                Err(e) => ChallengeVerifyResult {
                    success: false,
                    method: "PIN".to_string(),
                    error: Some(format!("PIN verification error: {}", e)),
                },
            }
        }
        Err(_) => ChallengeVerifyResult {
            success: false,
            method: "PIN".to_string(),
            error: Some("Invalid challenge code".to_string()),
        },
    }
}

/// Verify confirm with challenge response
/// Format: just the 4-digit challenge e.g., "4827"
fn verify_confirm_with_challenge(response: &str, command: &str) -> ChallengeVerifyResult {
    let challenge_id = response.trim();

    if challenge_id.len() != CHALLENGE_LENGTH {
        return ChallengeVerifyResult {
            success: false,
            method: "confirm".to_string(),
            error: Some(format!("Challenge must be {} digits", CHALLENGE_LENGTH)),
        };
    }

    match Challenge::load(challenge_id) {
        Ok(mut challenge) => {
            if challenge.used {
                return ChallengeVerifyResult {
                    success: false,
                    method: "confirm".to_string(),
                    error: Some("Challenge already used".to_string()),
                };
            }

            if challenge.is_expired() {
                let _ = challenge.delete();
                return ChallengeVerifyResult {
                    success: false,
                    method: "confirm".to_string(),
                    error: Some("Challenge expired".to_string()),
                };
            }

            if !challenge.is_valid_for_command(command) {
                return ChallengeVerifyResult {
                    success: false,
                    method: "confirm".to_string(),
                    error: Some("Challenge not valid for this command".to_string()),
                };
            }

            // Mark challenge as used
            let _ = challenge.mark_used();
            ChallengeVerifyResult {
                success: true,
                method: "challenge".to_string(),
                error: None,
            }
        }
        Err(_) => ChallengeVerifyResult {
            success: false,
            method: "confirm".to_string(),
            error: Some("Invalid challenge code".to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge_generation() {
        let challenge = Challenge::generate("test command").expect("Failed to generate challenge");

        assert_eq!(challenge.id.len(), 4);
        assert!(!challenge.used);
        assert!(!challenge.is_expired());

        // Cleanup
        let _ = challenge.delete();
    }

    #[test]
    fn test_challenge_command_binding() {
        let challenge = Challenge::generate("rm -rf /").expect("Failed to generate challenge");

        assert!(challenge.is_valid_for_command("rm -rf /"));
        assert!(!challenge.is_valid_for_command("rm -rf ~"));

        // Cleanup
        let _ = challenge.delete();
    }

    #[test]
    fn test_challenge_single_use() {
        let mut challenge = Challenge::generate("test").expect("Failed to generate challenge");
        let id = challenge.id.clone();

        // Mark as used
        challenge.mark_used().expect("Failed to mark used");

        // Load and verify it's marked as used
        let loaded = Challenge::load(&id).expect("Failed to load challenge");
        assert!(loaded.used);

        // Cleanup
        let _ = challenge.delete();
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("a < b & c > d"), "a &lt; b &amp; c &gt; d");
    }

    #[test]
    fn test_verify_confirm_with_challenge() {
        let challenge = Challenge::generate("echo test").expect("Failed to generate challenge");
        let id = challenge.id.clone();

        // Verify with correct challenge
        let result = verify_confirm_with_challenge(&id, "echo test");
        assert!(result.success);

        // Try to use again - should fail
        let result2 = verify_confirm_with_challenge(&id, "echo test");
        assert!(!result2.success);
        assert!(result2.error.unwrap().contains("already used"));
    }

    #[test]
    fn test_verify_confirm_wrong_command() {
        let challenge = Challenge::generate("echo test").expect("Failed to generate challenge");
        let id = challenge.id.clone();

        // Verify with wrong command
        let result = verify_confirm_with_challenge(&id, "echo other");
        assert!(!result.success);
        assert!(result.error.unwrap().contains("not valid for this command"));

        // Cleanup
        let _ = challenge.delete();
    }
}
