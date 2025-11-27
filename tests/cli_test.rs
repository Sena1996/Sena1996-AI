use std::process::Command;

fn sena_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sena"))
}

#[test]
fn test_version_flag() {
    let output = sena_cmd()
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sena") || stdout.contains("SENA"));
}

#[test]
fn test_help_flag() {
    let output = sena_cmd()
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage") || stdout.contains("USAGE"));
}

#[test]
fn test_health_command() {
    let output = sena_cmd()
        .arg("health")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("HEALTH") || stdout.contains("healthy"));
}

#[test]
fn test_who_command() {
    let output = sena_cmd()
        .arg("who")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_metrics_command() {
    let output = sena_cmd()
        .arg("metrics")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_invalid_command() {
    let output = sena_cmd()
        .arg("invalid_command_xyz")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
}
