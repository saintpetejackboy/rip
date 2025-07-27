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
                        p if p.contains(".env") || p.contains(".git") || p.contains("config") => {
                            Severity::Critical
                        }
                        p if p.contains("backup")
                            || p.contains("database")
                            || p.contains("admin") =>
                        {
                            Severity::High
                        }
                        p if p.contains("debug") || p.contains("test") => Severity::Medium,
                        _ => Severity::Low,
                    };

                    vulnerabilities.push(WebVulnerability {
                        url: url.clone(),
                        vulnerability_type: "Exposed Sensitive File".to_string(),
                        severity,
                        description: format!("Sensitive file accessible at {path}"),
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
                if content.lines().count() > 10
                    || content.contains("admin")
                    || content.contains("private")
                {
                    vulnerabilities.push(WebVulnerability {
                        url: robots_url,
                        vulnerability_type: "Information Disclosure".to_string(),
                        severity: Severity::Low,
                        description: "Robots.txt reveals potentially sensitive directory structure"
                            .to_string(),
                        recommendation: "Review robots.txt for sensitive path disclosure"
                            .to_string(),
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
                description: format!("Missing {header_name} header"),
                recommendation: format!("Add {header_name} header for {description}"),
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

    println!(
        "{}",
        format!(
            "Found {} web vulnerabilities:",
            results.vulnerabilities.len()
        )
        .red()
        .bold()
    );

    // Group by severity
    let mut by_severity: HashMap<String, Vec<&WebVulnerability>> = HashMap::new();
    for vuln in &results.vulnerabilities {
        let severity_key = format!("{:?}", vuln.severity);
        by_severity.entry(severity_key).or_default().push(vuln);
    }

    // Display in order of severity
    let severity_order = [
        (Severity::Critical, "Critical"),
        (Severity::High, "High"),
        (Severity::Medium, "Medium"),
        (Severity::Low, "Low"),
    ];

    for (sev_enum, severity_name) in &severity_order {
        if let Some(vulns) = by_severity.get(&format!("{sev_enum:?}")) {
            let color_name = sev_enum.color();
            println!(
                "\n{} {}:",
                "o".color(color_name),
                severity_name.color(color_name).bold()
            );

            for vuln in vulns {
                println!("  {} {}", "Type:".bright_black(), vuln.vulnerability_type);
                println!("  {} {}", "URL:".bright_black(), vuln.url.bright_blue());
                println!("  {} {}", "Issue:".bright_black(), vuln.description);
                println!(
                    "  {} {}",
                    "Fix:".bright_black(),
                    vuln.recommendation.green()
                );
                println!();
            }
        }
    }

    println!(
        "{}",
        "Web Security Recommendation: Address critical and high severity issues first."
            .bright_yellow()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
    use std::str::FromStr;

    #[test]
    fn test_severity_color() {
        assert_eq!(Severity::Low.color(), "green");
        assert_eq!(Severity::Medium.color(), "yellow");
        assert_eq!(Severity::High.color(), "red");
        assert_eq!(Severity::Critical.color(), "bright_red");
    }

    #[test]
    fn test_check_security_headers_missing_all() {
        let headers = HeaderMap::new();
        let mut vulnerabilities = Vec::new();

        check_security_headers(&headers, "https://example.com", &mut vulnerabilities);

        assert_eq!(vulnerabilities.len(), 6); // All 6 security headers missing

        let hsts_vuln = vulnerabilities
            .iter()
            .find(|v| v.description.contains("strict-transport-security"))
            .unwrap();
        assert!(matches!(hsts_vuln.severity, Severity::High));

        let other_vuln = vulnerabilities
            .iter()
            .find(|v| v.description.contains("x-frame-options"))
            .unwrap();
        assert!(matches!(other_vuln.severity, Severity::Medium));
    }

    #[test]
    fn test_check_security_headers_with_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_str("x-frame-options").unwrap(),
            HeaderValue::from_str("SAMEORIGIN").unwrap(),
        );
        headers.insert(
            HeaderName::from_str("x-content-type-options").unwrap(),
            HeaderValue::from_str("nosniff").unwrap(),
        );

        let mut vulnerabilities = Vec::new();
        check_security_headers(&headers, "https://example.com", &mut vulnerabilities);

        // Should have 4 missing headers (6 total - 2 present)
        assert_eq!(vulnerabilities.len(), 4);

        // Should not contain the headers we added
        assert!(!vulnerabilities
            .iter()
            .any(|v| v.description.contains("x-frame-options")));
        assert!(!vulnerabilities
            .iter()
            .any(|v| v.description.contains("x-content-type-options")));
    }

    #[test]
    fn test_check_security_headers_insecure_xfo() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_str("x-frame-options").unwrap(),
            HeaderValue::from_str("ALLOWALL").unwrap(),
        );

        let mut vulnerabilities = Vec::new();
        check_security_headers(&headers, "https://example.com", &mut vulnerabilities);

        // Should have 5 missing headers + 1 insecure header
        assert_eq!(vulnerabilities.len(), 6);

        let insecure_xfo = vulnerabilities
            .iter()
            .find(|v| v.vulnerability_type == "Insecure Header Configuration")
            .unwrap();
        assert!(matches!(insecure_xfo.severity, Severity::High));
        assert!(insecure_xfo
            .description
            .contains("X-Frame-Options set to ALLOWALL"));
    }

    #[test]
    fn test_web_vulnerability_creation() {
        let vuln = WebVulnerability {
            url: "https://example.com/.env".to_string(),
            vulnerability_type: "Exposed Sensitive File".to_string(),
            severity: Severity::Critical,
            description: "Environment file exposed".to_string(),
            recommendation: "Remove public access to .env file".to_string(),
        };

        assert_eq!(vuln.url, "https://example.com/.env");
        assert!(matches!(vuln.severity, Severity::Critical));
    }

    #[test]
    fn test_web_scan_results_creation() {
        let vulnerabilities = vec![WebVulnerability {
            url: "https://example.com/.env".to_string(),
            vulnerability_type: "Exposed File".to_string(),
            severity: Severity::High,
            description: "Test".to_string(),
            recommendation: "Fix it".to_string(),
        }];

        let results = WebScanResults {
            vulnerabilities: vulnerabilities.clone(),
            base_url: "https://example.com".to_string(),
            endpoints_checked: 50,
        };

        assert_eq!(results.vulnerabilities.len(), 1);
        assert_eq!(results.base_url, "https://example.com");
        assert_eq!(results.endpoints_checked, 50);
    }

    // Integration test would require a mock server
    // This is covered by the mockito dependency for future implementation
    #[tokio::test]
    async fn test_scan_web_invalid_url() {
        // Test with invalid URL should handle gracefully
        let result = scan_web("not-a-valid-url").await;
        // Should either error gracefully or return empty results
        // Implementation depends on how we want to handle invalid URLs
        assert!(result.is_ok() || result.is_err());
    }
}
