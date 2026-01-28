//! Audit logging for veto
//!
//! Logs all command evaluations to ~/.veto/audit.log

use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use chrono::Local;

use crate::config::loader::get_config_dir;
use crate::rules::RiskLevel;

const DENY_CACHE_FILENAME: &str = "deny_cache.json";

/// Audit log entry
pub struct AuditEntry {
    pub command: String,
    pub risk_level: RiskLevel,
    pub result: AuditResult,
    pub auth_method: Option<String>,
}

/// Result of the command evaluation
pub enum AuditResult {
    Allowed,
    Denied,
    Blocked,
}

impl std::fmt::Display for AuditResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditResult::Allowed => write!(f, "ALLOWED"),
            AuditResult::Denied => write!(f, "DENIED"),
            AuditResult::Blocked => write!(f, "BLOCKED"),
        }
    }
}

/// Log an audit entry
pub fn log_audit(entry: &AuditEntry) {
    if let Err(e) = log_audit_internal(entry) {
        eprintln!("Warning: Failed to write audit log: {}", e);
    }
}

fn log_audit_internal(entry: &AuditEntry) -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = get_config_dir();
    create_dir_all(&log_dir)?;

    let log_path = log_dir.join("audit.log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let risk = format!("{:?}", entry.risk_level).to_uppercase();
    let auth = entry.auth_method.as_deref().unwrap_or("-");

    // Format: [timestamp] RESULT RISK auth_method "command"
    writeln!(
        file,
        "[{}] {} {} {} {:?}",
        timestamp,
        entry.result,
        risk,
        auth,
        entry.command
    )?;

    Ok(())
}

/// Get the audit log path
pub fn get_audit_log_path() -> std::path::PathBuf {
    get_config_dir().join("audit.log")
}

/// Read all lines from the audit log
pub fn read_audit_log() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let log_path = get_audit_log_path();
    if !log_path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&log_path)?;
    Ok(content.lines().map(String::from).collect())
}

/// Clear the audit log
pub fn clear_audit_log() -> Result<(), Box<dyn std::error::Error>> {
    let log_path = get_audit_log_path();
    if log_path.exists() {
        std::fs::remove_file(&log_path)?;
    }
    Ok(())
}

fn get_deny_cache_path() -> std::path::PathBuf {
    get_config_dir().join(DENY_CACHE_FILENAME)
}

/// Record a command denial to prevent repeated prompts in hook modes.
pub fn record_denied_command(command: &str) {
    if let Err(e) = record_denied_command_internal(command) {
        eprintln!("Warning: Failed to update deny cache: {}", e);
    }
}

fn record_denied_command_internal(command: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cache_path = get_deny_cache_path();
    create_dir_all(get_config_dir())?;

    let mut entries: Vec<String> = if cache_path.exists() {
        let content = std::fs::read_to_string(&cache_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        vec![]
    };

    if !entries.iter().any(|c| c == command) {
        entries.push(command.to_string());
    }

    let content = serde_json::to_string_pretty(&entries)?;
    std::fs::write(cache_path, content)?;
    Ok(())
}

/// Check if a command was previously denied.
pub fn was_denied_command(command: &str) -> bool {
    let cache_path = get_deny_cache_path();
    if !cache_path.exists() {
        return false;
    }

    let Ok(content) = std::fs::read_to_string(&cache_path) else {
        return false;
    };

    let entries: Vec<String> = serde_json::from_str(&content).unwrap_or_default();
    entries.iter().any(|c| c == command)
}
