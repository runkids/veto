//! Audit logging for veto
//!
//! Logs all command evaluations to ~/.veto/audit.log

use chrono::Local;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;

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
#[allow(dead_code)]
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
        timestamp, entry.result, risk, auth, entry.command
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Set VETO_HOME to a temp directory for test isolation.
    /// Uses a mutex to prevent parallel env var modifications.
    fn with_temp_config<F: FnOnce()>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("VETO_HOME", tmp.path());
        }
        f();
        unsafe {
            std::env::remove_var("VETO_HOME");
        }
    }

    #[test]
    fn test_log_and_read() {
        with_temp_config(|| {
            let entry = AuditEntry {
                command: "rm -rf /".to_string(),
                risk_level: RiskLevel::Critical,
                result: AuditResult::Blocked,
                auth_method: Some("touchid".to_string()),
            };
            log_audit(&entry);

            let lines = read_audit_log().unwrap();
            assert_eq!(lines.len(), 1);
            let line = &lines[0];
            assert!(line.contains("BLOCKED"), "should contain BLOCKED");
            assert!(line.contains("CRITICAL"), "should contain CRITICAL");
            assert!(line.contains("touchid"), "should contain touchid");
            assert!(line.contains("rm -rf /"), "should contain command");
        });
    }

    #[test]
    fn test_read_empty_log() {
        with_temp_config(|| {
            let lines = read_audit_log().unwrap();
            assert!(lines.is_empty());
        });
    }

    #[test]
    fn test_clear_log() {
        with_temp_config(|| {
            let entry = AuditEntry {
                command: "ls".to_string(),
                risk_level: RiskLevel::Low,
                result: AuditResult::Allowed,
                auth_method: None,
            };
            log_audit(&entry);
            assert!(!read_audit_log().unwrap().is_empty());

            clear_audit_log().unwrap();
            assert!(read_audit_log().unwrap().is_empty());
        });
    }

    #[test]
    fn test_deny_cache_round_trip() {
        with_temp_config(|| {
            assert!(!was_denied_command("dangerous_cmd"));
            record_denied_command("dangerous_cmd");
            assert!(was_denied_command("dangerous_cmd"));
        });
    }

    #[test]
    fn test_deny_cache_dedup() {
        with_temp_config(|| {
            record_denied_command("rm -rf /");
            record_denied_command("rm -rf /");

            let content = std::fs::read_to_string(get_deny_cache_path()).unwrap();
            let entries: Vec<String> = serde_json::from_str(&content).unwrap();
            assert_eq!(entries.len(), 1, "duplicate should not be added");
        });
    }

    #[test]
    fn test_deny_cache_miss() {
        with_temp_config(|| {
            record_denied_command("rm -rf /");
            assert!(!was_denied_command("ls -la"));
        });
    }

    #[test]
    fn test_audit_result_display() {
        assert_eq!(AuditResult::Allowed.to_string(), "ALLOWED");
        assert_eq!(AuditResult::Denied.to_string(), "DENIED");
        assert_eq!(AuditResult::Blocked.to_string(), "BLOCKED");
    }
}
