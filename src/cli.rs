use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rip")]
#[command(about = "Rest In Peace, Vulnerabilities - A terminal-based security auditing tool")]
#[command(version = "0.2.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Automatically run with default settings, no interactive prompts
    #[arg(long)]
    pub auto: bool,

    /// Path to configuration file
    #[arg(long, value_name = "FILE", default_missing_value = ".ripconfig.toml", num_args = 0..=1)]
    pub config: Option<PathBuf>,

    /// Force reconfiguration, ignore existing config
    #[arg(long)]
    pub reconfigure: bool,

    /// Skip configuration loading/saving
    #[arg(long)]
    pub skip_config: bool,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Enable quiet output (minimal messages)
    #[arg(short, long)]
    pub quiet: bool,

    /// Output results in JSON format
    #[arg(long)]
    pub json: bool,

    /// Repository path to scan (default: current directory)
    #[arg(short, long, value_name = "PATH")]
    pub path: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan for vulnerabilities (default operation)
    Scan {
        /// Repository path to scan
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Include web scanning after local scan
        #[arg(long)]
        web: bool,

        /// Public URL for web scanning
        #[arg(long, value_name = "URL")]
        url: Option<String>,
    },

    /// Configure RIP settings interactively
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,

        /// Reset configuration to defaults
        #[arg(long)]
        reset: bool,
    },

    /// Display version information
    Version,
}

impl Default for Commands {
    fn default() -> Self {
        Commands::Scan {
            path: None,
            web: false,
            url: None,
        }
    }
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
