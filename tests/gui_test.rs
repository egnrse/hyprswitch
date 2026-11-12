//! test subcommand `gui`
//! options that need a running hyprland compositor have the flag 'ignore'

mod shared_test;
use shared_test::{assert_failure, assert_success, run_cli};
//use shared_test::{daemon_init, daemon_kill, DaemonGuard};

#[test]
fn test_cli_gui_invalid_modifier() {
	assert_failure(&["gui", "--mod-key", "invalid", "--key", "tab"]);
}
#[test]
fn test_cli_gui_missing_required() {
	assert_failure(&["gui", "--mod-key", "SUPER"]);
	assert_failure(&["gui", "--key", "tab"]);
}

#[test]
fn test_cli_gui_without_daemon() {
	let args = ["gui", "--mod-key", "SUPER", "--key", "tab"];
	let output = run_cli(&args);
	let stderr = String::from_utf8_lossy(&output.stderr);
	assert!(
		!output.status.success(),
		"`hyprswitch {}` did not fail without the daemon running\n{}",
		args.join(" "),
		stderr
	);
	// maybe change it to stdout at some point
	assert!(stderr.contains("Daemon not running"));
}
/*
#[test]
fn test_cli_gui_valid_modifiers() {
	let d = DaemonGuard::new(&[]);
	for modifier in ["ALT"] {
		assert_success(&["gui", "--mod-key", modifier, "--key", "tab"]);
	}
}
*/
