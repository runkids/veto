use std::process::{Command, ExitStatus, Stdio};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecError {
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(#[from] std::io::Error),
    #[error("Process terminated with non-zero exit code: {0}")]
    NonZeroExit(i32),
}

pub struct ShellExecutor {
    shell: String,
}

impl ShellExecutor {
    pub fn new() -> Self {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        Self { shell }
    }

    pub fn execute(&self, command: &str) -> Result<ExitStatus, ExecError> {
        let status = Command::new(&self.shell)
            .arg("-c")
            .arg(command)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        Ok(status)
    }

    pub fn execute_capture(&self, command: &str) -> Result<String, ExecError> {
        let output = Command::new(&self.shell)
            .arg("-c")
            .arg(command)
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl Default for ShellExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_simple_command() {
        let executor = ShellExecutor::new();
        let output = executor.execute_capture("echo hello").unwrap();
        assert_eq!(output.trim(), "hello");
    }

    #[test]
    fn test_execute_with_exit_code() {
        let executor = ShellExecutor::new();
        let status = executor.execute("true").unwrap();
        assert!(status.success());
    }
}
