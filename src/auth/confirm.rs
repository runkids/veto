use dialoguer::Confirm;
use colored::Colorize;
use super::{AuthResult, AuthError, Authenticator};

pub struct ConfirmAuth;

impl ConfirmAuth {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConfirmAuth {
    fn default() -> Self {
        Self::new()
    }
}

impl Authenticator for ConfirmAuth {
    fn is_available(&self) -> bool {
        true
    }

    fn authenticate(&self, command: &str) -> AuthResult {
        eprintln!("{} {}", "Command:".yellow(), command);

        let confirmed = Confirm::new()
            .with_prompt("Allow this operation?")
            .default(false)
            .interact()
            .map_err(|_| AuthError::Cancelled)?;

        if confirmed {
            Ok(true)
        } else {
            Err(AuthError::Cancelled)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_available() {
        let auth = ConfirmAuth::new();
        assert!(auth.is_available());
    }
}
