use crate::config::Config;
use anyhow::Result;
use colored::Colorize;
use dialoguer::{Confirm, Input, MultiSelect};
use std::path::PathBuf;

pub async fn run_interactive_setup(mut config: Config) -> Result<Config> {
    println!("{}", "[RIP-CONFIG] Interactive Setup".bright_cyan().bold());
    println!("Let's configure RIP for your security scanning needs.\n");

    // 1. Repository directory
    let repo_path: String = Input::new()
        .with_prompt("Repository directory to scan")
        .default(config.repository_path.to_string_lossy().to_string())
        .interact()?;
    config.repository_path = PathBuf::from(repo_path);

    // 2. Environment file name
    let env_file: String = Input::new()
        .with_prompt("Environment file name")
        .default(config.env_filename.clone())
        .interact()?;
    config.env_filename = env_file;

    // 3. Parse .env file and select keys
    println!("\nParsing environment file...");
    let available_keys = config.parse_env_file()?;

    if available_keys.is_empty() {
        println!("No environment variables found or all values are trivial");
        println!("Continuing with empty key list...");
    } else {
        println!(
            "Found {} environment variables with non-trivial values:",
            available_keys.len()
        );

        let selected_indices = MultiSelect::new()
            .with_prompt("Select environment keys to scan for")
            .items(&available_keys)
            .defaults(&vec![true; available_keys.len()]) // Select all by default
            .interact()?;

        config.env_keys = selected_indices
            .iter()
            .map(|&i| available_keys[i].clone())
            .collect();

        println!("Selected {} keys for scanning", config.env_keys.len());
    }

    // 4. File extensions
    println!("\nConfiguring file extensions to scan...");
    let extension_defaults = vec![
        true, true, true, true, // js, ts, jsx, tsx
        true, true, true, true, // py, rb, php, java
        true, true, true, true, // go, rs, cpp, c
        true, false, false, false, // cs, yaml, yml, json
        false, false, false, // xml, md, txt
    ];

    let selected_ext_indices = MultiSelect::new()
        .with_prompt("Select file extensions to include in scan")
        .items(&config.file_extensions)
        .defaults(&extension_defaults)
        .interact()?;

    config.file_extensions = selected_ext_indices
        .iter()
        .map(|&i| config.file_extensions[i].clone())
        .collect();

    // 5. Ignore patterns
    println!("\nConfiguring directories/patterns to ignore...");
    let ignore_defaults = vec![true; config.ignore_patterns.len()]; // All ignored by default

    let selected_ignore_indices = MultiSelect::new()
        .with_prompt("Select patterns to ignore during scan")
        .items(&config.ignore_patterns)
        .defaults(&ignore_defaults)
        .interact()?;

    config.ignore_patterns = selected_ignore_indices
        .iter()
        .map(|&i| config.ignore_patterns[i].clone())
        .collect();

    // 6. Web scanning configuration
    println!("\nWeb scanning configuration...");
    config.enable_web_scan = Confirm::new()
        .with_prompt("Enable web vulnerability scanning?")
        .default(false)
        .interact()?;

    if config.enable_web_scan {
        let web_url: String = Input::new()
            .with_prompt("Public URL for web scanning")
            .allow_empty(true)
            .interact()?;

        if !web_url.is_empty() {
            config.web_url = Some(web_url);
        }
    }

    // 7. Save configuration
    println!("\nConfiguration complete!");
    print_config_summary(&config);

    let save_config = Confirm::new()
        .with_prompt("Save this configuration to .ripconfig.toml?")
        .default(true)
        .interact()?;

    if save_config {
        config.save_to_file(&Config::default_config_path())?;
        println!("{}", "Configuration saved successfully!".green());
    }

    Ok(config)
}

fn print_config_summary(config: &Config) {
    println!("\n{}", "Configuration Summary:".bright_yellow().bold());
    println!("  Repository: {}", config.repository_path.display());
    println!("  Environment file: {}", config.env_filename);
    println!("  Selected keys: {} items", config.env_keys.len());
    println!("  File extensions: {} types", config.file_extensions.len());
    println!("  Ignore patterns: {} items", config.ignore_patterns.len());
    println!(
        "  Web scanning: {}",
        if config.enable_web_scan {
            "enabled"
        } else {
            "disabled"
        }
    );

    if let Some(url) = &config.web_url {
        println!("  Web URL: {}", url);
    }
}
