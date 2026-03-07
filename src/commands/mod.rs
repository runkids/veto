pub mod allow;
mod auth;
mod doctor;
mod init;
mod log;
mod setup;
mod shell;
mod upgrade;

pub use auth::*;
pub use doctor::*;
pub use init::*;
pub use log::run_log;
pub use setup::{
    is_claude_configured, is_cursor_configured, is_gemini_configured, run_setup_claude,
    run_setup_cursor, run_setup_gemini, run_setup_opencode,
};
pub use shell::run_shell;
pub use upgrade::run_upgrade;
