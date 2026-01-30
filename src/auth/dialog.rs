//! macOS dialog authentication
//!
//! Uses osascript to show a native macOS dialog for confirmation.
//! Works in non-interactive environments (hooks, background processes).

use std::process::Command;
use super::{AuthResult, AuthError, Authenticator, AuthContext};

pub struct DialogAuth;

impl DialogAuth {
    pub fn new() -> Self {
        Self
    }

    /// Show dialog with context information
    fn show_dialog(&self, command: &str, context: Option<&AuthContext>) -> AuthResult {
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

        // Build dialog message with context
        let message = if let Some(ctx) = context {
            let ctx_info = ctx.format_for_display();
            if ctx_info.is_empty() {
                format!("Allow this command?\\n\\n{}", display_cmd)
            } else {
                format!("Allow this command?\\n\\n{}\\n\\n{}", display_cmd, ctx_info)
            }
        } else {
            format!("Allow this command?\\n\\n{}", display_cmd)
        };

        let script = format!(
            r#"display dialog "{}" with title "Veto Security" buttons {{"Deny", "Allow"}} default button "Deny" cancel button "Deny" with icon caution"#,
            message
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
        self.show_dialog(command, None)
    }

    fn authenticate_with_context(&self, command: &str, context: &AuthContext) -> AuthResult {
        self.show_dialog(command, Some(context))
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
