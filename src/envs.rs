//! environment settings

use lazy_static::lazy_static;
use std::env;
use tracing::debug;

lazy_static! {
	// not implemented in config generator
	pub static ref LOG_MODULE_PATH: bool = env::var("LOG_MODULE_PATH")
		.map(|s| s.parse().expect("Failed to parse LOG_MODULE_PATH"))
		.unwrap_or(false);
	// allows exiting on new monitors or new versions as it automatically restarts
	pub static ref SYSTEMD_SERVICE: bool = env::var("SYSTEMD_SERVICE")
		.map(|s| s.parse().expect("Failed to parse SYSTEMD_SERVICE"))
		.unwrap_or(false);

	pub static ref REMOVE_HTML_FROM_WORKSPACE_NAME: bool = env::var("REMOVE_HTML_FROM_WORKSPACE_NAME")
		.map(|s| s.parse().expect("Failed to parse REMOVE_HTML_FROM_WORKSPACE_NAME"))
		.unwrap_or(true);
	pub static ref DISABLE_TOASTS: bool = env::var("DISABLE_TOASTS")
		.map(|s| s.parse().expect("Failed to parse DISABLE_TOASTS"))
		.unwrap_or(false);
}

pub fn envvar_dump() {
	debug!(
		"ENV dump: \
    LOG_MODULE_PATH: {:?}, REMOVE_HTML_FROM_WORKSPACE_NAME: {:?}, DISABLE_TOASTS: {:?}, \
    SYSTEMD_SERVICE: {:?}",
		*LOG_MODULE_PATH, *REMOVE_HTML_FROM_WORKSPACE_NAME, *DISABLE_TOASTS, *SYSTEMD_SERVICE
	);
}
