//! macOS dialog authentication
//!
//! Uses osascript to show a native macOS dialog for confirmation.
//! Works in non-interactive environments (hooks, background processes).

use std::process::Command;
use super::{AuthResult, AuthError, Authenticator};

pub struct DialogAuth;

impl DialogAuth {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DialogAuth {
    fn default() -> Self {
        Self::new()
    }
}

impl Authenticator for DialogAuth {
    fn is_available(&self) -> bool {
        cfg!(target_os = "macos")
    }

    fn authenticate(&self, command: &str) -> AuthResult {
        if !self.is_available() {
            return Err(AuthError::NotAvailable("Dialog auth is only available on macOS".to_string()));
        }

        // Escape command for AppleScript
        let escaped_cmd = command
            .replace('\\', "\\\\")
            .replace('"', "\\\"");

        // Truncate long commands for display
        let display_cmd = if escaped_cmd.len() > 100 {
            format!("{}...", &escaped_cmd[..100])
        } else {
            escaped_cmd.clone()
        };

        let script = format!(
            r#"display dialog "Allow this command?\n\n{}" with title "Veto Security" buttons {{"Deny", "Allow"}} default button "Deny" cancel button "Deny" with icon caution"#,
            display_cmd
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| AuthError::Failed(format!("Failed to show dialog: {}", e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Allow") {
                Ok(true)
            } else {
                Err(AuthError::Cancelled)
            }
        } else {
            // User clicked cancel or closed dialog
            Err(AuthError::Cancelled)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_available_on_macos() {
        let auth = DialogAuth::new();
        #[cfg(target_os = "macos")]
        assert!(auth.is_available());
        #[cfg(not(target_os = "macos"))]
        assert!(!auth.is_available());
    }
}
