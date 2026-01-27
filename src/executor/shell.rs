use std::process::{Command, ExitStatus, Stdio};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecError {
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(#[from] std::io::Error),
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
    fn test_execute_with_exit_code() {
        let executor = ShellExecutor::new();
        let status = executor.execute("true").unwrap();
        assert!(status.success());
    }
}
