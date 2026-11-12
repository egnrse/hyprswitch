//! custom constants, data structures, traits and helper functions

use hyprland::data::{Workspace, WorkspaceBasic, Workspaces};
use hyprland::shared::{Address, HyprData, MonitorId};

/* CONST */

/// panic message for `.expect()` on the [`Address`] field of [`hyprland::data`] structures
pub const ADDR_EXPECT: &str = "address should be available on hyprland >=0.57";

/* STRUCTS */

/// wrapper for [`Address`] when used for Workspaces
/// > **Warning (unstable):** I am trying to upstream this datatype to [`hyprland`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkspaceAddress(pub Address);
/// wrapper for [`Address`] when used for Clients
/// > **Warning (unstable):** I am trying to upstream this datatype to [`hyprland`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClientAddress(pub Address);
impl From<Address> for WorkspaceAddress {
	fn from(a: Address) -> Self {
		WorkspaceAddress(a)
	}
}
impl From<Address> for ClientAddress {
	fn from(a: Address) -> Self {
		ClientAddress(a)
	}
}
impl std::fmt::Display for WorkspaceAddress {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}
impl std::fmt::Display for ClientAddress {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// internal represenation of a [hyprland::data::Monitor]
#[derive(Debug, Clone)]
pub struct MonitorData {
	pub x: i32,
	pub y: i32,
	pub width: u16,
	pub height: u16,
	pub connector: String,
	pub enabled: bool,
}

/// internal represenation of a [`Workspace`]
#[derive(Debug, Clone)]
pub struct WorkspaceData {
	pub name: String,
	pub x: i32,
	pub y: i32,
	pub width: u16,
	pub height: u16,
	pub monitor: MonitorId,
	pub enabled: bool,
	/// upstream workspace struct
	pub ws: Workspace,
}

/// internal represenation of a [hyprland::data::Client]
#[derive(Debug, Clone)]
pub struct ClientData {
	pub x: i32,
	pub y: i32,
	pub width: i32,
	pub height: i32,
	pub class: String,
	pub title: String,
	pub workspace: WorkspaceAddress,
	pub monitor: MonitorId,
	pub focus_history_id: i8,
	pub floating: bool,
	pub enabled: bool,
	pub pid: i32,
	/// upstream workspace struct
	pub ws: WorkspaceBasic,
}

/// a data struct that holds all data we need from hyprland
#[derive(Debug, Default)]
pub struct HyprlandData {
	pub clients: Vec<(ClientAddress, ClientData)>,
	pub workspaces: Vec<(WorkspaceAddress, WorkspaceData)>,
	pub monitors: Vec<(MonitorId, MonitorData)>,
}

/* TRAITS */

/// lookup of a value in a collection of `(ID, Data)` pairs by its key
pub trait FindByFirst<ID, Data> {
	fn find_by_first(&self, id: &ID) -> Option<&Data>;
}
impl FindByFirst<ClientAddress, ClientData> for Vec<(ClientAddress, ClientData)> {
	fn find_by_first(&self, id: &ClientAddress) -> Option<&ClientData> {
		self.iter().find(|(addr, _)| *addr == *id).map(|(_, cd)| cd)
	}
}
impl FindByFirst<WorkspaceAddress, WorkspaceData> for Vec<(WorkspaceAddress, WorkspaceData)> {
	fn find_by_first(&self, id: &WorkspaceAddress) -> Option<&WorkspaceData> {
		self.iter().find(|(wid, _)| *wid == *id).map(|(_, wd)| wd)
	}
}
impl FindByFirst<MonitorId, MonitorData> for Vec<(MonitorId, MonitorData)> {
	fn find_by_first(&self, id: &MonitorId) -> Option<&MonitorData> {
		self.iter().find(|(mid, _)| *mid == *id).map(|(_, md)| md)
	}
}

/* FUNCTIONS */

/// returns the equivalent [`Workspace`] to a [`WorkspaceBasic`] object
pub fn get_ws(w: &WorkspaceBasic) -> Option<Workspace> {
	let workspaces = Workspaces::get().ok()?;
	workspaces
		.iter()
		.find(|ws| ws.address == w.address)
		.cloned()
}

#[cfg(test)]
mod tests {
	use super::*;

	fn monitor(x: i32, connector: &str) -> MonitorData {
		MonitorData {
			x,
			y: 0,
			width: 1920,
			height: 1080,
			connector: connector.to_string(),
			enabled: true,
		}
	}

	#[test]
	fn addresses_display_like_the_wrapped_address() {
		let address = Address::new("0x1f");

		assert_eq!(WorkspaceAddress::from(address.clone()).to_string(), "0x1f");
		assert_eq!(ClientAddress::from(address).to_string(), "0x1f");
	}

	#[test]
	fn find_by_first_returns_the_data_of_the_matching_monitor() {
		let monitors = vec![(0, monitor(0, "DP-1")), (1, monitor(1920, "HDMI-A-1"))];

		let found = monitors.find_by_first(&1).expect("monitor 1 exists");

		assert_eq!(found.connector, "HDMI-A-1");
		assert_eq!(found.x, 1920);
	}

	#[test]
	fn find_by_first_returns_none_for_an_unknown_monitor() {
		let monitors = vec![(0, monitor(0, "DP-1"))];

		assert!(monitors.find_by_first(&9).is_none());
	}

	#[test]
	fn find_by_first_on_an_empty_list_returns_none() {
		let monitors: Vec<(MonitorId, MonitorData)> = vec![];

		assert!(monitors.find_by_first(&0).is_none());
	}

	#[test]
	fn find_by_first_returns_the_first_of_duplicate_ids() {
		let monitors = vec![(0, monitor(0, "first")), (0, monitor(1, "second"))];

		assert_eq!(monitors.find_by_first(&0).unwrap().connector, "first");
	}
}
