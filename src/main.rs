mod cli;
mod config;
mod rules;
mod auth;
mod executor;
mod commands;
mod audit;

use clap::Parser;
use colored::Colorize;
use cli::{Cli, Commands, SetupCommands};
use config::{loader::{load_config, load_rules}, Config};
use rules::{RulesEngine, RiskLevel};
use auth::{
    Authenticator, AuthManager, ConfirmAuth, PinAuth, TotpAuth, TouchIdAuth, TelegramAuth, DialogAuth,
    manager::AsyncAuthBridge,
    Challenge, notify_challenge, verify_response,
};
use executor::ShellExecutor;
use commands::{
    run_init,
    run_doctor,
    run_auth_command,
    run_shell,
    run_setup_claude,
    run_setup_gemini,
    run_setup_opencode,
    run_setup_cursor,
    run_upgrade,
    run_log,
};

fn main() {
    let cli = Cli::parse();
    let engine = RulesEngine::new(load_rules());

    match cli.command {
        Commands::Check { command } => {
            run_check(&engine, &command, cli.verbose);
        }
        Commands::Exec { command, auth } => {
            run_exec(&engine, &command, auth, cli.verbose);
        }
        Commands::Gate(args) => {
            // Parse stdin and extract command + context
            let (actual_command, auth_context) = if args.claude {
                let reader = if args.file_op {
                    read_claude_stdin_file_op
                } else {
                    read_claude_stdin_command
                };
                match reader() {
                    Ok(result) => (result.command, Some(result.context)),
                    Err(e) => {
                        eprintln!("{} {}", "Error reading Claude stdin:".red(), e);
                        std::process::exit(1);
                    }
                }
            } else if args.gemini {
                let reader = if args.file_op {
                    read_gemini_stdin_file_op
                } else {
                    read_gemini_stdin_command
                };
                match reader() {
                    Ok(result) => (result.command, Some(result.context)),
                    Err(e) => {
                        eprintln!("{} {}", "Error reading Gemini stdin:".red(), e);
                        std::process::exit(1);
                    }
                }
            } else if args.cursor {
                match read_cursor_stdin_command() {
                    Ok(result) => (result.command, Some(result.context)),
                    Err(e) => {
                        eprintln!("{} {}", "Error reading Cursor stdin:".red(), e);
                        std::process::exit(1);
                    }
                }
            } else {
                match args.command {
                    Some(cmd) => (cmd, None),
                    None => {
                        eprintln!("{}", "Error: command required (or use --claude/--gemini/--opencode/--cursor)".red());
                        std::process::exit(1);
                    }
                }
            };
            run_gate(
                &engine,
                &actual_command,
                auth_context,
                args.auth,
                args.totp,
                args.pin,
                cli.verbose,
                args.claude,
                args.opencode,
                args.gemini,
                args.cursor,
            );
        }
        Commands::Init { force } => {
            if let Err(e) = run_init(force) {
                eprintln!("{} {}", "Error:".red(), e);
                std::process::exit(1);
            }
        }
        Commands::Doctor => {
            run_doctor();
        }
        Commands::Shell => {
            if let Err(e) = run_shell() {
                eprintln!("{} {}", "Error:".red(), e);
                std::process::exit(1);
            }
        }
        Commands::Auth { command } => {
            if let Err(e) = run_auth_command(command) {
                eprintln!("{} {}", "Error:".red(), e);
                std::process::exit(1);
            }
        }
        Commands::Setup { command } => {
            match command {
                SetupCommands::Claude { uninstall } => {
                    if let Err(e) = run_setup_claude(uninstall) {
                        eprintln!("{} {}", "Error:".red(), e);
                        std::process::exit(1);
                    }
                }
                SetupCommands::Opencode { uninstall } => {
                    if let Err(e) = run_setup_opencode(uninstall) {
                        eprintln!("{} {}", "Error:".red(), e);
                        std::process::exit(1);
                    }
                }
                SetupCommands::Gemini { uninstall } => {
                    if let Err(e) = run_setup_gemini(uninstall) {
                        eprintln!("{} {}", "Error:".red(), e);
                        std::process::exit(1);
                    }
                }
                SetupCommands::Cursor { uninstall } => {
                    if let Err(e) = run_setup_cursor(uninstall) {
                        eprintln!("{} {}", "Error:".red(), e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Upgrade { check, force } => {
            if let Err(e) = run_upgrade(check, force) {
                eprintln!("{} {}", "Error:".red(), e);
                std::process::exit(1);
            }
        }
        Commands::Log(args) => {
            if let Err(e) = run_log(args) {
                eprintln!("{} {}", "Error:".red(), e);
                std::process::exit(1);
            }
        }
    }
}

/// Result from reading stdin, includes command and optional context
struct StdinReadResult {
    command: String,
    context: auth::AuthContext,
}

/// Read command from Claude Code stdin JSON format
///
/// Claude Code sends JSON like:
/// {"tool_name": "Bash", "tool_input": {"command": "rm -rf test"}, "cwd": "/path", "session_id": "abc123"}
fn read_claude_stdin_command() -> Result<StdinReadResult, Box<dyn std::error::Error>> {
    use std::io::Read;
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let json: serde_json::Value = serde_json::from_str(&input)?;

    let command = json["tool_input"]["command"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| "No command found in Claude JSON (expected tool_input.command)")?;

    // Extract context information
    let context = auth::AuthContext::new()
        .with_cwd(json["cwd"].as_str().unwrap_or("").to_string())
        .with_session_id(json["session_id"].as_str().unwrap_or("").to_string())
        .with_tool_name(json["tool_name"].as_str().unwrap_or("Bash").to_string());

    Ok(StdinReadResult { command, context })
}

/// Read file operation from Claude Code stdin JSON format
///
/// Claude Code sends JSON like:
/// {"tool_name":"Write","tool_input":{"file_path":"/etc/passwd","content":"..."}, "cwd": "/path", "session_id": "abc123"}
/// {"tool_name":"Edit","tool_input":{"file_path":"/etc/shadow","old_string":"...","new_string":"..."}}
///
/// Returns a synthetic command string for risk evaluation plus context
fn read_claude_stdin_file_op() -> Result<StdinReadResult, Box<dyn std::error::Error>> {
    use std::io::Read;
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let json: serde_json::Value = serde_json::from_str(&input)?;

    let tool_name = json["tool_name"]
        .as_str()
        .unwrap_or("unknown");

    let path = json["tool_input"]["file_path"]
        .as_str()
        .or_else(|| json["tool_input"]["path"].as_str());

    match path {
        Some(p) => {
            // Create synthetic command for risk evaluation
            // e.g., "write_file:/etc/passwd" or "edit_file:~/.ssh/authorized_keys"
            let command = format!("{}:{}", tool_name.to_lowercase(), p);

            // Extract context information
            let context = auth::AuthContext::new()
                .with_cwd(json["cwd"].as_str().unwrap_or("").to_string())
                .with_session_id(json["session_id"].as_str().unwrap_or("").to_string())
                .with_tool_name(tool_name.to_string())
                .with_file_path(p.to_string());

            Ok(StdinReadResult { command, context })
        }
        None => Err("No path found in Claude file operation JSON".into())
    }
}

/// Read command from Gemini CLI stdin JSON format
///
/// Gemini CLI sends JSON like:
/// {"tool_name":"run_shell_command","tool_input":{"command":"rm -rf test"}}
fn read_gemini_stdin_command() -> Result<StdinReadResult, Box<dyn std::error::Error>> {
    use std::io::Read;
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let json: serde_json::Value = serde_json::from_str(&input)?;

    let command = json["tool_input"]["command"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| "No command found in Gemini JSON (expected tool_input.command)")?;

    // Extract context information (Gemini may have similar fields)
    let context = auth::AuthContext::new()
        .with_cwd(json["cwd"].as_str().unwrap_or("").to_string())
        .with_session_id(json["session_id"].as_str().unwrap_or("").to_string())
        .with_tool_name(json["tool_name"].as_str().unwrap_or("run_shell_command").to_string());

    Ok(StdinReadResult { command, context })
}

/// Read file operation from Gemini CLI stdin JSON format
///
/// Gemini CLI sends JSON like:
/// {"tool_name":"write_file","tool_input":{"path":"/etc/passwd","content":"..."}}
/// {"tool_name":"edit_file","tool_input":{"path":"/etc/shadow","old_string":"...","new_string":"..."}}
///
/// Returns a synthetic command string for risk evaluation plus context
fn read_gemini_stdin_file_op() -> Result<StdinReadResult, Box<dyn std::error::Error>> {
    use std::io::Read;
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let json: serde_json::Value = serde_json::from_str(&input)?;

    let tool_name = json["tool_name"]
        .as_str()
        .unwrap_or("unknown");

    let path = json["tool_input"]["path"]
        .as_str()
        .or_else(|| json["tool_input"]["file_path"].as_str())
        .or_else(|| json["tool_input"]["target_file"].as_str());

    match path {
        Some(p) => {
            // Create synthetic command for risk evaluation
            // e.g., "write_file:/etc/passwd" or "edit_file:~/.ssh/authorized_keys"
            let command = format!("{}:{}", tool_name, p);

            // Extract context information
            let context = auth::AuthContext::new()
                .with_cwd(json["cwd"].as_str().unwrap_or("").to_string())
                .with_session_id(json["session_id"].as_str().unwrap_or("").to_string())
                .with_tool_name(tool_name.to_string())
                .with_file_path(p.to_string());

            Ok(StdinReadResult { command, context })
        }
        None => Err("No path found in Gemini file operation JSON".into())
    }
}

/// Read command from Cursor CLI stdin JSON format
///
/// Cursor CLI sends JSON like:
/// {"command":"rm -rf test","cwd":"/path/to/project"}
fn read_cursor_stdin_command() -> Result<StdinReadResult, Box<dyn std::error::Error>> {
    use std::io::Read;
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let json: serde_json::Value = serde_json::from_str(&input)?;

    let command = json["command"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| "No command found in Cursor JSON (expected command)")?;

    // Cursor provides cwd in the JSON
    let context = auth::AuthContext::new()
        .with_cwd(json["cwd"].as_str().unwrap_or("").to_string())
        .with_tool_name("Bash".to_string());

    Ok(StdinReadResult { command, context })
}

fn run_check(engine: &RulesEngine, command: &str, verbose: bool) {
    let result = engine.evaluate(command);

    let level_colored = match result.level {
        RiskLevel::Allow => "ALLOW".green(),
        RiskLevel::Low => "LOW".cyan(),
        RiskLevel::Medium => "MEDIUM".yellow(),
        RiskLevel::High => "HIGH".red(),
        RiskLevel::Critical => "CRITICAL".red().bold(),
    };

    println!("{} {}", "Risk:".bold(), level_colored);

    if verbose {
        if let Some(cat) = &result.category {
            println!("{} {}", "Category:".bold(), cat);
        }
        if let Some(reason) = &result.reason {
            println!("{} {}", "Reason:".bold(), reason);
        }
        if let Some(pattern) = &result.matched_pattern {
            println!("{} {}", "Pattern:".bold(), pattern);
        }
    }

    // Exit with appropriate code
    let exit_code = match result.level {
        RiskLevel::Allow => 0,
        RiskLevel::Low => 1,
        RiskLevel::Medium => 2,
        RiskLevel::High => 3,
        RiskLevel::Critical => 4,
    };
    std::process::exit(exit_code);
}

/// Parse environment variable prefixes from command string
/// Returns (extracted_env_vars, actual_command)
fn parse_env_prefix(command: &str) -> (std::collections::HashMap<String, String>, String) {
    let mut env_vars = std::collections::HashMap::new();
    let mut remaining = command.trim();

    // Pattern: VAR=value at the start of command
    while let Some(eq_pos) = remaining.find('=') {
        let before_eq = &remaining[..eq_pos];
        // Check if it's a valid env var name (no spaces before =)
        if before_eq.contains(' ') || before_eq.is_empty() {
            break;
        }

        // Find the end of the value (next space or end of string)
        let after_eq = &remaining[eq_pos + 1..];
        let value_end = after_eq.find(' ').unwrap_or(after_eq.len());
        let value = &after_eq[..value_end];

        env_vars.insert(before_eq.to_string(), value.to_string());

        // Move past this env var assignment
        if value_end < after_eq.len() {
            remaining = after_eq[value_end + 1..].trim_start();
        } else {
            remaining = "";
            break;
        }
    }

    (env_vars, remaining.to_string())
}

/// Gate command - verify only, no execute (for use in hooks)
///
/// Flow:
/// 1. AI executes command → veto gate intercepts
/// 2. If high-risk and no credentials provided → output AUTH_REQUIRED with method
/// 3. AI tells user verification needed
/// 4. User provides code in chat
/// 5. AI retries with credentials: veto gate --totp 123456 "command"
/// 6. veto verifies → exit 0 (allow) or exit 1 (deny)
fn run_gate(
    engine: &RulesEngine,
    command: &str,
    auth_context: Option<auth::AuthContext>,
    auth_override: Option<String>,
    totp_code: Option<String>,
    pin_code: Option<String>,
    verbose: bool,
    claude_mode: bool,
    opencode_mode: bool,
    gemini_mode: bool,
    cursor_mode: bool,
) {
    // Parse environment variable prefixes from command string
    let (cmd_env_vars, actual_command) = parse_env_prefix(command);

    // Use actual command (without env prefix) for risk evaluation
    let eval_command = if actual_command.is_empty() { command } else { &actual_command };
    let result = engine.evaluate(eval_command);

    if verbose {
        let level_colored = match result.level {
            RiskLevel::Allow => "ALLOW".green(),
            RiskLevel::Low => "LOW".cyan(),
            RiskLevel::Medium => "MEDIUM".yellow(),
            RiskLevel::High => "HIGH".red(),
            RiskLevel::Critical => "CRITICAL".red().bold(),
        };
        eprintln!("{} {}", "Risk:".bold(), level_colored);
    }

    // Allow level always passes through without auth
    if matches!(result.level, RiskLevel::Allow) {
        if gemini_mode {
            let json = serde_json::json!({ "decision": "allow" });
            println!("{}", json);
        } else if cursor_mode {
            let json = serde_json::json!({ "continue": true, "permission": "allow" });
            println!("{}", json);
        }
        std::process::exit(0);
    }

    // Check if auth is configured for this risk level
    let config = load_config().unwrap_or_default();
    let auth_methods: Vec<String> = if let Some(method) = auth_override.as_deref() {
        vec![method.to_string()]
    } else if let Some(auth_config) = &config.auth {
        let manager = AuthManager::new(auth_config.clone());
        let methods = manager.get_methods_for_level(&result.level.clone().into());
        if methods.is_empty() {
            if let Some(default) = &auth_config.default {
                vec![default.clone()]
            } else {
                vec![] // No auth configured
            }
        } else {
            methods
        }
    } else {
        vec![] // No auth config at all
    };

    // If no auth method configured for this level, pass through
    if auth_methods.is_empty() {
        if gemini_mode {
            let json = serde_json::json!({ "decision": "allow" });
            println!("{}", json);
        } else if cursor_mode {
            let json = serde_json::json!({ "continue": true, "permission": "allow" });
            println!("{}", json);
        }
        std::process::exit(0);
    }

    let primary_method: &str = &auth_methods[0];

    // Command requires auth - get reason for display
    let reason = result.reason.as_deref().unwrap_or("Operation requires verification");

    // Check if credentials were provided (CLI args, environment variables, or command prefix)
    let env_pin = std::env::var("VETO_PIN").ok()
        .or_else(|| cmd_env_vars.get("VETO_PIN").cloned());
    let env_totp = std::env::var("VETO_TOTP").ok()
        .or_else(|| cmd_env_vars.get("VETO_TOTP").cloned());
    let env_confirm = std::env::var("VETO_CONFIRM").ok()
        .or_else(|| cmd_env_vars.get("VETO_CONFIRM").cloned());
    // VETO_RESPONSE for challenge-response authentication (PIN+challenge or just challenge)
    let env_response = std::env::var("VETO_RESPONSE").ok()
        .or_else(|| cmd_env_vars.get("VETO_RESPONSE").cloned());

    let effective_pin = pin_code.or(env_pin);
    let effective_totp = totp_code.or(env_totp);
    let has_credentials = effective_totp.is_some() || effective_pin.is_some() || env_confirm.is_some();
    let has_response = env_response.is_some();
    let env_force = std::env::var("VETO_FORCE").ok()
        .or_else(|| cmd_env_vars.get("VETO_FORCE").cloned());
    let force_retry = env_force
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
        .unwrap_or(false);

    // Check if this rule requires challenge-response authentication
    let requires_challenge = result.challenge && (claude_mode || opencode_mode || gemini_mode || cursor_mode);

    // Hook modes: prevent repeated prompts after a user denial unless explicitly forced
    if (claude_mode || opencode_mode || gemini_mode || cursor_mode)
        && !force_retry
        && audit::was_denied_command(eval_command)
    {
        let msg = "[veto] Previously rejected. Not retrying without user override. Retry with VETO_FORCE=yes if user explicitly approved.";
        output_blocked(eval_command, &result.level, msg, claude_mode, gemini_mode, cursor_mode);
    }

    // Handle challenge-response verification if VETO_RESPONSE is provided
    if requires_challenge && has_response {
        if let Some(response) = &env_response {
            let verify_result = verify_response(response, eval_command, primary_method);
            if verify_result.success {
                output_allowed(eval_command, &result.level, &verify_result.method, claude_mode, gemini_mode, cursor_mode);
            } else {
                let error_msg = verify_result.error.unwrap_or_else(|| "Challenge verification failed".to_string());
                output_blocked(eval_command, &result.level, &error_msg, claude_mode, gemini_mode, cursor_mode);
            }
        }
    }

    if !has_credentials {
        // No credentials - behavior depends on auth method
        match primary_method {
            "totp" => {
                // Check if TOTP is configured
                let auth = TotpAuth::new();
                if !auth.is_available() {
                    let msg = "[veto] TOTP not configured. User must run 'veto auth setup-totp' first to enable TOTP authentication.";
                    if gemini_mode || cursor_mode {
                        output_blocked(eval_command, &result.level, msg, claude_mode, gemini_mode, cursor_mode);
                    } else {
                        eprintln!("{}", msg);
                        std::process::exit(2);
                    }
                }
                // TOTP configured - ask for code
                if cursor_mode {
                    let msg = "[veto] TOTP required, but Cursor CLI hooks cannot accept codes. Run the command in a terminal with VETO_TOTP=<code> or configure dialog/touchid.";
                    output_blocked(eval_command, &result.level, msg, claude_mode, gemini_mode, cursor_mode);
                } else if claude_mode || opencode_mode || gemini_mode {
                    let msg = format!(
                        "[veto] {} command blocked. Ask user in chat for their TOTP code. If provided, retry command with VETO_TOTP=<code> prefix.",
                        risk_level_str(&result.level)
                    );
                    if gemini_mode {
                        output_blocked(eval_command, &result.level, &msg, claude_mode, gemini_mode, cursor_mode);
                    } else {
                        eprintln!("{}", msg);
                        std::process::exit(2);
                    }
                } else {
                    eprintln!("{}", "⚠️  AUTH_REQUIRED".red().bold());
                    eprintln!("risk_level: {}", risk_level_str(&result.level));
                    eprintln!("reason: {}", reason);
                    eprintln!("command: {}", command);
                    eprintln!("auth_method: totp");
                    eprintln!();
                    eprintln!("Retry with environment variable:");
                    eprintln!("  VETO_TOTP=<code> <command>");
                    std::process::exit(2);
                }
            }
            "pin" => {
                // Check if PIN is configured
                let auth = PinAuth::new();
                if !auth.is_available() {
                    let msg = "[veto] PIN not configured. User must run 'veto auth set-pin' first to enable PIN authentication.";
                    if gemini_mode || cursor_mode {
                        output_blocked(eval_command, &result.level, msg, claude_mode, gemini_mode, cursor_mode);
                    } else {
                        eprintln!("{}", msg);
                        std::process::exit(2);
                    }
                }
                // PIN configured - ask for code
                if cursor_mode {
                    let msg = if requires_challenge {
                        "[veto] PIN+challenge required, but Cursor CLI hooks cannot accept codes. Run the command in a terminal with VETO_RESPONSE=<PIN><challenge> or configure dialog/touchid."
                    } else {
                        "[veto] PIN required, but Cursor CLI hooks cannot accept codes. Run the command in a terminal with VETO_PIN=<code> or configure dialog/touchid."
                    };
                    output_blocked(eval_command, &result.level, msg, claude_mode, gemini_mode, cursor_mode);
                } else if claude_mode || opencode_mode || gemini_mode {
                    if requires_challenge {
                        // Generate challenge and send notification
                        match Challenge::generate(eval_command) {
                            Ok(challenge) => {
                                if let Err(e) = notify_challenge(&challenge, eval_command) {
                                    eprintln!("[veto] Warning: Failed to send notification: {}", e);
                                }
                                let msg = format!(
                                    "[veto] {} command blocked (challenge required). Challenge code sent via notification. Ask user to check notification and combine PIN + challenge code (format: PIN followed by challenge). Retry with VETO_RESPONSE=<PIN><challenge> prefix.",
                                    risk_level_str(&result.level)
                                );
                                if gemini_mode {
                                    output_blocked(eval_command, &result.level, &msg, claude_mode, gemini_mode, cursor_mode);
                                } else {
                                    eprintln!("{}", msg);
                                    std::process::exit(2);
                                }
                            }
                            Err(e) => {
                                let msg = format!("[veto] Failed to generate challenge: {}", e);
                                if gemini_mode {
                                    output_blocked(eval_command, &result.level, &msg, claude_mode, gemini_mode, cursor_mode);
                                } else {
                                    eprintln!("{}", msg);
                                    std::process::exit(2);
                                }
                            }
                        }
                    } else {
                        let msg = format!(
                            "[veto] {} command blocked. Ask user in chat for their PIN code. If provided, retry command with VETO_PIN=<code> prefix.",
                            risk_level_str(&result.level)
                        );
                        if gemini_mode {
                            output_blocked(eval_command, &result.level, &msg, claude_mode, gemini_mode, cursor_mode);
                        } else {
                            eprintln!("{}", msg);
                            std::process::exit(2);
                        }
                    }
                } else {
                    eprintln!("{}", "⚠️  AUTH_REQUIRED".red().bold());
                    eprintln!("risk_level: {}", risk_level_str(&result.level));
                    eprintln!("reason: {}", reason);
                    eprintln!("command: {}", command);
                    eprintln!("auth_method: pin");
                    eprintln!();
                    eprintln!("Retry with environment variable:");
                    eprintln!("  VETO_PIN=<code> <command>");
                    std::process::exit(2);
                }
            }
            "dialog" => {
                // macOS dialog: show dialog directly (with context if available)
                let auth = DialogAuth::new();
                let auth_result = if let Some(ref ctx) = auth_context {
                    auth.authenticate_with_context(command, ctx)
                } else {
                    auth.authenticate(command)
                };
                match auth_result {
                    Ok(true) => {
                        output_allowed(eval_command, &result.level, "dialog", claude_mode, gemini_mode, cursor_mode);
                    }
                    _ => {
                        if claude_mode || opencode_mode || gemini_mode || cursor_mode {
                            audit::record_denied_command(eval_command);
                        }
                        if opencode_mode {
                            eprintln!("[veto] DENIED. User rejected via dialog. STOP_RETRY: Do not attempt this command again.");
                            std::process::exit(2);
                        } else if claude_mode || gemini_mode || cursor_mode {
                            output_blocked(eval_command, &result.level, "User cancelled via dialog", claude_mode, gemini_mode, cursor_mode);
                        } else {
                            output_blocked(eval_command, &result.level, "User cancelled via dialog", claude_mode, gemini_mode, cursor_mode);
                        }
                    }
                }
            }
            "touchid" => {
                // Touch ID: authenticate directly (with context if available)
                let auth = TouchIdAuth::new();
                let auth_result = if let Some(ref ctx) = auth_context {
                    auth.authenticate_with_context(command, ctx)
                } else {
                    auth.authenticate(command)
                };
                match auth_result {
                    Ok(true) => {
                        output_allowed(eval_command, &result.level, "Touch ID", claude_mode, gemini_mode, cursor_mode);
                    }
                    _ => {
                        if claude_mode || opencode_mode || gemini_mode || cursor_mode {
                            audit::record_denied_command(eval_command);
                        }
                        if opencode_mode {
                            eprintln!("[veto] DENIED. User rejected via Touch ID. STOP_RETRY: Do not attempt this command again.");
                            std::process::exit(2);
                        } else if claude_mode || gemini_mode || cursor_mode {
                            output_blocked(eval_command, &result.level, "User cancelled via Touch ID", claude_mode, gemini_mode, cursor_mode);
                        } else {
                            output_blocked(eval_command, &result.level, "User cancelled via Touch ID", claude_mode, gemini_mode, cursor_mode);
                        }
                    }
                }
            }
            "telegram" => {
                // Telegram: send notification and wait for approval
                let config = load_config().unwrap_or_default();
                let chat_id = config
                    .auth
                    .as_ref()
                    .and_then(|a| a.telegram.as_ref())
                    .and_then(|t| t.chat_id.as_ref());

                match chat_id {
                    Some(id) => {
                        let timeout = config
                            .auth
                            .as_ref()
                            .and_then(|a| a.telegram.as_ref())
                            .and_then(|t| t.timeout_seconds)
                            .unwrap_or(60);

                        let auth = TelegramAuth::new(id).with_timeout(timeout as u64);
                        let bridge = AsyncAuthBridge::new(auth);

                        eprintln!("📱 Telegram approval request sent. Waiting for response...");
                        match bridge.authenticate(command) {
                            Ok(true) => {
                                output_allowed(eval_command, &result.level, "Telegram", claude_mode, gemini_mode, cursor_mode);
                            }
                            _ => {
                                if claude_mode || opencode_mode || gemini_mode || cursor_mode {
                                    audit::record_denied_command(eval_command);
                                }
                                if opencode_mode {
                                    eprintln!("[veto] DENIED. User rejected via Telegram. STOP_RETRY: Do not attempt this command again.");
                                    std::process::exit(2);
                                } else if claude_mode || gemini_mode || cursor_mode {
                                    output_blocked(eval_command, &result.level, "User denied via Telegram", claude_mode, gemini_mode, cursor_mode);
                                } else {
                                    output_blocked(eval_command, &result.level, "User denied via Telegram", claude_mode, gemini_mode, cursor_mode);
                                }
                            }
                        }
                    }
                    None => {
                        let msg = "Telegram not configured. Run 'veto auth setup-telegram' first.";
                        if gemini_mode || cursor_mode {
                            output_blocked(eval_command, &result.level, msg, claude_mode, gemini_mode, cursor_mode);
                        } else {
                            eprintln!("{}", msg);
                            std::process::exit(2);
                        }
                    }
                }
            }
            "confirm" => {
                // Check environment variable first (already parsed above)
                if let Some(ref val) = env_confirm {
                    if val.to_lowercase() == "yes" || val == "1" || val.to_lowercase() == "true" {
                        output_allowed(eval_command, &result.level, "VETO_CONFIRM", claude_mode, gemini_mode, cursor_mode);
                    }
                }

                if cursor_mode {
                    if requires_challenge {
                        let msg = "[veto] Challenge confirmation required, but Cursor CLI hooks cannot accept codes. Run the command in a terminal with VETO_RESPONSE=<challenge> or configure dialog/touchid.";
                        output_blocked(eval_command, &result.level, msg, claude_mode, gemini_mode, cursor_mode);
                    } else {
                        let user_message = format!(
                            "{} command requires approval.",
                            risk_level_str(&result.level)
                        );
                        let agent_message = format!(
                            "{}: {}",
                            reason,
                            eval_command
                        );
                        output_cursor_ask(eval_command, &result.level, &user_message, &agent_message);
                    }
                } else if opencode_mode || claude_mode || gemini_mode {
                    if requires_challenge {
                        // Generate challenge and send notification
                        match Challenge::generate(eval_command) {
                            Ok(challenge) => {
                                if let Err(e) = notify_challenge(&challenge, eval_command) {
                                    eprintln!("[veto] Warning: Failed to send notification: {}", e);
                                }
                                let msg = format!(
                                    "[veto] {} command blocked (challenge required). Challenge code sent via notification. Ask user to check notification and enter the 4-digit challenge code. Retry with VETO_RESPONSE=<challenge> prefix.",
                                    risk_level_str(&result.level)
                                );
                                if gemini_mode {
                                    output_blocked(eval_command, &result.level, &msg, claude_mode, gemini_mode, cursor_mode);
                                } else {
                                    eprintln!("{}", msg);
                                    std::process::exit(2);
                                }
                            }
                            Err(e) => {
                                let msg = format!("[veto] Failed to generate challenge: {}", e);
                                if gemini_mode {
                                    output_blocked(eval_command, &result.level, &msg, claude_mode, gemini_mode, cursor_mode);
                                } else {
                                    eprintln!("{}", msg);
                                    std::process::exit(2);
                                }
                            }
                        }
                    } else {
                        // AI mode: tell AI how to retry with user confirmation
                        let msg = format!(
                            "[veto] {} command blocked. Ask user in chat: \"Do you want to allow `{}`?\" If YES, retry command with VETO_CONFIRM=yes prefix.",
                            risk_level_str(&result.level),
                            eval_command
                        );
                        if gemini_mode {
                            output_blocked(eval_command, &result.level, &msg, claude_mode, gemini_mode, cursor_mode);
                        } else {
                            eprintln!("{}", msg);
                            std::process::exit(2);
                        }
                    }
                } else {
                    // Terminal mode: interactive confirmation
                    let auth = ConfirmAuth::new();
                    match auth.authenticate(command) {
                        Ok(true) => {
                            output_allowed(eval_command, &result.level, "confirmation", claude_mode, gemini_mode, cursor_mode);
                        }
                        _ => {
                            output_blocked(eval_command, &result.level, "User cancelled confirmation", claude_mode, gemini_mode, cursor_mode);
                        }
                    }
                }
            }
            _ => {
                // Default: require setup
                let msg = "No suitable auth method configured. Run 'veto auth setup-totp' or 'veto auth set-pin' first.";
                if gemini_mode || cursor_mode {
                    output_blocked(eval_command, &result.level, msg, claude_mode, gemini_mode, cursor_mode);
                } else {
                    eprintln!("{}", "⚠️  AUTH_REQUIRED".red().bold());
                    eprintln!("{}", msg);
                    std::process::exit(2);
                }
            }
        }
    }

    // Credentials provided - verify them
    let (verified, method) = if let Some(code) = effective_totp {
        (verify_totp(&code), "TOTP")
    } else if let Some(code) = effective_pin {
        (verify_pin(&code), "PIN")
    } else if let Some(ref val) = env_confirm {
        // VETO_CONFIRM - check if it's "yes"
        let ok = val.to_lowercase() == "yes" || val == "1" || val.to_lowercase() == "true";
        (ok, "VETO_CONFIRM")
    } else {
        (false, "unknown")
    };

    if verified {
        output_allowed(eval_command, &result.level, method, claude_mode, gemini_mode, cursor_mode);
    } else {
        output_blocked(eval_command, &result.level, "Verification failed", claude_mode, gemini_mode, cursor_mode);
    }
}

/// Convert risk level to display string
fn risk_level_str(level: &RiskLevel) -> &'static str {
    match level {
        RiskLevel::Allow => "ALLOW",
        RiskLevel::Low => "LOW",
        RiskLevel::Medium => "MEDIUM",
        RiskLevel::High => "HIGH",
        RiskLevel::Critical => "CRITICAL",
    }
}

/// Output allowed message - JSON for Claude mode to bypass permission prompt
fn output_allowed(
    command: &str,
    risk_level: &RiskLevel,
    method: &str,
    claude_mode: bool,
    gemini_mode: bool,
    cursor_mode: bool,
) -> ! {
    // Log to audit trail
    audit::log_audit(&audit::AuditEntry {
        command: command.to_string(),
        risk_level: risk_level.clone(),
        result: audit::AuditResult::Allowed,
        auth_method: Some(method.to_string()),
    });

    if claude_mode {
        // Claude Code hooks: permissionDecision "allow" bypasses permission prompt
        let json = serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "permissionDecisionReason": format!("Authorized via veto {}", method)
            }
        });
        println!("{}", json);
        std::process::exit(0);
    } else if gemini_mode {
        let json = serde_json::json!({
            "decision": "allow",
            "reason": format!("Authorized via veto {}", method)
        });
        println!("{}", json);
        std::process::exit(0);
    } else if cursor_mode {
        let json = serde_json::json!({
            "continue": true,
            "permission": "allow"
        });
        println!("{}", json);
        std::process::exit(0);
    } else {
        eprintln!("{}", format!("✓ Approved via {}", method).green());
        std::process::exit(0);
    }
}

/// Output blocked message - JSON for Claude mode, text for normal mode
fn output_blocked(
    command: &str,
    risk_level: &RiskLevel,
    reason: &str,
    claude_mode: bool,
    gemini_mode: bool,
    cursor_mode: bool,
) -> ! {
    // Log to audit trail
    audit::log_audit(&audit::AuditEntry {
        command: command.to_string(),
        risk_level: risk_level.clone(),
        result: audit::AuditResult::Denied,
        auth_method: None,
    });

    if claude_mode {
        // Claude Code hooks: output JSON with deny decision and continue: false
        // This tells Claude Code to stop completely without showing its own dialog
        let json = serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": reason
            },
            "continue": false
        });
        println!("{}", json);
        std::process::exit(0);
    } else if gemini_mode {
        let json = serde_json::json!({
            "decision": "deny",
            "reason": reason,
            "systemMessage": reason
        });
        println!("{}", json);
        std::process::exit(0);
    } else if cursor_mode {
        let json = serde_json::json!({
            "continue": false,
            "permission": "deny",
            "user_message": reason,
            "agent_message": reason
        });
        println!("{}", json);
        std::process::exit(0);
    } else {
        // Normal mode: text message with exit code 2 (blocking)
        eprintln!("{}", reason.red());
        std::process::exit(2);
    }
}

/// Output ask message for Cursor CLI (permission prompt handled by Cursor)
fn output_cursor_ask(
    command: &str,
    risk_level: &RiskLevel,
    user_message: &str,
    agent_message: &str,
) -> ! {
    audit::log_audit(&audit::AuditEntry {
        command: command.to_string(),
        risk_level: risk_level.clone(),
        result: audit::AuditResult::Blocked,
        auth_method: None,
    });

    let json = serde_json::json!({
        "continue": true,
        "permission": "ask",
        "user_message": user_message,
        "agent_message": agent_message
    });
    println!("{}", json);
    std::process::exit(0);
}

/// Verify TOTP code
fn verify_totp(code: &str) -> bool {
    let auth = TotpAuth::new();
    if !auth.is_available() {
        eprintln!("TOTP not configured. Run 'veto auth setup-totp' first.");
        return false;
    }
    // TotpAuth::verify is a static method
    TotpAuth::verify(code).unwrap_or(false)
}

/// Verify PIN code
fn verify_pin(code: &str) -> bool {
    let auth = PinAuth::new();
    if !auth.is_available() {
        eprintln!("PIN not configured. Run 'veto auth set-pin' first.");
        return false;
    }
    // PinAuth verify_pin is private, need to add public method
    // For now, use authenticate which prompts - we need to add verify method
    auth.verify_direct(code).unwrap_or(false)
}

fn run_exec(engine: &RulesEngine, command: &str, auth_override: Option<String>, verbose: bool) {
    let result = engine.evaluate(command);

    if verbose {
        let level_colored = match result.level {
            RiskLevel::Allow => "ALLOW".green(),
            RiskLevel::Low => "LOW".cyan(),
            RiskLevel::Medium => "MEDIUM".yellow(),
            RiskLevel::High => "HIGH".red(),
            RiskLevel::Critical => "CRITICAL".red().bold(),
        };
        println!("{} {}", "Risk:".bold(), level_colored);
    }

    // Check if auth is required
    let needs_auth = !matches!(result.level, RiskLevel::Allow);

    if needs_auth {
        // Show warning for high-risk commands
        if matches!(result.level, RiskLevel::High | RiskLevel::Critical) {
            println!("{}", "⚠️  High-risk operation detected!".red().bold());
            if let Some(reason) = &result.reason {
                println!("{} {}", "Reason:".bold(), reason);
            }
        }

        // Get auth methods based on config and risk level
        let auth_methods = get_auth_methods(&result.level, auth_override.as_deref());

        // Run authentication chain
        match run_auth_chain(&auth_methods, command) {
            Ok(()) => {
                println!("{}", "✓ Authentication passed".green());
            }
            Err(e) => {
                eprintln!("{} {}", "Authentication failed:".red(), e);
                std::process::exit(1);
            }
        }
    }

    // Execute the command using shell
    println!("{} {}", "Executing:".cyan(), command);
    println!("{} {:?}", "Working dir:".cyan(), std::env::current_dir().unwrap_or_default());
    let executor = ShellExecutor::new();
    match executor.execute(command) {
        Ok(status) => {
            let code = status.code().unwrap_or(1);
            println!("{} {}", "Exit code:".cyan(), code);
            std::process::exit(code);
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            std::process::exit(1);
        }
    }
}

/// Get authentication methods for a risk level
fn get_auth_methods(level: &RiskLevel, auth_override: Option<&str>) -> Vec<String> {
    // If override specified, use that
    if let Some(method) = auth_override {
        return vec![method.to_string()];
    }

    // Load config and get methods for level
    let config = match load_config() {
        Ok(c) => c,
        Err(_) => return vec!["confirm".to_string()],
    };

    let auth_config = match config.auth {
        Some(a) => a,
        None => return vec!["confirm".to_string()],
    };

    let manager = AuthManager::new(auth_config);
    manager.get_methods_for_level(&level.clone().into())
}

/// Run authentication chain
fn run_auth_chain(methods: &[String], command: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config().unwrap_or_default();

    for method in methods {
        let authenticated = match method.as_str() {
            "confirm" => {
                let auth = ConfirmAuth::new();
                auth.authenticate(command)?
            }
            "pin" => {
                let auth = PinAuth::new();
                if !auth.is_available() {
                    return Err("PIN not configured. Run 'veto auth set-pin' first.".into());
                }
                auth.authenticate(command)?
            }
            "totp" => {
                let auth = TotpAuth::new();
                if !auth.is_available() {
                    return Err("TOTP not configured. Run 'veto auth setup-totp' first.".into());
                }
                auth.authenticate(command)?
            }
            "touchid" => {
                let auth = TouchIdAuth::new();
                if !auth.is_available() {
                    return Err("Touch ID is only available on macOS.".into());
                }
                auth.authenticate(command)?
            }
            "dialog" => {
                let auth = DialogAuth::new();
                if !auth.is_available() {
                    return Err("Dialog auth is only available on macOS.".into());
                }
                auth.authenticate(command)?
            }
            "telegram" => {
                let chat_id = config
                    .auth
                    .as_ref()
                    .and_then(|a| a.telegram.as_ref())
                    .and_then(|t| t.chat_id.as_ref())
                    .ok_or("Telegram chat_id not configured")?;

                let timeout = config
                    .auth
                    .as_ref()
                    .and_then(|a| a.telegram.as_ref())
                    .and_then(|t| t.timeout_seconds)
                    .unwrap_or(60);

                let auth = TelegramAuth::new(chat_id).with_timeout(timeout as u64);
                let bridge = AsyncAuthBridge::new(auth);
                bridge.authenticate(command)?
            }
            _ => {
                return Err(format!("Unknown authentication method: {}", method).into());
            }
        };

        if !authenticated {
            return Err("Authentication cancelled".into());
        }
    }

    Ok(())
}

/// Auto-select best available auth method
fn auto_select_auth_method(config: &Config) -> String {
    // Priority: telegram > touchid > dialog > totp > pin > confirm
    if config.auth.as_ref()
        .and_then(|a| a.telegram.as_ref())
        .and_then(|t| t.chat_id.as_ref())
        .is_some()
    {
        "telegram".to_string()
    } else if TouchIdAuth::new().is_available() {
        "touchid".to_string()
    } else if cfg!(target_os = "macos") {
        "dialog".to_string()
    } else if TotpAuth::new().is_available() {
        "totp".to_string()
    } else if PinAuth::new().is_available() {
        "pin".to_string()
    } else {
        crate::auth::default_auth_method().to_string()
    }
}
