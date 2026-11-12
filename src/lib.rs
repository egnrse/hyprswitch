//! Switch between windows/workspaces/monitors on [hyprland](https://hypr.land/).

#![deny(clippy::print_stdout)]

use anyhow::Context;
use async_channel::{Receiver, Sender};
use hyprland::data::Version as HyprlandVersion;
use hyprland::prelude::HyprData;
use hyprland::shared::MonitorId;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::env::var;
use std::fmt;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{info, trace, warn};

pub use configs::*;
pub use data::*;

pub mod client;
pub mod daemon;
pub mod envs;
pub mod handle;

mod configs;
mod data;

/// minimum hyprland version which is supported
///
/// reason: changed id field to address for eg. clients
const MIN_VERSION: Version = Version::new(0, 57, 0);

/// global variables
pub mod global {
	/// store if we are in dry mode
	pub static DRY: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

	/// store if the gui is open
	pub static OPEN: std::sync::OnceLock<std::sync::Mutex<bool>> = std::sync::OnceLock::new();
}

/// between what to switch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwitchType {
	Client,
	Workspace,
	Monitor,
}

/// when to close the gui
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloseType {
	Default,
	/// on release of the modifier key
	ModKeyRelease,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReverseKey {
	Mod(ModKey),
	Key(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferType {
	/// switch to next/prev workspace/monitor/client
	Dispatch(DispatchConfig),
	/// init with config, gui_config and submap
	Init(SimpleConfig, GuiConfig, SubmapConfig),
	/// close command with kill
	Close(bool),
	/// check if versions match (always succeeds)
	VersionCheck,
	/// check if the daemon is active (gui is open)
	Open,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
	pub transfer: TransferType,
	pub version: String,
}

#[derive(Debug)]
pub struct Exec {
	pub exec: Box<str>,
	pub path: Option<Box<str>>,
	pub terminal: bool,
}

#[derive(Debug)]
pub enum Active {
	Workspace(WorkspaceAddress),
	Monitor(MonitorId),
	Client(ClientAddress),
}

#[derive(Debug, Default)]
pub struct SharedData {
	pub simple_config: SimpleConfig,
	pub submap_config: SubmapConfig,
	pub gui_config: GuiConfig,
	pub active: Option<Active>,
	pub hypr_data: HyprlandData,
}

/// modifier keys
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ModKey {
	AltL,
	AltR,
	CtrlL,
	CtrlR,
	SuperL,
	SuperR,
	ShiftL,
	ShiftR,
}

impl Display for ModKey {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			// need SCREAMING_SNAKE_CASE
			ModKey::SuperL => write!(f, "SUPER_L"),
			ModKey::SuperR => write!(f, "SUPER_R"),
			ModKey::AltL => write!(f, "ALT_L"),
			ModKey::AltR => write!(f, "ALT_R"),
			ModKey::CtrlL => write!(f, "CTRL_L"),
			ModKey::CtrlR => write!(f, "CTRL_R"),
			ModKey::ShiftL => write!(f, "SHIFT_L"),
			ModKey::ShiftR => write!(f, "SHIFT_R"),
		}
	}
}

#[derive(Debug, Clone)]
pub enum GUISend {
	Refresh,
	New,
	Hide,
	Exit,
}

#[derive(Debug, Clone)]
pub enum UpdateCause {
	Client(u8),
	GuiClick,
	BackgroundThread(Option<u8>),
}

impl Display for UpdateCause {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			UpdateCause::Client(id) => write!(f, "id:{}", id),
			UpdateCause::GuiClick => write!(f, "gc"),
			UpdateCause::BackgroundThread(op) => match op {
				Some(id) => write!(f, "bt:{}", id),
				None => write!(f, "bt"),
			},
		}
	}
}
pub type Payload = (GUISend, UpdateCause);

// shared ARC with Mutex and Notify for new_gui and update_gui
pub type Share = Arc<(
	Mutex<SharedData>,
	Sender<Payload>,
	Receiver<Option<Payload>>,
)>;

/// get the path to the socket for communication between the daemon and a client
pub fn get_socket_path_buff() -> PathBuf {
	let mut buf = if let Ok(runtime_path) = var("XDG_RUNTIME_DIR") {
		PathBuf::from(runtime_path)
	} else if let Ok(uid) = var("UID") {
		PathBuf::from("/run/user/".to_owned() + &uid)
	} else {
		PathBuf::from("/tmp")
	};

	buf.push(envs::SOCKET_NAME.as_str());
	buf
}
pub trait Warn<A> {
	fn warn(self, msg: &str) -> Option<A>;
}

impl<A> Warn<A> for Option<A> {
	fn warn(self, msg: &str) -> Option<A> {
		match self {
			Some(o) => Some(o),
			None => {
				warn!("{}", msg);
				None
			}
		}
	}
}

impl<A, E: Display> Warn<A> for Result<A, E> {
	fn warn(self, msg: &str) -> Option<A> {
		match self {
			Ok(o) => Some(o),
			Err(e) => {
				warn!("{}: {}", msg, e);
				None
			}
		}
	}
}

/// send a notification
///
/// can be disabled with the env option `DISABLE_TOASTS`
pub fn toast(_body: &str) {
	if !*envs::DISABLE_TOASTS {
		#[cfg(not(debug_assertions))]
		let _ = notify_rust::Notification::new()
			.summary(&format!(
				"{} ({}) Error",
				env!("CARGO_PKG_NAME"),
				env!("CARGO_PKG_VERSION")
			))
			.body(_body)
			.timeout(10000)
			.hint(notify_rust::Hint::Urgency(notify_rust::Urgency::Critical))
			.show();
	}
}

/// check running hyprland version
pub fn check_version() -> anyhow::Result<()> {
	let version = HyprlandVersion::get()
		.context("Failed to get version! (Hyprland is probably outdated or too new??)")?;

	trace!("Hyprland {version:?}");
	info!(
		"Starting Hyprswitch ({}) on Hyprland {}",
		option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?"),
		version.version.clone().unwrap_or(version.tag.clone()),
	);

	let parsed_version = Version::parse(
		&version
			.version
			.unwrap_or(version.tag.trim_start_matches('v').to_string()),
	)
	.context("Unable to parse Hyprland Version")?;

	if parsed_version.lt(&MIN_VERSION) {
		toast(&format!(
			"Hyprland version too old or unknown: {parsed_version:?} < {MIN_VERSION:?}"
		));
		warn!("Hyprland version too old or unknown: {parsed_version:?} < {MIN_VERSION:?}");
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use std::ffi::OsString;

	use super::*;

	#[test]
	fn update_cause_display() {
		assert_eq!(UpdateCause::Client(3).to_string(), "id:3");
		assert_eq!(UpdateCause::GuiClick.to_string(), "gc");
		assert_eq!(UpdateCause::BackgroundThread(Some(7)).to_string(), "bt:7");
		assert_eq!(UpdateCause::BackgroundThread(None).to_string(), "bt");
	}

	#[test]
	fn warn_passes_values_through_and_turns_errors_into_none() {
		assert_eq!(Some(1).warn("unused"), Some(1));
		assert_eq!(None::<i32>.warn("something is missing"), None);
		assert_eq!(Ok::<_, String>(2).warn("unused"), Some(2));
		assert_eq!(Err::<i32, _>("boom").warn("something failed"), None);
	}

	/// sends `transfer` through the same (de)serialization the client and the daemon use
	fn roundtrip(transfer: TransferType) -> Transfer {
		let sent = Transfer {
			transfer,
			version: "6.0.0".to_string(),
		};
		let bytes = bincode::serialize(&sent).expect("transfer must be serializable");
		bincode::deserialize(&bytes).expect("transfer must be deserializable")
	}

	#[test]
	fn dispatch_transfer_survives_bincode() {
		let received = roundtrip(TransferType::Dispatch(DispatchConfig {
			reverse: true,
			offset: 3,
		}));

		assert_eq!(received.version, "6.0.0");
		assert!(matches!(
			received.transfer,
			TransferType::Dispatch(DispatchConfig {
				reverse: true,
				offset: 3
			})
		));
	}

	#[test]
	fn init_transfer_survives_bincode() {
		let simple = SimpleConfig {
			switch_type: SwitchType::Workspace,
			sort_recent: true,
			filter_same_class: true,
			..Default::default()
		};
		let gui = GuiConfig {
			max_switch_offset: 4,
			monitors: Some(vec!["DP-1".to_string(), "HDMI-A-1".to_string()]),
			..Default::default()
		};
		let submaps = [
			SubmapConfig::Config {
				mod_key: ModKey::AltL,
				key: "tab".to_string(),
				close: CloseType::ModKeyRelease,
				reverse_key: ReverseKey::Key("grave".to_string()),
			},
			SubmapConfig::Name {
				name: "hyprswitch".to_string(),
				reverse_key: ReverseKey::Mod(ModKey::ShiftR),
			},
		];

		for submap in submaps {
			let sent = TransferType::Init(simple.clone(), gui.clone(), submap);
			let expected = format!("{sent:?}");

			let received = roundtrip(sent);

			assert_eq!(format!("{:?}", received.transfer), expected);
		}
	}

	#[test]
	fn simple_transfers_survive_bincode() {
		assert!(matches!(
			roundtrip(TransferType::Close(true)).transfer,
			TransferType::Close(true)
		));
		assert!(matches!(
			roundtrip(TransferType::Close(false)).transfer,
			TransferType::Close(false)
		));
		assert!(matches!(
			roundtrip(TransferType::VersionCheck).transfer,
			TransferType::VersionCheck
		));
		assert!(matches!(
			roundtrip(TransferType::Open).transfer,
			TransferType::Open
		));
	}

	#[test]
	fn truncated_transfer_is_an_error_instead_of_a_panic() {
		let sent = Transfer {
			transfer: TransferType::Dispatch(DispatchConfig {
				reverse: false,
				offset: 1,
			}),
			version: "6.0.0".to_string(),
		};
		let bytes = bincode::serialize(&sent).unwrap();

		assert!(bincode::deserialize::<Transfer>(&[]).is_err());
		assert!(bincode::deserialize::<Transfer>(&bytes[..bytes.len() - 1]).is_err());
	}

	/// restores the given environment variables when dropped (even if the test fails)
	struct EnvGuard(Vec<(&'static str, Option<OsString>)>);

	impl EnvGuard {
		fn new(keys: &[&'static str]) -> Self {
			Self(
				keys.iter()
					.map(|key| (*key, std::env::var_os(key)))
					.collect(),
			)
		}
	}

	impl Drop for EnvGuard {
		fn drop(&mut self) {
			for (key, value) in self.0.drain(..) {
				match value {
					Some(value) => std::env::set_var(key, value),
					None => std::env::remove_var(key),
				}
			}
		}
	}

	// the only test that touches these variables, so it is safe next to the other (parallel) tests
	#[test]
	fn socket_path_is_taken_from_the_environment() {
		let _guard = EnvGuard::new(&["XDG_RUNTIME_DIR", "UID"]);
		let file = if cfg!(debug_assertions) {
			"hyprswitch.debug.sock"
		} else {
			"hyprswitch.sock"
		};

		std::env::set_var("XDG_RUNTIME_DIR", "/xdg/runtime");
		std::env::set_var("UID", "1234");
		assert_eq!(
			get_socket_path_buff(),
			PathBuf::from("/xdg/runtime").join(file)
		);

		std::env::remove_var("XDG_RUNTIME_DIR");
		assert_eq!(
			get_socket_path_buff(),
			PathBuf::from("/run/user/1234").join(file)
		);

		std::env::remove_var("UID");
		assert_eq!(get_socket_path_buff(), PathBuf::from("/tmp").join(file));
	}
}
