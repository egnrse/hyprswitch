//! shared procedures for tests

use std::process::Command;
//use std::process::{Child, Stdio};
//use std::thread;
//use std::time::Duration;

/// get the location of the compiled binary
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

/// run the cli, return the process output
pub fn run_cli(args: &[&str]) -> std::process::Output {
	let default_options = ["-vv"];
	Command::new(get_bin_path())
		.args(default_options)
		.args(args)
		.output()
		.expect("failed to execute hyprswitch process")
}
/// test if the command exited with success
/// (appends `--dry-run` to args)
pub fn assert_success(args: &[&str]) {
	let mut args = args.to_vec();
	args.push("--dry-run");
	let output = run_cli(&args);

	assert!(
		output.status.success(),
		"`hyprswitch {}` failed:\n{}",
		args.join(" "),
		String::from_utf8_lossy(&output.stderr)
	);
}
/// test if the command exited with a failure
pub fn assert_failure(args: &[&str]) {
	let output = run_cli(args);

	assert!(
		!output.status.success(),
		"`hyprswitch {}` unexpectedly succeeded:\n{}",
		args.join(" "),
		String::from_utf8_lossy(&output.stderr)
	);
}

/*
/// init a daemon
pub fn daemon_init(args: &[&str]) -> Child {
	let default_options = ["init", "-vv"];
	let child = Command::new(get_bin_path())
		.args(default_options)
		.args(args)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.expect("failed to run hyprswitch init");

	thread::sleep(Duration::from_millis(500));
	child

}
/// kill the daemon, return the daemon's output
pub fn daemon_kill(mut instance: Child) -> std::process::Output {
	instance
		.kill()
		.expect("failed to kill hyprswitch init");

	instance
		.wait_with_output()
		.expect("failed to wait for hyprswitch init")
}

pub struct DaemonGuard {
	child: Option<Child>,
}

impl DaemonGuard {
	pub fn new(args: &[&str]) -> Self {
		Self {
			child: Some(daemon_init(args)),
		}
	}
}

impl Drop for DaemonGuard {
	fn drop(&mut self) {
		if let Some(mut child) = self.child.take() {
			let _ = child.kill();
			let _ = child.wait();
		}
	}
}
*/
