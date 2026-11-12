//! test subcommand `simple`
//! options that need a running hyprland compositor have the flag 'ignore'

mod shared_test;
use shared_test::{assert_failure, assert_success};

#[test]
fn test_cli_invalid_offset_zero() {
	assert_failure(&["simple", "--offset", "0"]);
}
#[test]
fn test_cli_invalid_offset_non_numeric() {
	assert_failure(&["simple", "--offset", "abc"]);
}
#[test]
#[ignore = "requires a running hyprland instance"]
fn test_cli_simple_reverse() {
	assert_success(&["simple", "--reverse"]);
	assert_success(&["simple", "-r"]);
}
#[test]
#[ignore = "requires a running hyprland instance"]
fn test_cli_simple_sort_recent() {
	assert_success(&["simple", "--sort-recent"]);
}
#[test]
#[ignore = "requires a running hyprland instance"]
fn test_cli_simple_boolean_options() {
	for option in [
		"--include-special-workspaces",
		"--ignore-workspaces",
		"--ignore-monitors",
		"--filter-same-class",
		"--filter-current-workspace",
		"--filter-current-monitor",
		"--sort-recent",
	] {
		assert_success(&["simple", option]);
	}
}
#[test]
#[ignore = "requires a running hyprland instance"]
fn test_cli_switch_types() {
	for switch_type in ["client", "workspace", "monitor"] {
		assert_success(&["simple", "--switch-type", switch_type]);
	}
}
#[test]
fn test_cli_invalid_switch_type() {
	assert_failure(&["simple", "--switch-type", "invalid"]);
}
#[test]
#[ignore = "requires a running hyprland instance"]
fn test_cli_simple_dry_run() {
	// dry run is already added by assert_success
	assert_success(&["simple", "--offset", "1"]);
}
