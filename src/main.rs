mod cli;
mod config;
mod scanner;
mod tui;
mod web_scanner;

use anyhow::Result;
use cli::{parse_args, Commands};
use config::Config;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_args();

    // Load or create configuration
    let mut config = load_config(&args).await?;

    // Override config with CLI arguments
    if let Some(path) = &args.path {
        config.repository_path = path.clone();
    }

    // Handle different commands
    match args.command.unwrap_or_default() {
        Commands::Scan { path, web, url } => {
            if let Some(scan_path) = path {
                config.repository_path = scan_path;
            }
            if web {
                config.enable_web_scan = true;
            }
            if let Some(scan_url) = url {
                config.web_url = Some(scan_url);
            }

            run_scan(&config).await?;
        }
        Commands::Config { show, reset } => {
            if reset {
                config = Config::default();
                config.save_to_file(&Config::default_config_path())?;
                println!("Configuration reset to defaults");
            } else if show {
                println!("{}", toml::to_string_pretty(&config)?);
            } else {
                tui::run_interactive_setup(config).await?;
            }
        }
        Commands::Version => {
            println!(
                "RIP (Rest In Peace, Vulnerabilities) v{}",
                env!("CARGO_PKG_VERSION")
            );
        }
    }

    Ok(())
}

async fn load_config(args: &cli::Cli) -> Result<Config> {
    // Skip config loading if requested
    if args.skip_config {
        return Ok(Config::default());
    }

    // Use specified config file or default
    let config_path = args
        .config
        .clone()
        .unwrap_or_else(Config::default_config_path);

    // Force reconfiguration if requested
    if args.reconfigure {
        let config = Config::default();
        return if args.auto {
            Ok(config)
        } else {
            tui::run_interactive_setup(config).await
        };
    }

    // Try to load existing config
    if config_path.exists() {
        match Config::load_from_file(&config_path) {
            Ok(config) => Ok(config),
            Err(_) => {
                eprintln!("Warning: Failed to load config file, using defaults");
                Ok(Config::default())
            }
        }
    } else if args.auto {
        // Auto mode with no config - use defaults
        Ok(Config::default())
    } else {
        // No config and not auto mode - run interactive setup
        tui::run_interactive_setup(Config::default()).await
    }
}

async fn run_scan(config: &Config) -> Result<()> {
    // Display ASCII art
    display_logo();

    println!("[RIP-SCAN] Starting vulnerability scan...");
    println!("Scanning path: {}", config.repository_path.display());

    // Run local file scan
    let scan_results = scanner::scan_files(config)?;

    // Display results
    scanner::display_results(&scan_results);

    // Run web scan if enabled
    if config.enable_web_scan {
        if let Some(url) = &config.web_url {
            println!("\n[RIP-WEB] Starting web vulnerability scan...");
            let web_results = web_scanner::scan_web(url).await?;
            web_scanner::display_web_results(&web_results);
        } else {
            println!("Web scan enabled but no URL provided");
        }
    }

    println!("\n[RIP-SCAN] Scan complete!");
    Ok(())
}

fn display_logo() {
    // Try to load and display ASCII art
    if let Ok(logo) = fs::read_to_string("art/rip-logo.txt") {
        println!("{logo}");
    } else if let Ok(skull_logo) = fs::read_to_string("art/rip-skull-logo.txt") {
        println!("{skull_logo}");
    } else {
        // Fallback ASCII art
        println!(
            r#"
 ██▀███   ██▓ ██▓███  
▓██ ▒ ██▒▓██▒▓██░  ██▒
▓██ ░▄█ ▒▒██▒▓██░ ██▓▒
▒██▀▀█▄  ░██░▒██▄█▓▒ ▒
░██▓ ▒██▒░██░▒██▒ ░  ░
░ ▒▓ ░▒▓░░▓  ▒▓▒░ ░  ░
  ░▒ ░ ▒░ ▒ ░░▒ ░     
  ░░   ░  ▒ ░░░       
   ░      ░           
                      
Rest In Peace, Vulnerabilities
"#
        );
    }
}
