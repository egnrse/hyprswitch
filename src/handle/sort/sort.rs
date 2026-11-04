use crate::ClientData;
use crate::{ClientAddress, WorkspaceAddress};
use hyprland::shared::MonitorId;
use std::collections::{BTreeMap, VecDeque};

/// Sorts clients with complex sorting
///
/// * 'clients' - Vector of clients to sort
/// * 'ignore_workspaces' - Don't split clients into workspaces (treat all clients on monitor as one workspace)
/// * 'ignore_monitors' - Don't split clients into monitors (treat all clients as one monitor)
pub fn sort_clients(
	clients: Vec<(ClientAddress, ClientData)>,
	ignore_workspaces: bool,
	ignore_monitors: bool,
) -> Vec<(ClientAddress, ClientData)> {
	// monitor -> workspace -> clients
	let monitors: Vec<Vec<Vec<(ClientAddress, ClientData)>>> =
		match (ignore_workspaces, ignore_monitors) {
			(true, true) => {
				panic!(
				"Can't ignore workspaces and monitors at the same time (currently not implemented)"
			);
				// one monitor with one workspace with every client
				// vec![vec![clients]]
			}
			(true, false) => {
				// workspace -> clients
				let mut monitors: BTreeMap<MonitorId, Vec<(ClientAddress, ClientData)>> =
					BTreeMap::new();
				for (addr, client) in clients {
					monitors
						.entry(client.monitor)
						.or_default()
						.push((addr, client));
				}
				monitors.into_values().map(|m| vec![m]).collect()
			}
			(false, true) => {
				// monitor -> workspaces
				let mut workspaces: BTreeMap<MonitorId, Vec<WorkspaceAddress>> = BTreeMap::new();
				for (_, client) in clients.iter() {
					workspaces
						.entry(client.monitor)
						.or_default()
						.push(client.workspace.clone());
				}
				// sort workspaces on monitor (and remove duplicates)
				for ws in workspaces.values_mut() {
					ws.sort();
					ws.dedup();
					ws.reverse();
				}

				// old (real) workspaceId -> new workspaceId
				let mut workspaces_map: BTreeMap<WorkspaceAddress, WorkspaceAddress> =
					BTreeMap::new();
				loop {
					let mut current_workspaces = vec![];
					// get one workspace from each monitor
					for ws in workspaces.values_mut() {
						if let Some(workspace) = ws.pop() {
							current_workspaces.push(workspace);
						}
					}
					if current_workspaces.is_empty() {
						break;
					}
					let new_workspace_id = current_workspaces[0].clone();
					// debug!("current_workspaces: {:?}, new_workspace_id: {}", current_workspaces, new_workspace_id);
					for wss in current_workspaces {
						workspaces_map.insert(wss, new_workspace_id.clone());
					}
				}

				let mut new_workspaces: BTreeMap<
					WorkspaceAddress,
					Vec<(ClientAddress, ClientData)>,
				> = BTreeMap::new();
				for (addr, client) in clients {
					new_workspaces
						.entry(
							workspaces_map
								.get(&client.workspace)
								.expect("Workspace for client not found")
								.clone(),
						)
						.or_default()
						.push((addr, client));
				}

				new_workspaces.into_values().map(|m| vec![m]).collect()
			}
			(false, false) => {
				// monitor -> workspace -> clients
				let mut monitors: BTreeMap<
					MonitorId,
					BTreeMap<WorkspaceAddress, Vec<(ClientAddress, ClientData)>>,
				> = BTreeMap::new();
				for (addr, client) in clients {
					monitors
						.entry(client.monitor)
						.or_default()
						.entry(client.workspace.clone())
						.or_default()
						.push((addr, client));
				}
				monitors
					.into_values()
					.map(|m| m.into_values().collect())
					.collect()
			}
		};

	let mut sorted_clients = Vec::new();

	for workspaces in monitors {
		for mut clients in workspaces {
			clients.sort_by(|(_, a), (_, b)| {
				if a.x == b.x {
					a.y.cmp(&b.y)
				} else {
					a.x.cmp(&b.x)
				}
			});
			let mut queue: VecDeque<(ClientAddress, ClientData)> = VecDeque::from(clients);

			let mut line_start = queue.pop_front();
			while let Some((current_addr, current)) = line_start {
				let mut current_bottom = current.y + current.height;
				sorted_clients.push((current_addr, current));

				loop {
					let mut next_index = None;

					/*
					1. Check If Top left of window is higher or lower than bottom left of current
					2. Check if any window(not taken) on left top is higher or lower than current Lower (if true take this)
					3. Check if any window(not taken) on left bottom is higher than current bottom (if true take this)
					=> Take if Top higher than current Bottom and no window on left has higher Top than window Bottom
					 */
					for (i, (_, client)) in queue.iter().enumerate() {
						let client_top = client.y;
						let client_bottom = client.y + client.height;
						let client_left = client.x;

						if client_top < current_bottom {
							// 1.
							// client top is inside current row

							// 2.
							let on_left = queue
								.iter()
								.enumerate()
								.find(|(_, (_, c))| c.x < client_left && c.y < client_bottom);

							// 3.
							let on_left_2 = queue.iter().enumerate().find(|(_, (_, c))| {
								c.x < client_left && c.y + c.height < client_bottom
							});

							match (on_left, on_left_2) {
								(Some((idx, (_, c))), _) => {
									current_bottom = c.y + c.height;
									next_index = Some(idx);
								}
								(_, Some((idx, (_, c)))) => {
									current_bottom = c.y + c.height;
									next_index = Some(idx);
								}
								(None, None) => {
									next_index = Some(i);
								}
							}
							break;
						}
					}
					match next_index.and_then(|i| queue.remove(i)) {
						Some(next) => {
							sorted_clients.push(next);
						}
						None => {
							break;
						}
					}
				}
				line_start = queue.pop_front();
			}
		}
	}

	sorted_clients
}
