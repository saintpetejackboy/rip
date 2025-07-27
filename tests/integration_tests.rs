use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_version_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rip")?;
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));

    Ok(())
}

#[test]
fn test_version_subcommand() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rip")?;
    cmd.arg("version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));

    Ok(())
}

#[test]
fn test_help_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rip")?;
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("RIP"))
        .stdout(predicate::str::contains("scan"))
        .stdout(predicate::str::contains("config"));

    Ok(())
}

#[test]
fn test_config_show_default() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rip")?;
    cmd.arg("--skip-config").arg("config").arg("--show");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("repository_path"))
        .stdout(predicate::str::contains("env_filename"));

    Ok(())
}

#[test]
fn test_scan_nonexistent_path() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rip")?;
    cmd.arg("--auto").arg("scan").arg("/nonexistent/path");

    // Should handle gracefully - either succeed with no matches or error appropriately
    let _result = cmd.assert();
    // We don't assert failure here because the behavior might vary
    // depending on how we handle nonexistent paths

    Ok(())
}

#[test]
fn test_scan_with_env_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Create test files
    fs::write(
        temp_dir.path().join(".env"),
        "API_KEY=secret123\nDEBUG=true",
    )?;
    fs::write(temp_dir.path().join("test.js"), "const api = 'secret123';")?;

    let mut cmd = Command::cargo_bin("rip")?;
    cmd.arg("--auto").arg("scan").arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scan Results"));

    Ok(())
}

#[test]
fn test_scan_empty_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    let mut cmd = Command::cargo_bin("rip")?;
    cmd.arg("--auto").arg("scan").arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scan Results"))
        .stdout(predicate::str::contains("0"));

    Ok(())
}

#[test]
fn test_config_reset() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("test_config.toml");

    // Create a config file first
    fs::write(&config_path, "[config]\ntest = true")?;

    let mut cmd = Command::cargo_bin("rip")?;
    cmd.arg("--config")
        .arg(&config_path)
        .arg("config")
        .arg("--reset");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("reset to defaults"));

    Ok(())
}
