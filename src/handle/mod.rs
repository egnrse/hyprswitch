//! event handling and switching logic
//!
//! handles most interactions with hyprland

use crate::handle::next::{find_next_client, find_next_monitor, find_next_workspace};
use crate::ClientAddress;
use crate::{Active, DispatchConfig, HyprlandData, SwitchType};
use anyhow::Context;
use hyprland::data::{Client, Monitor, Monitors};
use hyprland::prelude::{HyprData, HyprDataActiveOptional, HyprDataVec};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use tracing::info;

pub use data::collect_data;
pub use exec::switch_to_active;

mod data;
mod exec;
mod next;
mod sort;

pub fn find_next(
	switch_type: &SwitchType,
	dispatch_config: &DispatchConfig,
	clients_data: &HyprlandData,
	active: Option<&Active>,
) -> anyhow::Result<Active> {
	match switch_type {
		SwitchType::Client => {
			let (addr, _) = find_next_client(
				dispatch_config,
				&clients_data.clients,
				if let Some(Active::Client(addr)) = &active {
					Some(addr)
				} else {
					None
				},
			)
			.with_context(|| {
				format!("Failed to find next client with dispatch_config {dispatch_config:?}")
			})?;
			info!("Next client: {:?}", addr);
			Ok(Active::Client(addr.clone()))
		}
		SwitchType::Workspace => {
			let (workspace_id, _) = find_next_workspace(
				dispatch_config,
				&clients_data.workspaces,
				if let Some(Active::Workspace(ws)) = &active {
					Some(ws)
				} else {
					None
				},
			)
			.with_context(|| {
				format!("Failed to find next workspace with dispatch_config {dispatch_config:?}")
			})?;
			info!("Next workspace: {:?}", workspace_id);
			Ok(Active::Workspace(workspace_id.clone()))
		}
		SwitchType::Monitor => {
			let (monitor_id, _) = find_next_monitor(
				dispatch_config,
				&clients_data.monitors,
				if let Some(Active::Monitor(monitor)) = &active {
					Some(monitor)
				} else {
					None
				},
			)
			.with_context(|| {
				format!("Failed to find next monitor with dispatch_config {dispatch_config:?}")
			})?;
			info!("Next monitor: {:?}", monitor_id);
			Ok(Active::Monitor(*monitor_id))
		}
	}
}

fn get_recent_clients_map() -> &'static Mutex<HashMap<ClientAddress, i8>> {
	static MAP_LOCK: OnceLock<Mutex<HashMap<ClientAddress, i8>>> = OnceLock::new();
	MAP_LOCK.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn clear_recent_clients() {
	get_recent_clients_map()
		.lock()
		.expect("Failed to lock focus_map")
		.clear();
}

pub fn get_monitors() -> Vec<Monitor> {
	Monitors::get().map_or(vec![], |monitors| monitors.to_vec())
}

pub fn get_active_monitor() -> Option<String> {
	match Client::get_active().map(|c| {
		c.map(|c| {
			Monitors::get().map(|monitors| {
				monitors
					.iter()
					.find(|m| Some(m.id) == c.monitor)
					.map(|mm| mm.name.clone())
			})
		})
	}) {
		Ok(Some(Ok(Some(monitor)))) => Some(monitor),
		_ => None,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::handle::sort::tests::{client_vec, monitor_map, workspace_map};

	fn dispatch(reverse: bool, offset: u8) -> DispatchConfig {
		DispatchConfig { reverse, offset }
	}

	/// 3 clients, 3 workspaces and 2 monitors
	fn hypr_data() -> HyprlandData {
		HyprlandData {
			clients: client_vec![(0, 0, 1, 1, 0, 0), (1, 0, 1, 1, 0, 0), (2, 0, 1, 1, 0, 1)],
			workspaces: workspace_map![(0, 0, 0), (1, 0, 0), (2, 0, 1)],
			monitors: monitor_map![(0, 0, 1, 1), (1, 1, 1, 1)],
		}
	}

	#[test]
	fn client_switch_without_active_selects_the_first_client() {
		let data = hypr_data();
		let next = find_next(&SwitchType::Client, &dispatch(false, 1), &data, None).unwrap();
		assert!(matches!(next, Active::Client(addr) if addr == data.clients[0].0));
	}

	#[test]
	fn client_switch_continues_after_the_active_client() {
		let data = hypr_data();
		let active = Active::Client(data.clients[1].0.clone());
		let next = find_next(
			&SwitchType::Client,
			&dispatch(false, 1),
			&data,
			Some(&active),
		)
		.unwrap();
		assert!(matches!(next, Active::Client(addr) if addr == data.clients[2].0));
	}

	#[test]
	fn client_switch_reverse_goes_to_the_previous_client() {
		let data = hypr_data();
		let active = Active::Client(data.clients[1].0.clone());
		let next = find_next(
			&SwitchType::Client,
			&dispatch(true, 1),
			&data,
			Some(&active),
		)
		.unwrap();
		assert!(matches!(next, Active::Client(addr) if addr == data.clients[0].0));
	}

	#[test]
	fn workspace_switch_continues_after_the_active_workspace() {
		let data = hypr_data();
		let active = Active::Workspace(data.workspaces[0].0.clone());
		let next = find_next(
			&SwitchType::Workspace,
			&dispatch(false, 1),
			&data,
			Some(&active),
		)
		.unwrap();
		assert!(matches!(next, Active::Workspace(ws) if ws == data.workspaces[1].0));
	}

	#[test]
	fn workspace_switch_wraps_around() {
		let data = hypr_data();
		let active = Active::Workspace(data.workspaces[2].0.clone());
		let next = find_next(
			&SwitchType::Workspace,
			&dispatch(false, 1),
			&data,
			Some(&active),
		)
		.unwrap();
		assert!(matches!(next, Active::Workspace(ws) if ws == data.workspaces[0].0));
	}

	#[test]
	fn monitor_switch_continues_after_the_active_monitor() {
		let data = hypr_data();
		let active = Active::Monitor(0);
		let next = find_next(
			&SwitchType::Monitor,
			&dispatch(false, 1),
			&data,
			Some(&active),
		)
		.unwrap();
		assert!(matches!(next, Active::Monitor(1)));
	}

	#[test]
	fn monitor_switch_reverse_wraps_around() {
		let data = hypr_data();
		let active = Active::Monitor(0);
		let next = find_next(
			&SwitchType::Monitor,
			&dispatch(true, 1),
			&data,
			Some(&active),
		)
		.unwrap();
		assert!(matches!(next, Active::Monitor(1)));
	}

	#[test]
	fn active_of_a_different_kind_is_ignored() {
		let data = hypr_data();
		// the switch type is `Client`, so an active workspace must not influence the result
		let active = Active::Workspace(data.workspaces[1].0.clone());
		let next = find_next(
			&SwitchType::Client,
			&dispatch(false, 1),
			&data,
			Some(&active),
		)
		.unwrap();
		assert!(matches!(next, Active::Client(addr) if addr == data.clients[0].0));
	}

	#[test]
	fn result_kind_follows_the_switch_type() {
		let data = hypr_data();
		let cfg = dispatch(false, 1);
		assert!(matches!(
			find_next(&SwitchType::Client, &cfg, &data, None).unwrap(),
			Active::Client(_)
		));
		assert!(matches!(
			find_next(&SwitchType::Workspace, &cfg, &data, None).unwrap(),
			Active::Workspace(_)
		));
		assert!(matches!(
			find_next(&SwitchType::Monitor, &cfg, &data, None).unwrap(),
			Active::Monitor(_)
		));
	}

	#[test]
	fn disabled_clients_are_not_selected() {
		let mut data = hypr_data();
		data.clients[0].1.enabled = false;
		let next = find_next(&SwitchType::Client, &dispatch(false, 1), &data, None).unwrap();
		assert!(matches!(next, Active::Client(addr) if addr == data.clients[1].0));
	}

	#[test]
	fn empty_data_is_an_error_with_context() {
		let data = HyprlandData::default();
		let cfg = dispatch(false, 1);

		let err = find_next(&SwitchType::Client, &cfg, &data, None).unwrap_err();
		assert!(err.to_string().contains("next client"), "{err}");
		let err = find_next(&SwitchType::Workspace, &cfg, &data, None).unwrap_err();
		assert!(err.to_string().contains("next workspace"), "{err}");
		let err = find_next(&SwitchType::Monitor, &cfg, &data, None).unwrap_err();
		assert!(err.to_string().contains("next monitor"), "{err}");
	}

	#[test]
	fn clear_recent_clients_empties_the_shared_map() {
		let map = get_recent_clients_map();
		let addr = ClientAddress::from(hyprland::shared::Address::new(0x77aa_u32));
		map.lock().unwrap().insert(addr.clone(), 3);
		assert_eq!(map.lock().unwrap().get(&addr), Some(&3));

		clear_recent_clients();

		assert!(map.lock().unwrap().get(&addr).is_none());
	}
}
