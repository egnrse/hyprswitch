use crate::{ClientAddress, WorkspaceAddress};
use crate::{ClientData, FindByFirst, MonitorData, WorkspaceData};
use hyprland::shared::MonitorId;
use tracing::error;

/// updates clients with workspace and monitor data
/// * 'clients' - Vector of clients to update
/// * 'workspace_data' - HashMap of workspace data
/// * 'monitor_data' - HashMap of monitor data, None if ignore_monitors
///
/// removes offset by monitor, adds offset by workspace (client on monitor 1 and workspace 2 will be moved left by monitor 1 offset and right by workspace 2 offset (workspace width * 2))
pub fn update_clients(
	clients: Vec<(ClientAddress, ClientData)>,
	workspace_data: Option<&Vec<(WorkspaceAddress, WorkspaceData)>>,
	monitor_data: Option<&Vec<(MonitorId, MonitorData)>>,
) -> Vec<(ClientAddress, ClientData)> {
	clients
		.into_iter()
		.filter_map(|(a, mut c)| {
			let ws = if let Some(wdt) = workspace_data {
				wdt.find_by_first(&c.workspace)
					.map(|ws| (ws.x, ws.y))
					.or_else(|| {
						error!("Workspace {:?} not found for client: {:?}", c.workspace, c);
						None
					})
			} else {
				Some((0, 0))
			};

			let md = if let Some(mdt) = monitor_data {
				mdt.find_by_first(&c.monitor)
					.map(|md| (md.x, md.y))
					.or_else(|| {
						error!("Monitor {:?} not found: {:?}", c.monitor, c);
						None
					})
			} else {
				Some((0, 0))
			};

			if let (Some((ws_x, ws_y)), Some((md_x, md_y))) = (ws, md) {
				c.x += ws_x - md_x; // move x cord by workspace offset
				c.y += ws_y - md_y; // move y cord by workspace offset
				Some((a, c))
			} else {
				None
			}
		})
		.collect()
}

#[cfg(test)]
mod tests {
	use super::update_clients;
	use crate::handle::sort::tests::{client_vec, monitor_map, workspace_map};

	#[test]
	fn without_workspace_and_monitor_data_nothing_moves() {
		let clients = client_vec![(10, 20, 30, 40, 0, 0), (50, 60, 70, 80, 0, 0)];
		let updated = update_clients(clients.clone(), None, None);

		assert_eq!(updated.len(), clients.len());
		for ((addr, old), (new_addr, new)) in clients.iter().zip(updated.iter()) {
			assert_eq!(addr, new_addr);
			assert_eq!((old.x, old.y), (new.x, new.y));
		}
	}

	#[test]
	fn workspace_offset_is_added_and_monitor_offset_is_subtracted() {
		let monitors = monitor_map![(30, 40, 1920, 1080)];
		let workspaces = workspace_map![(100, 200, 0)];
		let clients = client_vec![(10, 20, 300, 400, 0, 0)];

		let updated = update_clients(clients, Some(&workspaces), Some(&monitors));

		assert_eq!(updated.len(), 1);
		let (_, client) = &updated[0];
		// 10 + 100 - 30, 20 + 200 - 40
		assert_eq!((client.x, client.y), (80, 180));
	}

	#[test]
	fn size_is_not_changed() {
		let monitors = monitor_map![(30, 40, 1920, 1080)];
		let workspaces = workspace_map![(100, 200, 0)];
		let clients = client_vec![(10, 20, 300, 400, 0, 0)];

		let updated = update_clients(clients, Some(&workspaces), Some(&monitors));

		assert_eq!((updated[0].1.width, updated[0].1.height), (300, 400));
	}

	#[test]
	fn only_the_monitor_offset_is_used_without_workspace_data() {
		let monitors = monitor_map![(30, 40, 1920, 1080)];
		let clients = client_vec![(10, 20, 1, 1, 0, 0)];

		let updated = update_clients(clients, None, Some(&monitors));

		assert_eq!((updated[0].1.x, updated[0].1.y), (-20, -20));
	}

	#[test]
	fn only_the_workspace_offset_is_used_without_monitor_data() {
		let workspaces = workspace_map![(100, 200, 0)];
		let clients = client_vec![(10, 20, 1, 1, 0, 0)];

		let updated = update_clients(clients, Some(&workspaces), None);

		assert_eq!((updated[0].1.x, updated[0].1.y), (110, 220));
	}

	#[test]
	fn every_client_uses_the_offset_of_its_own_workspace_and_monitor() {
		// workspace 0 is on monitor 0, workspace 1 on monitor 1
		let monitors = monitor_map![(0, 0, 1920, 1080), (1920, 0, 1920, 1080)];
		let workspaces = workspace_map![(0, 0, 0), (5000, 0, 1)];
		let clients = client_vec![(10, 5, 1, 1, 0, 0), (1930, 5, 1, 1, 1, 1)];

		let updated = update_clients(clients, Some(&workspaces), Some(&monitors));

		assert_eq!(updated.len(), 2);
		assert_eq!((updated[0].1.x, updated[0].1.y), (10, 5));
		// 1930 + 5000 - 1920
		assert_eq!((updated[1].1.x, updated[1].1.y), (5010, 5));
	}

	#[test]
	fn clients_on_an_unknown_workspace_are_dropped() {
		let workspaces = workspace_map![(0, 0, 0)];
		let clients = client_vec![(1, 1, 1, 1, 0, 0), (2, 2, 1, 1, 5, 0), (3, 3, 1, 1, 0, 0)];
		let kept = [clients[0].0.clone(), clients[2].0.clone()];

		let updated = update_clients(clients, Some(&workspaces), None);

		let addresses: Vec<_> = updated.iter().map(|(a, _)| a.clone()).collect();
		assert_eq!(addresses, kept);
	}

	#[test]
	fn clients_on_an_unknown_monitor_are_dropped() {
		let monitors = monitor_map![(0, 0, 1920, 1080)];
		let clients = client_vec![(1, 1, 1, 1, 0, 0), (2, 2, 1, 1, 0, 7)];
		let kept = clients[0].0.clone();

		let updated = update_clients(clients, None, Some(&monitors));

		assert_eq!(updated.len(), 1);
		assert_eq!(updated[0].0, kept);
	}

	#[test]
	fn unknown_workspaces_are_ignored_when_no_workspace_data_is_given() {
		// `None` means "don't use workspace offsets", not "no workspace is known"
		let clients = client_vec![(1, 1, 1, 1, 42, 0)];

		let updated = update_clients(clients, None, None);

		assert_eq!(updated.len(), 1);
	}

	#[test]
	fn no_clients_gives_no_clients() {
		let monitors = monitor_map![(0, 0, 1920, 1080)];
		let workspaces = workspace_map![(0, 0, 0)];

		let updated = update_clients(vec![], Some(&workspaces), Some(&monitors));

		assert!(updated.is_empty());
	}
}
