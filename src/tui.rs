use crate::config::Config;
use anyhow::Result;
use colored::Colorize;
use dialoguer::{Confirm, Input, MultiSelect};
use std::path::PathBuf;

pub async fn run_interactive_setup(mut config: Config) -> Result<Config> {
    // Display skull ASCII art
    display_skull_art();
    
    println!("{}", "[RIP-CONFIG] Interactive Setup".bright_cyan().bold().on_black());
    println!("{}", "💀 Let's configure RIP for your security scanning needs. 💀\n".bright_green().bold());

    // 1. Repository directory
    let repo_path: String = Input::new()
        .with_prompt(format!("{} {}", "🎯".bright_yellow(), "Repository directory to scan".bright_cyan().bold()))
        .default(config.repository_path.to_string_lossy().to_string())
        .interact()?;
    config.repository_path = PathBuf::from(repo_path);

    // 2. Environment file name
    let env_file: String = Input::new()
        .with_prompt(format!("{} {}", "📄".bright_yellow(), "Environment file name".bright_cyan().bold()))
        .default(config.env_filename.clone())
        .interact()?;
    config.env_filename = env_file;

    // 3. Parse .env file and select keys
    println!("\n{} {}", "🔍".bright_yellow(), "Parsing environment file...".bright_cyan());
    let available_keys = config.parse_env_file()?;

    if available_keys.is_empty() {
        println!("{} {}", "⚠️".bright_yellow(), "No environment variables found or all values are trivial".bright_red());
        println!("{} {}", "➡️".bright_green(), "Continuing with empty key list...".bright_white());
    } else {
        println!(
            "{} {} {}",
            "✅".bright_green(),
            "Found".bright_cyan(),
            format!("{} environment variables with non-trivial values:", available_keys.len()).bright_white().bold()
        );

        let selected_indices = MultiSelect::new()
            .with_prompt(format!("{} {}", "🔑".bright_yellow(), "Select environment keys to scan for".bright_cyan().bold()))
            .items(&available_keys)
            .defaults(&vec![true; available_keys.len()]) // Select all by default
            .interact()?;

        config.env_keys = selected_indices
            .iter()
            .map(|&i| available_keys[i].clone())
            .collect();

        println!("{} {} {}", "📊".bright_green(), "Selected".bright_cyan(), format!("{} keys for scanning", config.env_keys.len()).bright_white().bold());
    }

    // 4. File extensions
    println!("\n{} {}", "📁".bright_yellow(), "Configuring file extensions to scan...".bright_cyan().bold());
    let extension_defaults = vec![
        true, true, true, true, // js, ts, jsx, tsx
        true, true, true, true, // py, rb, php, java
        true, true, true, true, // go, rs, cpp, c
        true, false, false, false, // cs, yaml, yml, json
        false, false, false, // xml, md, txt
    ];

    let selected_ext_indices = MultiSelect::new()
        .with_prompt(format!("{} {}", "🔧".bright_yellow(), "Select file extensions to include in scan".bright_cyan().bold()))
        .items(&config.file_extensions)
        .defaults(&extension_defaults)
        .interact()?;

    config.file_extensions = selected_ext_indices
        .iter()
        .map(|&i| config.file_extensions[i].clone())
        .collect();

    // 5. Ignore patterns
    println!("\n{} {}", "🚫".bright_yellow(), "Configuring directories/patterns to ignore...".bright_cyan().bold());
    let ignore_defaults = vec![true; config.ignore_patterns.len()]; // All ignored by default

    let selected_ignore_indices = MultiSelect::new()
        .with_prompt(format!("{} {}", "⛔".bright_yellow(), "Select patterns to ignore during scan".bright_cyan().bold()))
        .items(&config.ignore_patterns)
        .defaults(&ignore_defaults)
        .interact()?;

    config.ignore_patterns = selected_ignore_indices
        .iter()
        .map(|&i| config.ignore_patterns[i].clone())
        .collect();

    // 6. Web scanning configuration
    println!("\n{} {}", "🌐".bright_yellow(), "Web scanning configuration...".bright_cyan().bold());
    config.enable_web_scan = Confirm::new()
        .with_prompt(format!("{} {}", "🔗".bright_yellow(), "Enable web vulnerability scanning?".bright_cyan().bold()))
        .default(false)
        .interact()?;

    if config.enable_web_scan {
        let web_url: String = Input::new()
            .with_prompt(format!("{} {}", "🌍".bright_yellow(), "Public URL for web scanning".bright_cyan().bold()))
            .allow_empty(true)
            .interact()?;

        if !web_url.is_empty() {
            config.web_url = Some(web_url);
        }
    }

    // 7. Save configuration
    println!("\n{} {}", "🎉".bright_green(), "Configuration complete!".bright_cyan().bold());
    print_config_summary(&config);

    let save_config = Confirm::new()
        .with_prompt(format!("{} {}", "💾".bright_yellow(), "Save this configuration to .ripconfig.toml?".bright_cyan().bold()))
        .default(true)
        .interact()?;

    if save_config {
        config.save_to_file(&Config::default_config_path())?;
        println!("{} {}", "✅".bright_green(), "Configuration saved successfully!".green().bold());
    }
    
    println!("{}", "═".repeat(60).bright_magenta());

    Ok(config)
}

fn print_config_summary(config: &Config) {
    println!("\n{}", "Configuration Summary:".bright_yellow().bold().on_black());
    println!("  {} {}", "Repository:".bright_cyan(), config.repository_path.display().to_string().bright_white());
    println!("  {} {}", "Environment file:".bright_cyan(), config.env_filename.bright_white());
    println!("  {} {} items", "Selected keys:".bright_cyan(), config.env_keys.len().to_string().bright_white());
    println!("  {} {} types", "File extensions:".bright_cyan(), config.file_extensions.len().to_string().bright_white());
    println!("  {} {} items", "Ignore patterns:".bright_cyan(), config.ignore_patterns.len().to_string().bright_white());
    println!(
        "  {} {}",
        "Web scanning:".bright_cyan(),
        if config.enable_web_scan {
            "enabled".bright_green()
        } else {
            "disabled".bright_red()
        }
    );

    if let Some(url) = &config.web_url {
        println!("  {} {}", "Web URL:".bright_cyan(), url.bright_white());
    }
}

fn display_skull_art() {
    use std::fs;
    
    // Try to load skull art from file
    if let Ok(skull_art) = fs::read_to_string("art/rip-skull-logo.txt") {
        println!("{skull_art}");
    } else {
        // Fallback skull ASCII art with colors
        println!("{}", r#"
                            💀 RIP CONFIGURATION 💀
        
                             (
                .            )        )
                         (  (|              .
                     )   )\/ ( ( (
             *  (   ((  /     ))\))  (  )    )
           (     \   )\(          |  ))( )  (|
           >)     ))/   |          )/  \((  ) \
           (     (      .        -.     V )/   )(    (
            \   /     .   \            .       \))   ))
              )(      (  | |   )            .    (  /
             )(    ,'))     \ /          \( `.    )
             (\>  ,'/__      ))            __`.  /
            ( \   | /  ___   ( \/     ___   \ | ( (
             \.)  |/  /   \__      __/   \   \|  ))
            .  \. |>  \      | __ |      /   <|  /
                 )/    \____/ :..: \____/     \ <
          )   \ (|__  .      / ;: \          __| )  (
         ((    )\)  ~--_     --  --      _--~    /  ))
          \    (    |  ||               ||  |   (  /
                \.  |  ||_             _||  |  /
                  > :  |  ~V+-I_I_I-+V~  |  : (.
                 (  \:  T\   _     _   /T  : ./
                  \  :    T^T T-+-T T^T    ;<
                   \..`_       -+-       _'  )
         )            . `--=.._____..=--'. ./
        
            🔓 REST IN PEACE, VULNERABILITIES 🔓
        "#.bright_red().bold());
    }
    
    println!("{}", "═".repeat(60).bright_magenta());
}
