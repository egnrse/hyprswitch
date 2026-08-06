use std::process::Command;

fn get_bin_path() -> std::path::PathBuf {
    // Cargo sets CARGO_BIN_EXE_<name> during integration testing
    if let Ok(bin) = std::env::var("CARGO_BIN_EXE_hyprswitch") {
        std::path::PathBuf::from(bin)
    } else {
        // Fallback to cargo target directory
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        if path.ends_with("deps") {
            path.pop();
        }
        path.push("hyprswitch");
        path
    }
}

#[test]
fn test_cli_help_flag() {
    let bin = get_bin_path();
    let output = Command::new(&bin)
        .arg("--help")
        .output()
        .expect("Failed to execute hyprswitch process");

    assert!(output.status.success());
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
	// maybe change it to stdout at some point
    assert!(stderr.contains("hyprswitch"));
    assert!(stderr.contains("Usage:"));
}

#[test]
#[ignore = "Requires a running Hyprland instance"]
fn test_cli_subcommand_dry_run() {
    let bin = get_bin_path();
    
    // Testing simple mode with dry-run
    let output = Command::new(&bin)
        .args(["simple", "--dry-run", "--offset", "1"])
        .output()
        .expect("Failed to execute hyprswitch process");

    assert!(output.status.success());
}

#[test]
fn test_cli_missing_required_args() {
    let bin = get_bin_path();
    
    // Calling `gui` without required `--mod-key` and `--key` should fail gracefully
    let output = Command::new(&bin)
        .arg("gui")
        .output()
        .expect("Failed to execute hyprswitch process");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("required"));
}
