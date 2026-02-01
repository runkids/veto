//! macOS Touch ID / biometric authentication
//!
//! Uses macOS LocalAuthentication framework via Swift bridge.
//! Falls back to password authentication if Touch ID is unavailable.

use colored::Colorize;

use super::{AuthError, AuthResult, Authenticator, AuthContext};

/// Default prompt for Touch ID
const DEFAULT_PROMPT: &str = "Veto: Approve running this command?";

/// Touch ID authenticator (macOS only)
pub struct TouchIdAuth {
    prompt: String,
}

impl TouchIdAuth {
    pub fn new() -> Self {
        Self {
            prompt: DEFAULT_PROMPT.to_string(),
        }
    }

    /// Build prompt with context information
    fn build_prompt(&self, command: &str, _context: Option<&AuthContext>) -> String {
        let mut lines = vec![self.prompt.clone()];

        // Truncate command for display
        let display_cmd = if command.len() > 80 {
            format!("{}...", &command[..77])
        } else {
            command.to_string()
        };
        lines.push(format!("Command: {}", display_cmd));

        lines.join("\n")
    }


    /// Check if Touch ID is available on this system
    #[cfg(target_os = "macos")]
    pub fn check_available() -> bool {
        // On macOS, we'll use the biometric-prompt approach
        // which falls back to password if Touch ID is unavailable
        true
    }

    #[cfg(not(target_os = "macos"))]
    pub fn check_available() -> bool {
        false
    }

    /// Perform biometric authentication using compiled helper binary
    #[cfg(target_os = "macos")]
    fn do_authenticate(&self, prompt: &str) -> Result<bool, AuthError> {
        use std::process::Command;

        // Try to find Touch ID helper in common locations
        let helper_paths = [
            dirs::home_dir().map(|p| p.join(".local/bin/VetoAuth")),
            Some(std::path::PathBuf::from("/usr/local/bin/VetoAuth")),
            // Backward compatible name
            dirs::home_dir().map(|p| p.join(".local/bin/veto-touchid")),
            Some(std::path::PathBuf::from("/usr/local/bin/veto-touchid")),
        ];

        let helper_path = helper_paths
            .iter()
            .filter_map(|p| p.as_ref())
            .find(|p| p.exists());

        let output = match helper_path {
            Some(path) => {
                Command::new(path)
                    .arg(prompt)
                    .output()
                    .map_err(|e| AuthError::Failed(format!("Touch ID helper failed: {}", e)))?
            }
            None => {
                // Fallback to inline Swift if helper not found
                return self.do_authenticate_inline(prompt);
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.contains("AUTH_SUCCESS") {
            Ok(true)
        } else if stdout.contains("AUTH_UNAVAILABLE") {
            Err(AuthError::NotAvailable("Biometric authentication not available".to_string()))
        } else {
            Err(AuthError::Cancelled)
        }
    }

    /// Fallback: inline Swift execution (shows swift-frontend in dialog)
    #[cfg(target_os = "macos")]
    fn do_authenticate_inline(&self, prompt: &str) -> Result<bool, AuthError> {
        use std::process::Command;

        let escaped_prompt = prompt.replace('"', "\\\"");

        let swift_code = format!(r#"
import LocalAuthentication
import Foundation
let context = LAContext()
var error: NSError?
if context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) {{
    let sem = DispatchSemaphore(value: 0)
    var ok = false
    context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: "{}") {{ r, _ in ok = r; sem.signal() }}
    sem.wait()
    print(ok ? "AUTH_SUCCESS" : "AUTH_FAILED")
    exit(ok ? 0 : 1)
}}
if context.canEvaluatePolicy(.deviceOwnerAuthentication, error: &error) {{
    let sem = DispatchSemaphore(value: 0)
    var ok = false
    context.evaluatePolicy(.deviceOwnerAuthentication, localizedReason: "{}") {{ r, _ in ok = r; sem.signal() }}
    sem.wait()
    print(ok ? "AUTH_SUCCESS" : "AUTH_FAILED")
    exit(ok ? 0 : 1)
}}
print("AUTH_UNAVAILABLE")
exit(2)
"#, escaped_prompt, escaped_prompt);

        let output = Command::new("swift")
            .args(["-e", &swift_code])
            .output()
            .map_err(|e| AuthError::Failed(format!("Touch ID verification failed: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.contains("AUTH_SUCCESS") {
            Ok(true)
        } else if stdout.contains("AUTH_UNAVAILABLE") {
            Err(AuthError::NotAvailable("Biometric authentication not available".to_string()))
        } else {
            Err(AuthError::Cancelled)
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn do_authenticate(&self, _prompt: &str) -> Result<bool, AuthError> {
        Err(AuthError::NotAvailable("Touch ID is only available on macOS".to_string()))
    }
}

impl Default for TouchIdAuth {
    fn default() -> Self {
        Self::new()
    }
}

impl Authenticator for TouchIdAuth {
    fn is_available(&self) -> bool {
        Self::check_available()
    }

    fn authenticate(&self, command: &str) -> AuthResult {
        eprintln!("{} {}", "Command:".yellow(), command);
        eprintln!("{}", "Touch ID verification required".cyan());

        self.do_authenticate(&self.prompt)
    }

    fn authenticate_with_context(&self, command: &str, context: &AuthContext) -> AuthResult {
        let prompt = self.build_prompt(command, Some(context));

        // Print context info to stderr for visibility
        eprintln!("{} {}", "Command:".yellow(), command);
        if let Some(ref tool) = context.tool_name {
            eprintln!("{} {}", "Tool:".yellow(), tool);
        }
        if let Some(ref path) = context.file_path {
            eprintln!("{} {}", "File:".yellow(), path);
        }
        if let Some(ref cwd) = context.cwd {
            eprintln!("{} {}", "Dir:".yellow(), cwd);
        }
        eprintln!("{}", "Touch ID verification required".cyan());

        self.do_authenticate(&prompt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn test_touchid_available_on_macos() {
        assert!(TouchIdAuth::check_available());
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn test_touchid_unavailable_on_other_platforms() {
        assert!(!TouchIdAuth::check_available());
    }
}
