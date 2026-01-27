//! Auth subcommands for managing authentication methods

use clap::Subcommand;
use colored::Colorize;
use dialoguer::{Input, Password};

use crate::auth::{
    AuthenticatorFactory, PinAuth, TotpAuth, TelegramAuth,
    keyring::SecureKeyring,
};
use crate::config::loader::load_config;

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Set or update PIN
    SetPin,

    /// Setup TOTP (generates QR code)
    SetupTotp {
        /// Account name for TOTP (e.g., email)
        #[arg(short, long, default_value = "veto")]
        account: String,
    },

    /// Setup Telegram bot
    SetupTelegram,

    /// Test an authentication method
    Test {
        /// Method to test (confirm, pin, totp, touchid, telegram)
        method: String,
    },

    /// List configured authentication methods
    List,

    /// Remove an authentication method
    Remove {
        /// Method to remove (pin, totp, telegram)
        method: String,
    },
}

pub fn run_auth_command(cmd: AuthCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        AuthCommands::SetPin => run_set_pin(),
        AuthCommands::SetupTotp { account } => run_setup_totp(&account),
        AuthCommands::SetupTelegram => run_setup_telegram(),
        AuthCommands::Test { method } => run_test(&method),
        AuthCommands::List => run_list(),
        AuthCommands::Remove { method } => run_remove(&method),
    }
}

fn run_set_pin() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Setting up PIN authentication".cyan().bold());
    println!();

    // Check if PIN already exists
    if SecureKeyring::has_pin() {
        println!("{}", "A PIN is already configured.".yellow());

        let confirm = dialoguer::Confirm::new()
            .with_prompt("Replace existing PIN?")
            .default(false)
            .interact()?;

        if !confirm {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Get new PIN
    let pin: String = Password::new()
        .with_prompt("Enter new PIN (minimum 4 characters)")
        .with_confirmation("Confirm PIN", "PINs don't match")
        .interact()?;

    if pin.len() < 4 {
        return Err("PIN must be at least 4 characters".into());
    }

    // Store PIN
    PinAuth::set_pin(&pin)?;

    println!();
    println!("{}", "✓ PIN configured successfully!".green());
    Ok(())
}

fn run_setup_totp(account: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Setting up TOTP authentication".cyan().bold());
    println!();

    // Check if TOTP already exists
    if SecureKeyring::has_totp() {
        println!("{}", "TOTP is already configured.".yellow());

        let confirm = dialoguer::Confirm::new()
            .with_prompt("Replace existing TOTP?")
            .default(false)
            .interact()?;

        if !confirm {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Get issuer from config
    let config = load_config()?;
    let issuer = config
        .auth
        .and_then(|a| a.totp)
        .and_then(|t| t.issuer)
        .unwrap_or_else(|| "veto".to_string());

    // Generate TOTP
    let result = TotpAuth::setup(account, Some(&issuer))?;

    println!("Scan this QR code with your authenticator app:");
    println!();

    // Display QR code
    match result.generate_qr_terminal() {
        Ok(qr) => println!("{}", qr),
        Err(_) => println!("{}", "Failed to generate QR code".red()),
    }

    println!();
    println!("{} {}", "Manual entry secret:".bold(), result.secret);
    println!("{} {}", "OTP URL:".bold(), result.otpauth_url);
    println!();

    // Verify setup
    let code: String = Input::new()
        .with_prompt("Enter the 6-digit code to verify setup")
        .validate_with(|input: &String| {
            if input.len() == 6 && input.chars().all(|c| c.is_ascii_digit()) {
                Ok(())
            } else {
                Err("Code must be exactly 6 digits")
            }
        })
        .interact_text()?;

    // Use static verify method to avoid double input
    match TotpAuth::verify(&code) {
        Ok(true) => {
            println!();
            println!("{}", "✓ TOTP configured successfully!".green());
            Ok(())
        }
        Ok(false) => {
            // Try to delete on failure, ignore errors (cleanup is best-effort)
            let _ = TotpAuth::delete();
            Err("TOTP verification failed: invalid code. Check your authenticator app time sync.".into())
        }
        Err(e) => {
            // Try to delete on failure, ignore errors (cleanup is best-effort)
            let _ = TotpAuth::delete();
            Err(format!("TOTP verification error: {}. Setup cancelled.", e).into())
        }
    }
}

fn run_setup_telegram() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Setting up Telegram authentication".cyan().bold());
    println!();

    // Check if Telegram already exists
    if SecureKeyring::has_telegram() {
        println!("{}", "Telegram is already configured.".yellow());

        let confirm = dialoguer::Confirm::new()
            .with_prompt("Replace existing Telegram configuration?")
            .default(false)
            .interact()?;

        if !confirm {
            println!("Cancelled.");
            return Ok(());
        }
    }

    println!("To set up Telegram authentication:");
    println!("1. Create a bot via @BotFather on Telegram");
    println!("2. Get the bot token");
    println!("3. Start a chat with your bot");
    println!("4. Get your chat ID (use @userinfobot)");
    println!();

    let token: String = Password::new()
        .with_prompt("Enter bot token")
        .interact()?;

    if token.is_empty() {
        return Err("Bot token cannot be empty".into());
    }

    // Store token
    TelegramAuth::setup(&token)?;

    println!();
    println!("{}", "✓ Telegram bot token stored!".green());
    println!();
    println!("{}", "Next step:".bold());
    println!("Configure your chat_id in config.toml:");
    println!();
    println!("  [auth.telegram]");
    println!("  enabled = true");
    println!("  chat_id = \"YOUR_CHAT_ID\"");
    println!();

    Ok(())
}

fn run_test(method: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::auth::{Authenticator, ConfirmAuth, TouchIdAuth};
    use crate::auth::manager::AsyncAuthBridge;

    println!("{} {}", "Testing authentication method:".cyan().bold(), method);
    println!();

    let test_command = "veto auth test";

    let result = match method {
        "confirm" => {
            let auth = ConfirmAuth::new();
            auth.authenticate(test_command)
        }
        "pin" => {
            if !AuthenticatorFactory::is_available("pin") {
                return Err("PIN not configured. Run 'veto auth set-pin' first.".into());
            }
            let auth = PinAuth::new();
            auth.authenticate(test_command)
        }
        "totp" => {
            if !AuthenticatorFactory::is_available("totp") {
                return Err("TOTP not configured. Run 'veto auth setup-totp' first.".into());
            }
            let auth = TotpAuth::new();
            auth.authenticate(test_command)
        }
        "touchid" => {
            if !AuthenticatorFactory::is_available("touchid") {
                return Err("Touch ID is only available on macOS.".into());
            }
            let auth = TouchIdAuth::new();
            auth.authenticate(test_command)
        }
        "telegram" => {
            if !SecureKeyring::has_telegram() {
                return Err("Telegram not configured. Run 'veto auth setup-telegram' first.".into());
            }

            // Get chat_id from config
            let config = load_config()?;
            let chat_id = config
                .auth
                .and_then(|a| a.telegram)
                .and_then(|t| t.chat_id)
                .ok_or("Telegram chat_id not configured in config.toml")?;

            let auth = TelegramAuth::new(&chat_id);
            let bridge = AsyncAuthBridge::new(auth);
            bridge.authenticate(test_command)
        }
        _ => {
            return Err(format!("Unknown authentication method: {}", method).into());
        }
    };

    match result {
        Ok(true) => {
            println!();
            println!("{}", "✓ Authentication successful!".green());
            Ok(())
        }
        Ok(false) | Err(_) => {
            println!();
            println!("{}", "✗ Authentication failed.".red());
            Ok(())
        }
    }
}

fn run_list() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Configured authentication methods:".cyan().bold());
    println!();

    let methods = AuthenticatorFactory::list_configured();

    for (name, available) in methods {
        let status = if available {
            "✓".green()
        } else {
            "✗".red()
        };

        let availability = if available {
            "configured".green()
        } else {
            "not configured".dimmed()
        };

        println!("  {} {} - {}", status, name.bold(), availability);
    }

    println!();

    // Show config info
    let config = load_config()?;
    if let Some(auth) = config.auth {
        if let Some(default) = auth.default {
            println!("{} {}", "Default method:".bold(), default);
        }
        if let Some(levels) = auth.levels {
            println!("{}", "Level mappings:".bold());
            for (level, method) in levels {
                let method_str = match method {
                    crate::config::AuthMethod::Single(m) => m,
                    crate::config::AuthMethod::Multiple(ms) => ms.join(", "),
                };
                println!("  {} = {}", level, method_str);
            }
        }
    }

    Ok(())
}

fn run_remove(method: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} {}", "Removing authentication method:".cyan().bold(), method);

    let confirm = dialoguer::Confirm::new()
        .with_prompt(format!("Are you sure you want to remove {}?", method))
        .default(false)
        .interact()?;

    if !confirm {
        println!("Cancelled.");
        return Ok(());
    }

    match method {
        "pin" => {
            PinAuth::delete_pin()?;
            println!("{}", "✓ PIN removed.".green());
        }
        "totp" => {
            TotpAuth::delete()?;
            println!("{}", "✓ TOTP removed.".green());
        }
        "telegram" => {
            TelegramAuth::delete()?;
            println!("{}", "✓ Telegram configuration removed.".green());
        }
        "confirm" | "touchid" => {
            return Err(format!("{} cannot be removed (no stored secrets).", method).into());
        }
        _ => {
            return Err(format!("Unknown authentication method: {}", method).into());
        }
    }

    Ok(())
}
