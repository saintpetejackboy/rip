use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Repository path to scan
    pub repository_path: PathBuf,

    /// Environment file name to parse
    pub env_filename: String,

    /// List of environment keys to scan for
    pub env_keys: Vec<String>,

    /// File extensions to include in scan
    pub file_extensions: Vec<String>,

    /// Directories to ignore during scan
    pub ignore_patterns: Vec<String>,

    /// Whether to include web scanning
    pub enable_web_scan: bool,

    /// Public URL for web scanning
    pub web_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            repository_path: PathBuf::from("."),
            env_filename: ".env".to_string(),
            env_keys: Vec::new(),
            file_extensions: vec![
                "js".to_string(),
                "ts".to_string(),
                "jsx".to_string(),
                "tsx".to_string(),
                "py".to_string(),
                "rb".to_string(),
                "php".to_string(),
                "java".to_string(),
                "go".to_string(),
                "rs".to_string(),
                "cpp".to_string(),
                "c".to_string(),
                "cs".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "json".to_string(),
                "xml".to_string(),
                "md".to_string(),
                "txt".to_string(),
            ],
            ignore_patterns: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".next".to_string(),
                "coverage".to_string(),
                ".nyc_output".to_string(),
                "logs".to_string(),
                "*.log".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
            ],
            enable_web_scan: false,
            web_url: None,
        }
    }
}

impl Config {
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let contents = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, contents)?;
        Ok(())
    }

    pub fn default_config_path() -> PathBuf {
        PathBuf::from(".ripconfig.toml")
    }

    /// Parse .env file and extract non-trivial keys
    pub fn parse_env_file(&mut self) -> Result<Vec<String>> {
        let env_path = self.repository_path.join(&self.env_filename);

        if !env_path.exists() {
            return Ok(Vec::new());
        }

        let contents = fs::read_to_string(&env_path)?;
        let mut keys = Vec::new();
        let trivial_values: HashSet<&str> = [
            "",
            "0",
            "localhost",
            "127.0.0.1",
            "true",
            "false",
            "null",
            "undefined",
        ]
        .iter()
        .cloned()
        .collect();

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').trim_matches('\'');

                // Skip trivial values
                if !trivial_values.contains(value) && !value.is_empty() {
                    keys.push(key.to_string());
                }
            }
        }

        Ok(keys)
    }

    /// Get environment variable values for scanning
    pub fn get_env_values(&self) -> Result<Vec<String>> {
        let env_path = self.repository_path.join(&self.env_filename);

        if !env_path.exists() {
            return Ok(Vec::new());
        }

        let contents = fs::read_to_string(&env_path)?;
        let mut values = Vec::new();
        let trivial_values: HashSet<&str> = [
            "",
            "0",
            "localhost",
            "127.0.0.1",
            "true",
            "false",
            "null",
            "undefined",
        ]
        .iter()
        .cloned()
        .collect();

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').trim_matches('\'');

                // Only include values for selected keys
                if self.env_keys.contains(&key.to_string())
                    && !trivial_values.contains(value)
                    && !value.is_empty()
                {
                    values.push(value.to_string());
                }
            }
        }

        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.repository_path, PathBuf::from("."));
        assert_eq!(config.env_filename, ".env");
        assert!(config.env_keys.is_empty());
        assert!(!config.file_extensions.is_empty());
        assert!(!config.ignore_patterns.is_empty());
        assert!(!config.enable_web_scan);
        assert!(config.web_url.is_none());
    }

    #[test]
    fn test_save_and_load_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("test_config.toml");

        let mut config = Config::default();
        config.env_keys = vec!["API_KEY".to_string(), "SECRET".to_string()];
        config.enable_web_scan = true;
        config.web_url = Some("https://example.com".to_string());

        config.save_to_file(&config_path)?;
        let loaded_config = Config::load_from_file(&config_path)?;

        assert_eq!(config.env_keys, loaded_config.env_keys);
        assert_eq!(config.enable_web_scan, loaded_config.enable_web_scan);
        assert_eq!(config.web_url, loaded_config.web_url);

        Ok(())
    }

    #[test]
    fn test_parse_env_file_empty() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut config = Config::default();
        config.repository_path = temp_dir.path().to_path_buf();

        fs::write(temp_dir.path().join(".env"), "")?;
        let keys = config.parse_env_file()?;
        assert!(keys.is_empty());

        Ok(())
    }

    #[test]
    fn test_parse_env_file_with_comments() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut config = Config::default();
        config.repository_path = temp_dir.path().to_path_buf();

        let env_content = "# This is a comment\nAPI_KEY=secret123\n# Another comment\nDEBUG=true";
        fs::write(temp_dir.path().join(".env"), env_content)?;

        let keys = config.parse_env_file()?;
        assert_eq!(keys, vec!["API_KEY"]);

        Ok(())
    }

    #[test]
    fn test_parse_env_file_trivial_values() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut config = Config::default();
        config.repository_path = temp_dir.path().to_path_buf();

        let env_content = "EMPTY=\nZERO=0\nLOCALHOST=localhost\nTRUE=true\nFALSE=false\nNULL=null";
        fs::write(temp_dir.path().join(".env"), env_content)?;

        let keys = config.parse_env_file()?;
        assert!(keys.is_empty());

        Ok(())
    }

    #[test]
    fn test_parse_env_file_mixed_values() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut config = Config::default();
        config.repository_path = temp_dir.path().to_path_buf();

        let env_content = "API_KEY=sk_live_secret\nDEBUG=true\nSECRET_TOKEN=abc123\nHOST=localhost";
        fs::write(temp_dir.path().join(".env"), env_content)?;

        let keys = config.parse_env_file()?;
        assert_eq!(keys, vec!["API_KEY", "SECRET_TOKEN"]);

        Ok(())
    }

    #[test]
    fn test_parse_env_file_quoted_values() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut config = Config::default();
        config.repository_path = temp_dir.path().to_path_buf();

        let env_content = "API_KEY=\"sk_live_secret\"\nTOKEN='bearer_token'";
        fs::write(temp_dir.path().join(".env"), env_content)?;

        let keys = config.parse_env_file()?;
        assert_eq!(keys, vec!["API_KEY", "TOKEN"]);

        Ok(())
    }

    #[test]
    fn test_get_env_values() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut config = Config::default();
        config.repository_path = temp_dir.path().to_path_buf();
        config.env_keys = vec!["API_KEY".to_string(), "SECRET".to_string()];

        let env_content = "API_KEY=secret123\nSECRET=mysecret\nOTHER=value";
        fs::write(temp_dir.path().join(".env"), env_content)?;

        let values = config.get_env_values()?;
        assert_eq!(values, vec!["secret123", "mysecret"]);

        Ok(())
    }

    #[test]
    fn test_get_env_values_nonexistent_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut config = Config::default();
        config.repository_path = temp_dir.path().to_path_buf();
        config.env_keys = vec!["API_KEY".to_string()];

        let values = config.get_env_values()?;
        assert!(values.is_empty());

        Ok(())
    }
}
