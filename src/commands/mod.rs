mod init;
mod doctor;
mod auth;
mod shell;
mod setup;
mod upgrade;
mod log;

pub use init::*;
pub use doctor::*;
pub use auth::*;
pub use shell::run_shell;
pub use setup::{run_setup_claude, run_setup_opencode, is_claude_configured, is_opencode_configured};
pub use upgrade::run_upgrade;
pub use log::run_log;
