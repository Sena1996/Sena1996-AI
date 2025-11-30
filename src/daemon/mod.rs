//! Daemon Module
//!
//! Background daemon for services

pub mod background;

pub use background::{BackgroundAgentManager, BackgroundTask, TaskQueue, TaskStatus};

use crate::config::SenaConfig;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
#[cfg(unix)]
use std::process::{Command, Stdio};

/// PID file location
fn pid_file() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".claude").join("sena_daemon.pid")
}

/// Log file location
fn log_file() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".claude").join("sena_daemon.log")
}

/// Check if daemon is running
pub fn is_running() -> bool {
    if let Ok(pid_str) = fs::read_to_string(pid_file()) {
        #[cfg(unix)]
        {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                let result = Command::new("kill")
                    .args(["-0", &pid.to_string()])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();

                return result.map(|s| s.success()).unwrap_or(false);
            }
        }

        #[cfg(not(unix))]
        {
            let _ = pid_str;
            return true;
        }
    }
    false
}

pub async fn start_daemon() -> Result<String, String> {
    let brand = SenaConfig::brand();
    if is_running() {
        return Err(format!("{} daemon is already running", brand));
    }

    // Ensure directory exists
    if let Some(parent) = pid_file().parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Cannot create directory: {}", e))?;
    }

    // For actual daemon mode, we'd fork here
    // For simplicity, we just record the intent and run in foreground
    let pid = std::process::id();

    fs::write(pid_file(), pid.to_string()).map_err(|e| format!("Cannot write PID file: {}", e))?;

    // Log startup
    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file())
        .map_err(|e| format!("Cannot open log: {}", e))?;

    writeln!(
        log,
        "[{}] {} daemon started (PID: {})",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        brand,
        pid
    )
    .ok();

    Ok(format!("{} daemon started (PID: {})", brand, pid))
}

pub async fn stop_daemon() -> Result<String, String> {
    let brand = SenaConfig::brand();
    if !is_running() {
        let _ = fs::remove_file(pid_file());
        return Err(format!("{} daemon is not running", brand));
    }

    let pid_str =
        fs::read_to_string(pid_file()).map_err(|e| format!("Cannot read PID file: {}", e))?;

    let pid: i32 = pid_str
        .trim()
        .parse()
        .map_err(|e| format!("Invalid PID: {}", e))?;

    // Send SIGTERM
    #[cfg(unix)]
    {
        let result = Command::new("kill")
            .args(["-15", &pid.to_string()])
            .status();

        match result {
            Ok(status) if status.success() => {}
            Ok(_) => return Err("Failed to stop daemon".to_string()),
            Err(e) => return Err(format!("Cannot send signal: {}", e)),
        }
    }

    // Remove PID file
    fs::remove_file(pid_file()).map_err(|e| format!("Cannot remove PID file: {}", e))?;

    if let Ok(mut log) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file())
    {
        writeln!(
            log,
            "[{}] {} daemon stopped",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            brand
        )
        .ok();
    }

    Ok(format!("{} daemon stopped (was PID: {})", brand, pid))
}

pub async fn daemon_status() -> Result<String, String> {
    let brand = SenaConfig::brand();
    if is_running() {
        let pid = fs::read_to_string(pid_file()).unwrap_or_else(|_| "unknown".to_string());

        Ok(format!("{} daemon is running (PID: {})", brand, pid.trim()))
    } else {
        Ok(format!("{} daemon is not running", brand))
    }
}

pub async fn run_daemon_loop() -> Result<(), String> {
    let brand = SenaConfig::brand();
    eprintln!("{} daemon running...", brand);

    // Main daemon loop
    loop {
        // Check for shutdown signal
        if !pid_file().exists() {
            eprintln!("PID file removed, shutting down...");
            break;
        }

        // Perform periodic tasks
        perform_periodic_tasks().await;

        // Sleep for 5 seconds between iterations
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    Ok(())
}

/// Perform periodic daemon tasks
async fn perform_periodic_tasks() {
    // Health check
    let health = crate::metrics::SenaHealth::new();
    let report = health.get_health();

    // Log health status periodically
    if let Ok(mut log) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file())
    {
        writeln!(
            log,
            "[{}] Health check: {} ({}%)",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            report.overall_status,
            report.metrics.overall_health_percentage
        )
        .ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_file_path() {
        let path = pid_file();
        assert!(path.to_string_lossy().contains("sena_daemon.pid"));
    }

    #[test]
    fn test_log_file_path() {
        let path = log_file();
        assert!(path.to_string_lossy().contains("sena_daemon.log"));
    }
}
