use dialoguer::Password;
use colored::Colorize;
use super::{AuthResult, AuthError, Authenticator};

pub struct PinAuth {
    expected_pin: String,
}

impl PinAuth {
    pub fn new(pin: String) -> Self {
        Self { expected_pin: pin }
    }
}

impl Authenticator for PinAuth {
    fn name(&self) -> &'static str {
        "pin"
    }

    fn is_available(&self) -> bool {
        !self.expected_pin.is_empty()
    }

    fn authenticate(&self, command: &str) -> AuthResult {
        println!("{} {}", "Command:".yellow(), command);
        println!("{}", "PIN required for this operation".red());

        let entered = Password::new()
            .with_prompt("Enter PIN")
            .interact()
            .map_err(|_| AuthError::Cancelled)?;

        if entered == self.expected_pin {
            Ok(true)
        } else {
            Err(AuthError::Failed("Invalid PIN".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_available_when_set() {
        let auth = PinAuth::new("1234".to_string());
        assert!(auth.is_available());
    }

    #[test]
    fn test_pin_unavailable_when_empty() {
        let auth = PinAuth::new("".to_string());
        assert!(!auth.is_available());
    }
}
