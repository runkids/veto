//! Telegram bot authentication
//!
//! Sends a message to a Telegram chat and waits for /allow or /deny response.
//! This is an async authenticator that uses the AsyncAuthenticator trait.

use std::time::Duration;

use async_trait::async_trait;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use super::keyring::SecureKeyring;
use super::manager::AsyncAuthenticator;
use super::{AuthError, AuthResult};

/// Default timeout for waiting for Telegram response
const DEFAULT_TIMEOUT_SECONDS: u64 = 60;

/// Telegram bot API base URL
const TELEGRAM_API_BASE: &str = "https://api.telegram.org/bot";

/// Telegram authenticator
pub struct TelegramAuth {
    chat_id: String,
    timeout: Duration,
}

impl TelegramAuth {
    pub fn new(chat_id: &str) -> Self {
        Self {
            chat_id: chat_id.to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECONDS),
        }
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout = Duration::from_secs(seconds);
        self
    }

    /// Setup Telegram authentication
    pub fn setup(bot_token: &str) -> Result<(), AuthError> {
        SecureKeyring::set_telegram_token(bot_token)
            .map_err(|e| AuthError::Failed(format!("Failed to store Telegram token: {}", e)))
    }

    /// Delete Telegram configuration
    pub fn delete() -> Result<(), AuthError> {
        SecureKeyring::delete_telegram()
            .map_err(|e| AuthError::Failed(format!("Failed to delete Telegram config: {}", e)))
    }

    /// Get bot token from keychain
    fn get_token() -> Result<String, AuthError> {
        SecureKeyring::get_telegram_token()
            .map_err(|_| AuthError::NotAvailable("Telegram not configured".to_string()))
    }

    /// Send a message to the chat
    async fn send_message(&self, token: &str, text: &str) -> Result<i64, AuthError> {
        let url = format!("{}{}/sendMessage", TELEGRAM_API_BASE, token);

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&SendMessageRequest {
                chat_id: &self.chat_id,
                text,
                parse_mode: Some("HTML"),
            })
            .send()
            .await
            .map_err(|e| AuthError::Failed(format!("Failed to send message: {}", e)))?;

        let result: TelegramResponse<Message> = response
            .json()
            .await
            .map_err(|e| AuthError::Failed(format!("Failed to parse response: {}", e)))?;

        if !result.ok {
            return Err(AuthError::Failed(
                result.description.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        Ok(result.result.unwrap().message_id)
    }

    /// Poll for updates and check for /allow or /deny response
    /// Only processes messages received AFTER request_time
    async fn wait_for_response(&self, token: &str, request_time: i64) -> Result<bool, AuthError> {
        let url = format!("{}{}/getUpdates", TELEGRAM_API_BASE, token);
        let client = reqwest::Client::new();

        let start = std::time::Instant::now();
        let mut last_update_id: Option<i64> = None;

        // First, clear any old updates by getting the latest update_id
        let init_response = client
            .post(&url)
            .json(&serde_json::json!({ "limit": 1, "offset": -1 }))
            .send()
            .await
            .ok();

        if let Some(resp) = init_response {
            if let Ok(result) = resp.json::<TelegramResponse<Vec<Update>>>().await {
                if let Some(updates) = result.result {
                    if let Some(last) = updates.last() {
                        last_update_id = Some(last.update_id);
                    }
                }
            }
        }

        while start.elapsed() < self.timeout {
            let request = GetUpdatesRequest {
                timeout: 10,
                offset: last_update_id.map(|id| id + 1),
            };

            let response = client
                .post(&url)
                .json(&request)
                .timeout(Duration::from_secs(15))
                .send()
                .await
                .map_err(|e| AuthError::Failed(format!("Failed to get updates: {}", e)))?;

            let result: TelegramResponse<Vec<Update>> = response
                .json()
                .await
                .map_err(|e| AuthError::Failed(format!("Failed to parse updates: {}", e)))?;

            if let Some(updates) = result.result {
                for update in updates {
                    last_update_id = Some(update.update_id);

                    if let Some(msg) = update.message {
                        // Only process messages from the correct chat
                        // and received after our request was sent
                        if msg.chat.id.to_string() == self.chat_id {
                            // Check message timestamp (msg.date is unix timestamp)
                            let msg_time = msg.date.unwrap_or(0);
                            if msg_time >= request_time {
                                if let Some(text) = msg.text {
                                    let text_lower = text.to_lowercase();
                                    if text_lower.starts_with("/allow") || text_lower == "allow" || text_lower == "yes" {
                                        return Ok(true);
                                    }
                                    if text_lower.starts_with("/deny") || text_lower == "deny" || text_lower == "no" {
                                        return Ok(false);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Small delay before next poll
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Err(AuthError::Timeout)
    }
}

#[async_trait]
impl AsyncAuthenticator for TelegramAuth {
    fn is_available(&self) -> bool {
        SecureKeyring::has_telegram() && !self.chat_id.is_empty()
    }

    async fn authenticate_async(&self, command: &str) -> AuthResult {
        eprintln!("{} {}", "Command:".yellow(), command);
        eprintln!("{}", "Waiting for Telegram approval...".cyan());

        let token = Self::get_token()?;

        // Record current time before sending (for filtering old messages)
        let request_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Send approval request
        let message = format!(
            "🔐 <b>Veto Authorization Request</b>\n\n\
             <b>Command:</b>\n<code>{}</code>\n\n\
             Reply with /allow or /deny",
            html_escape(command)
        );

        let _msg_id = self.send_message(&token, &message).await?;

        eprintln!("Sent request to Telegram. Waiting for response (timeout: {}s)...",
            self.timeout.as_secs());

        // Wait for response (only process messages after request_time)
        match self.wait_for_response(&token, request_time).await {
            Ok(true) => {
                eprintln!("{}", "✓ Approved via Telegram".green());
                Ok(true)
            }
            Ok(false) => {
                eprintln!("{}", "✗ Denied via Telegram".red());
                Err(AuthError::Cancelled)
            }
            Err(AuthError::Timeout) => {
                eprintln!("{}", "✗ Telegram response timeout".red());
                // Send timeout notification
                let _ = self.send_message(&token, "⏱️ Authorization request timed out.").await;
                Err(AuthError::Timeout)
            }
            Err(e) => Err(e),
        }
    }
}

// === Telegram API types ===

#[derive(Serialize)]
struct SendMessageRequest<'a> {
    chat_id: &'a str,
    text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<&'a str>,
}

#[derive(Serialize)]
struct GetUpdatesRequest {
    timeout: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<i64>,
}

#[derive(Deserialize)]
struct TelegramResponse<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct Message {
    message_id: i64,
    chat: Chat,
    text: Option<String>,
    date: Option<i64>,
}

#[derive(Deserialize)]
struct Chat {
    id: i64,
}

#[derive(Deserialize)]
struct Update {
    update_id: i64,
    message: Option<Message>,
}

/// Escape HTML special characters for Telegram
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telegram_name() {
        let auth = TelegramAuth::new("12345");
        assert_eq!(auth.name(), "telegram");
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("rm -rf <dir>"), "rm -rf &lt;dir&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_with_timeout() {
        let auth = TelegramAuth::new("12345").with_timeout(120);
        assert_eq!(auth.timeout, Duration::from_secs(120));
    }
}
