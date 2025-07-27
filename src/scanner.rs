use crate::config::Config;
use anyhow::Result;
use colored::Colorize;
use grep_regex::RegexMatcher;
use grep_searcher::{SearcherBuilder, Sink, SinkMatch};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct ScanMatch {
    pub file_path: PathBuf,
    pub line_number: u64,
    pub line_content: String,
    #[allow(dead_code)]
    pub matched_text: String,
    pub env_key: String,
}

#[derive(Debug)]
pub struct ScanResults {
    pub matches: Vec<ScanMatch>,
    pub files_scanned: usize,
    pub scan_duration: std::time::Duration,
    pub log_file_path: PathBuf,
}

pub fn scan_files(config: &Config) -> Result<ScanResults> {
    let start_time = SystemTime::now();

    // Get environment values to search for
    let env_values = config.get_env_values()?;
    
    // If no env values, scan for common secret patterns
    let (search_patterns, search_keys) = if env_values.is_empty() {
        get_common_secret_patterns()
    } else {
        (env_values, config.env_keys.clone())
    };

    println!(
        "Scanning for {} secret patterns...",
        search_patterns.len()
    );

    // Create progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap(),
    );
    pb.set_message("Scanning files...");

    // Collect all files to scan
    let files_to_scan = collect_files(
        &config.repository_path,
        &config.file_extensions,
        &config.ignore_patterns,
    )?;
    let total_files = files_to_scan.len();

    pb.set_length(total_files as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} {pos:>7}/{len:7} [{elapsed_precise}] {msg}")
            .unwrap(),
    );

    let mut all_matches = Vec::new();
    let mut files_scanned = 0;

    // Scan each file
    for file_path in files_to_scan {
        pb.set_message(format!(
            "Scanning: {}",
            file_path.file_name().unwrap_or_default().to_string_lossy()
        ));

        // Scan for each pattern
        for (env_key, pattern) in search_keys.iter().zip(search_patterns.iter()) {
            if let Ok(matches) = scan_file_for_pattern(&file_path, pattern, env_key) {
                all_matches.extend(matches);
            }
        }

        files_scanned += 1;
        pb.inc(1);
    }

    pb.finish_with_message("Scan complete!");

    let scan_duration = start_time.elapsed()?;
    let log_file_path = create_log_file(&all_matches)?;

    Ok(ScanResults {
        matches: all_matches,
        files_scanned,
        scan_duration,
        log_file_path,
    })
}

fn collect_files(
    root_path: &Path,
    extensions: &[String],
    ignore_patterns: &[String],
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_recursive(root_path, extensions, ignore_patterns, &mut files)?;
    Ok(files)
}

fn collect_files_recursive(
    dir: &Path,
    extensions: &[String],
    ignore_patterns: &[String],
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    if should_ignore_path(dir, ignore_patterns) {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_files_recursive(&path, extensions, ignore_patterns, files)?;
        } else if path.is_file() {
            if let Some(extension) = path.extension() {
                if let Some(ext_str) = extension.to_str() {
                    if extensions.contains(&ext_str.to_string()) {
                        files.push(path);
                    }
                }
            }
        }
    }
    Ok(())
}

fn should_ignore_path(path: &Path, ignore_patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();

    for pattern in ignore_patterns {
        if pattern.contains('*') {
            // Simple glob matching
            if let Some(ext) = pattern.strip_prefix("*.") {
                if let Some(file_ext) = path.extension() {
                    if file_ext == ext {
                        return true;
                    }
                }
            }
        } else if path_str.contains(pattern) {
            return true;
        }
    }
    false
}

fn get_common_secret_patterns() -> (Vec<String>, Vec<String>) {
    let patterns = vec![
        // Common API keys and tokens
        "sk_live_".to_string(),
        "sk_test_".to_string(),
        "pk_live_".to_string(),
        "pk_test_".to_string(),
        "Bearer ".to_string(),
        "ghp_".to_string(),        // GitHub personal access tokens
        "gho_".to_string(),        // GitHub OAuth tokens
        "ghu_".to_string(),        // GitHub user tokens
        "ghs_".to_string(),        // GitHub server tokens
        "ghr_".to_string(),        // GitHub refresh tokens
        "xoxb-".to_string(),       // Slack bot tokens
        "xoxp-".to_string(),       // Slack user tokens
        "AKIA".to_string(),        // AWS access keys
        "ASIA".to_string(),        // AWS temporary access keys
        "ya29.".to_string(),       // Google OAuth tokens
        "AIza".to_string(),        // Google API keys
    ];
    
    let keys = vec![
        "STRIPE_SECRET_KEY".to_string(),
        "STRIPE_TEST_KEY".to_string(),
        "STRIPE_PUBLIC_KEY".to_string(),
        "STRIPE_TEST_PUBLIC_KEY".to_string(),
        "BEARER_TOKEN".to_string(),
        "GITHUB_TOKEN".to_string(),
        "GITHUB_OAUTH_TOKEN".to_string(),
        "GITHUB_USER_TOKEN".to_string(),
        "GITHUB_SERVER_TOKEN".to_string(),
        "GITHUB_REFRESH_TOKEN".to_string(),
        "SLACK_BOT_TOKEN".to_string(),
        "SLACK_USER_TOKEN".to_string(),
        "AWS_ACCESS_KEY_ID".to_string(),
        "AWS_TEMP_ACCESS_KEY_ID".to_string(),
        "GOOGLE_OAUTH_TOKEN".to_string(),
        "GOOGLE_API_KEY".to_string(),
    ];
    
    (patterns, keys)
}

fn scan_file_for_pattern(file_path: &Path, pattern: &str, env_key: &str) -> Result<Vec<ScanMatch>> {
    // Skip scanning the .env file itself to avoid self-references
    if file_path.file_name().and_then(|n| n.to_str()) == Some(".env") {
        return Ok(Vec::new());
    }
    
    let matcher = RegexMatcher::new_line_matcher(&regex::escape(pattern))?;
    let mut searcher = SearcherBuilder::new().line_number(true).build();
    let mut matches = Vec::new();

    let mut sink = ScanSink::new(file_path, env_key, &mut matches);
    searcher.search_path(&matcher, file_path, &mut sink)?;

    Ok(matches)
}

struct ScanSink<'a> {
    file_path: &'a Path,
    env_key: &'a str,
    matches: &'a mut Vec<ScanMatch>,
}

impl<'a> ScanSink<'a> {
    fn new(file_path: &'a Path, env_key: &'a str, matches: &'a mut Vec<ScanMatch>) -> Self {
        Self {
            file_path,
            env_key,
            matches,
        }
    }
}

impl<'a> Sink for ScanSink<'a> {
    type Error = io::Error;

    fn matched(
        &mut self,
        _searcher: &grep_searcher::Searcher,
        mat: &SinkMatch<'_>,
    ) -> Result<bool, Self::Error> {
        let line_content = String::from_utf8_lossy(mat.bytes()).to_string();
        let matched_text = String::from_utf8_lossy(mat.bytes()).to_string();

        self.matches.push(ScanMatch {
            file_path: self.file_path.to_path_buf(),
            line_number: mat.line_number().unwrap_or(0),
            line_content,
            matched_text,
            env_key: self.env_key.to_string(),
        });

        Ok(true)
    }
}

fn create_log_file(matches: &[ScanMatch]) -> Result<PathBuf> {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let log_path = PathBuf::from(format!("/tmp/rip-{timestamp}.log"));

    let mut file = fs::File::create(&log_path)?;

    writeln!(file, "RIP Vulnerability Scan Results")?;
    writeln!(
        file,
        "Generated: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(file, "Total matches: {}\n", matches.len())?;
    writeln!(
        file,
        "WARNING: This file contains potentially sensitive information."
    )?;
    writeln!(
        file,
        "Ensure proper handling and secure deletion after review.\n"
    )?;

    for scan_match in matches {
        let masked_content = mask_sensitive_content(&scan_match.line_content);
        writeln!(
            file,
            "File: {}\nLine: {}\nKey: {}\nContent: {}\n---",
            scan_match.file_path.display(),
            scan_match.line_number,
            scan_match.env_key,
            masked_content.trim()
        )?;
    }

    Ok(log_path)
}

fn mask_sensitive_content(content: &str) -> String {
    // Common patterns for sensitive values
    let patterns = vec![
        (r"(api[_-]?key\s*[=:]\s*)([^,\s\n]+)", "${1}***MASKED***"),
        (r"(secret[_-]?key\s*[=:]\s*)([^,\s\n]+)", "${1}***MASKED***"),
        (r"(password\s*[=:]\s*)([^,\s\n]+)", "${1}***MASKED***"),
        (r"(token\s*[=:]\s*)([^,\s\n]+)", "${1}***MASKED***"),
        (r"(bearer\s+)([^\s,]+)", "${1}***MASKED***"),
        (r"(sk_[a-zA-Z0-9_-]+)", "***MASKED***"),
        (r"(pk_[a-zA-Z0-9_-]+)", "***MASKED***"),
        (r"([a-fA-F0-9]{32,})", "***MASKED***"), // Long hex strings
    ];

    let mut masked = content.to_string();
    for (pattern, replacement) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            masked = re.replace_all(&masked, replacement).to_string();
        }
    }

    masked
}

pub fn display_results(results: &ScanResults) {
    println!("\nScan Results");
    println!("Files scanned: {}", results.files_scanned);
    println!("Scan duration: {:.2?}", results.scan_duration);
    println!("Log file: {}", results.log_file_path.display());

    if results.matches.is_empty() {
        println!("{}", "No vulnerabilities found!".green().bold());
        return;
    }

    println!(
        "{}",
        format!("Found {} potential vulnerabilities:", results.matches.len())
            .red()
            .bold()
    );

    // Group matches by file
    let mut matches_by_file: HashMap<&PathBuf, Vec<&ScanMatch>> = HashMap::new();
    for scan_match in &results.matches {
        matches_by_file
            .entry(&scan_match.file_path)
            .or_default()
            .push(scan_match);
    }

    for (file_path, file_matches) in matches_by_file {
        println!("\nFile: {}", file_path.display().to_string().bright_blue());

        for scan_match in file_matches {
            println!(
                "  {}:{} {} {}",
                "Line".bright_black(),
                scan_match.line_number.to_string().yellow(),
                "Key:".bright_black(),
                scan_match.env_key.red().bold()
            );

            // Show the line content with masking for sensitive data
            let masked_content = mask_sensitive_content(&scan_match.line_content);
            let trimmed_content = masked_content.trim();
            if trimmed_content.len() > 100 {
                println!("    {}...", &trimmed_content[..100]);
            } else {
                println!("    {trimmed_content}");
            }
        }
    }

    println!(
        "\n{}",
        "Recommendation: Review these files to ensure secrets are not exposed.".bright_yellow()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_should_ignore_path() {
        let ignore_patterns = vec![
            "node_modules".to_string(),
            "*.log".to_string(),
            ".git".to_string(),
        ];

        assert!(should_ignore_path(
            Path::new("node_modules"),
            &ignore_patterns
        ));
        assert!(should_ignore_path(
            Path::new("src/node_modules"),
            &ignore_patterns
        ));
        assert!(should_ignore_path(Path::new("test.log"), &ignore_patterns));
        assert!(should_ignore_path(Path::new(".git"), &ignore_patterns));

        assert!(!should_ignore_path(Path::new("src"), &ignore_patterns));
        assert!(!should_ignore_path(Path::new("test.js"), &ignore_patterns));
    }

    #[test]
    fn test_collect_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create test file structure
        fs::create_dir_all(root.join("src"))?;
        fs::create_dir_all(root.join("node_modules"))?;
        fs::write(root.join("src/main.js"), "console.log('hello');")?;
        fs::write(root.join("src/config.ts"), "export const config = {};")?;
        fs::write(root.join("README.md"), "# Project")?;
        fs::write(root.join("node_modules/package.json"), "{}")?;

        let extensions = vec!["js".to_string(), "ts".to_string()];
        let ignore_patterns = vec!["node_modules".to_string()];

        let files = collect_files(root, &extensions, &ignore_patterns)?;

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.file_name().unwrap() == "main.js"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "config.ts"));
        assert!(!files
            .iter()
            .any(|f| f.to_string_lossy().contains("node_modules")));

        Ok(())
    }

    #[test]
    fn test_scan_file_for_pattern() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.js");

        let content = "const api_key = 'secret123';\nconst other = 'value';\napi_key: 'secret123'";
        fs::write(&file_path, content)?;

        let matches = scan_file_for_pattern(&file_path, "secret123", "API_KEY")?;

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].env_key, "API_KEY");
        assert_eq!(matches[0].line_number, 1);
        assert!(matches[0].line_content.contains("secret123"));

        Ok(())
    }

    #[test]
    fn test_mask_sensitive_content() {
        let test_cases = vec![
            ("api_key=sk_live_1234567890", "api_key=***MASKED***"),
            ("secret_key: bearer_token", "secret_key: ***MASKED***"),
            ("password=mysecret", "password=***MASKED***"),
            ("token = abc123def456", "token = ***MASKED***"),
            ("normal text", "normal text"),
            ("API_KEY=sk_test_abcdef123456", "API_KEY=***MASKED***"),
            ("pk_live_1234567890abcdef", "***MASKED***"),
            ("0123456789abcdef0123456789abcdef", "***MASKED***"), // 32 char hex
        ];

        for (input, expected) in test_cases {
            let result = mask_sensitive_content(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_create_log_file() -> Result<()> {
        let matches = vec![ScanMatch {
            file_path: PathBuf::from("test.js"),
            line_number: 1,
            line_content: "api_key=secret123".to_string(),
            matched_text: "secret123".to_string(),
            env_key: "API_KEY".to_string(),
        }];

        let log_path = create_log_file(&matches)?;
        assert!(log_path.exists());

        let log_content = fs::read_to_string(&log_path)?;
        assert!(log_content.contains("RIP Vulnerability Scan Results"));
        assert!(
            log_content.contains("WARNING: This file contains potentially sensitive information")
        );
        assert!(log_content.contains("API_KEY"));
        assert!(log_content.contains("***MASKED***"));

        // Clean up
        fs::remove_file(&log_path)?;

        Ok(())
    }

    #[test]
    fn test_scan_files_empty_env_values() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut config = Config::default();
        config.repository_path = temp_dir.path().to_path_buf();

        // No .env file, so no env values
        let results = scan_files(&config)?;

        assert_eq!(results.matches.len(), 0);
        assert_eq!(results.files_scanned, 0);

        Ok(())
    }
}
