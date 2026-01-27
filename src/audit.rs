//! Audit logging for veto
//!
//! Logs all command evaluations to ~/.veto/audit.log

use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use chrono::Local;

use crate::config::loader::get_config_dir;
use crate::rules::RiskLevel;

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
