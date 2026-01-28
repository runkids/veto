use clap::{Args, Parser, Subcommand};
use crate::commands::AuthCommands;

#[derive(Parser)]
#[command(name = "veto")]
#[command(about = "AI operation guardian - verify before execute")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress output (exit code only)
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check command risk level (no execute)
    Check {
        /// Command to check
        command: String,
    },
    /// Verify and execute command
    Exec {
        /// Command to execute
        command: String,

        /// Override authentication method
        #[arg(long)]
        auth: Option<String>,
    },
    /// Gate command (verify only, no execute) - for use in hooks
    Gate(GateArgs),
    /// Initialize config files
    Init {
        /// Overwrite existing config
        #[arg(long)]
        force: bool,
    },
    /// Verify installation and config
    Doctor,
    /// Start interactive shell wrapper
    Shell,
    /// Manage authentication methods
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Setup integrations with AI tools
    Setup {
        #[command(subcommand)]
        command: SetupCommands,
    },
    /// Upgrade veto to the latest version
    Upgrade {
        /// Only check for updates, don't install
        #[arg(long)]
        check: bool,
        /// Force reinstall even if already on latest version
        #[arg(long)]
        force: bool,
    },
    /// View audit log
    Log(LogArgs),
}

#[derive(Args)]
pub struct LogArgs {
    /// Show last N entries
    #[arg(short = 'n', long)]
    pub tail: Option<usize>,

    /// Follow log in real-time
    #[arg(short, long)]
    pub follow: bool,

    /// Filter by result: ALLOWED, DENIED, or BLOCKED
    #[arg(long)]
    pub filter: Option<String>,

    /// Clear the audit log
    #[arg(long)]
    pub clear: bool,
}

#[derive(Args)]
pub struct GateArgs {
    /// Command to verify (optional if using --claude/--gemini)
    pub command: Option<String>,

    /// Read command from Claude Code stdin JSON format
    #[arg(long, conflicts_with_all = ["command", "opencode", "gemini"])]
    pub claude: bool,

    /// Read command from Gemini CLI stdin JSON format
    #[arg(long, conflicts_with_all = ["command", "opencode", "claude"])]
    pub gemini: bool,

    /// OpenCode mode - use dialog/touchid auth, don't suggest env var bypass
    #[arg(long, conflicts_with_all = ["claude", "gemini"])]
    pub opencode: bool,

    /// Override authentication method
    #[arg(long)]
    pub auth: Option<String>,

    /// TOTP code for verification
    #[arg(long)]
    pub totp: Option<String>,

    /// PIN for verification
    #[arg(long)]
    pub pin: Option<String>,
}

#[derive(Subcommand)]
pub enum SetupCommands {
    /// Setup Claude Code hooks integration
    Claude {
        /// Remove veto hooks from Claude Code
        #[arg(long)]
        uninstall: bool,
    },
    /// Setup Gemini CLI hooks integration
    Gemini {
        /// Remove veto hooks from Gemini CLI
        #[arg(long)]
        uninstall: bool,
    },
    /// Setup OpenCode plugin integration
    Opencode {
        /// Remove veto plugin from OpenCode
        #[arg(long)]
        uninstall: bool,
    },
}
