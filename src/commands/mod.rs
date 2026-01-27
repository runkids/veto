mod init;
mod doctor;
mod auth;
mod shell;
mod setup;
mod upgrade;

pub use init::*;
pub use doctor::*;
pub use auth::*;
pub use shell::run_shell;
pub use setup::{run_setup_claude, is_claude_configured};
pub use upgrade::run_upgrade;
