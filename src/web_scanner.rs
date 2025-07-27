use anyhow::Result;
use colored::Colorize;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct WebVulnerability {
    pub url: String,
    pub vulnerability_type: String,
    pub severity: Severity,
    pub description: String,
    pub recommendation: String,
}

#[derive(Debug, Clone)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    fn color(&self) -> &str {
        match self {
            Severity::Low => "green",
            Severity::Medium => "yellow", 
            Severity::High => "red",
            Severity::Critical => "bright_red",
        }
    }
}

#[derive(Debug)]
pub struct WebScanResults {
    pub vulnerabilities: Vec<WebVulnerability>,
    pub base_url: String,
    pub endpoints_checked: usize,
}

pub async fn scan_web(base_url: &str) -> Result<WebScanResults> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("RIP-Scanner/0.1.0")
        .build()?;

    let mut vulnerabilities = Vec::new();
    let mut endpoints_checked = 0;

    println!("Scanning web endpoints...");

    // Common sensitive file paths to check
    let sensitive_paths = vec![
        "/.env",
        "/.git/config", 
        "/.git/HEAD",
        "/backup.sql",
        "/database.sql",
        "/config.php",
        "/wp-config.php",
        "/admin",
        "/phpmyadmin",
        "/debug",
        "/test",
        "/.htaccess",
        "/robots.txt",
        "/sitemap.xml",
        "/crossdomain.xml",
        "/clientaccesspolicy.xml",
    ];

    // Check each sensitive path
    for path in &sensitive_paths {
        let url = format!("{}{}", base_url.trim_end_matches('/'), path);
        endpoints_checked += 1;
        
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let severity = match path {
                        p if p.contains(".env") || p.contains(".git") || p.contains("config") => Severity::Critical,
                        p if p.contains("backup") || p.contains("database") || p.contains("admin") => Severity::High,
                        p if p.contains("debug") || p.contains("test") => Severity::Medium,
                        _ => Severity::Low,
                    };

                    vulnerabilities.push(WebVulnerability {
                        url: url.clone(),
                        vulnerability_type: "Exposed Sensitive File".to_string(),
                        severity,
                        description: format!("Sensitive file accessible at {}", path),
                        recommendation: "Remove or restrict access to this file".to_string(),
                    });
                }
            }
            Err(_) => {
                // Connection errors are expected for most paths
                continue;
            }
        }
    }

    // Check robots.txt for information disclosure
    let robots_url = format!("{}/robots.txt", base_url.trim_end_matches('/'));
    if let Ok(response) = client.get(&robots_url).send().await {
        if response.status().is_success() {
            if let Ok(content) = response.text().await {
                if content.lines().count() > 10 || content.contains("admin") || content.contains("private") {
                    vulnerabilities.push(WebVulnerability {
                        url: robots_url,
                        vulnerability_type: "Information Disclosure".to_string(),
                        severity: Severity::Low,
                        description: "Robots.txt reveals potentially sensitive directory structure".to_string(),
                        recommendation: "Review robots.txt for sensitive path disclosure".to_string(),
                    });
                }
            }
        }
    }

    // Check security headers
    if let Ok(response) = client.get(base_url).send().await {
        let headers = response.headers();
        check_security_headers(headers, base_url, &mut vulnerabilities);
    }

    Ok(WebScanResults {
        vulnerabilities,
        base_url: base_url.to_string(),
        endpoints_checked,
    })
}

fn check_security_headers(
    headers: &reqwest::header::HeaderMap,
    base_url: &str,
    vulnerabilities: &mut Vec<WebVulnerability>,
) {
    // Check for missing security headers
    let security_headers = vec![
        ("x-frame-options", "Clickjacking protection"),
        ("x-content-type-options", "MIME type sniffing protection"),
        ("x-xss-protection", "XSS protection"),
        ("strict-transport-security", "HTTPS enforcement"),
        ("content-security-policy", "Content Security Policy"),
        ("referrer-policy", "Referrer information control"),
    ];

    for (header_name, description) in security_headers {
        if !headers.contains_key(header_name) {
            vulnerabilities.push(WebVulnerability {
                url: base_url.to_string(),
                vulnerability_type: "Missing Security Header".to_string(),
                severity: if header_name == "strict-transport-security" { 
                    Severity::High 
                } else { 
                    Severity::Medium 
                },
                description: format!("Missing {} header", header_name),
                recommendation: format!("Add {} header for {}", header_name, description),
            });
        }
    }

    // Check for insecure header values
    if let Some(xfo) = headers.get("x-frame-options") {
        if let Ok(value) = xfo.to_str() {
            if value.to_lowercase() == "allowall" {
                vulnerabilities.push(WebVulnerability {
                    url: base_url.to_string(),
                    vulnerability_type: "Insecure Header Configuration".to_string(),
                    severity: Severity::High,
                    description: "X-Frame-Options set to ALLOWALL".to_string(),
                    recommendation: "Set X-Frame-Options to DENY or SAMEORIGIN".to_string(),
                });
            }
        }
    }
}

pub fn display_web_results(results: &WebScanResults) {
    println!("\n{}", "Web Scan Results".bright_blue().bold());
    println!("Base URL: {}", results.base_url);
    println!("Endpoints checked: {}", results.endpoints_checked);

    if results.vulnerabilities.is_empty() {
        println!("{}", "No web vulnerabilities found!".green().bold());
        return;
    }

    println!("{}", format!("Found {} web vulnerabilities:", results.vulnerabilities.len()).red().bold());

    // Group by severity
    let mut by_severity: HashMap<String, Vec<&WebVulnerability>> = HashMap::new();
    for vuln in &results.vulnerabilities {
        let severity_key = format!("{:?}", vuln.severity);
        by_severity.entry(severity_key).or_default().push(vuln);
    }

    // Display in order of severity
    for severity in &["Critical", "High", "Medium", "Low"] {
        if let Some(vulns) = by_severity.get(&severity.to_string()) {
            println!("\n{} {}:", 
                "o".color(match *severity {
                    "Critical" => "bright_red",
                    "High" => "red", 
                    "Medium" => "yellow",
                    "Low" => "green",
                    _ => "white",
                }),
                severity.color(match *severity {
                    "Critical" => "bright_red",
                    "High" => "red",
                    "Medium" => "yellow", 
                    "Low" => "green",
                    _ => "white",
                }).bold()
            );

            for vuln in vulns {
                println!("  {} {}", "Type:".bright_black(), vuln.vulnerability_type);
                println!("  {} {}", "URL:".bright_black(), vuln.url.bright_blue());
                println!("  {} {}", "Issue:".bright_black(), vuln.description);
                println!("  {} {}", "Fix:".bright_black(), vuln.recommendation.green());
                println!();
            }
        }
    }

    println!("{}", "Web Security Recommendation: Address critical and high severity issues first.".bright_yellow());
}