//! small tests for the edge cases of [`sort_clients`]
use crate::handle::sort::sort_clients;
use crate::handle::sort::tests::client_vec;
use crate::{ClientAddress, ClientData};

/// every combination of `(ignore_workspaces, ignore_monitors)` that is implemented
const MODES: [(bool, bool); 3] = [(false, false), (true, false), (false, true)];

fn addresses(clients: &[(ClientAddress, ClientData)]) -> Vec<String> {
	clients.iter().map(|(a, _)| a.to_string()).collect()
}

#[test]
fn no_clients_gives_no_clients() {
	for (ignore_workspaces, ignore_monitors) in MODES {
		assert!(sort_clients(vec![], ignore_workspaces, ignore_monitors).is_empty());
	}
}

#[test]
fn a_single_client_is_returned_unchanged() {
	for (ignore_workspaces, ignore_monitors) in MODES {
		let clients = client_vec![(5, 5, 10, 10, 0, 0)];
		let sorted = sort_clients(clients, ignore_workspaces, ignore_monitors);
		assert_eq!(addresses(&sorted), ["0x1"]);
	}
}

#[test]
fn every_client_is_kept_exactly_once() {
	// 4 clients on workspace 0 and 2 on workspace 1 (both on monitor 0)
	let clients = client_vec![
		(1, 1, 1, 3, 0, 0),
		(3, 1, 1, 3, 0, 0),
		(1, 5, 1, 2, 0, 0),
		(3, 5, 1, 2, 0, 0),
		(1, 1, 1, 3, 1, 0),
		(3, 1, 1, 3, 1, 0),
	];
	let mut expected = addresses(&clients);
	expected.sort();

	for (ignore_workspaces, ignore_monitors) in MODES {
		let sorted = sort_clients(clients.clone(), ignore_workspaces, ignore_monitors);
		let mut got = addresses(&sorted);
		got.sort();
		assert_eq!(got, expected, "{ignore_workspaces}/{ignore_monitors}");
	}
}

#[test]
fn clients_in_the_same_column_are_ordered_top_to_bottom() {
	// the lower client is given first
	let clients = client_vec![(1, 5, 1, 2, 0, 0), (1, 1, 1, 3, 0, 0)];

	let sorted = sort_clients(clients, false, false);

	assert_eq!(addresses(&sorted), ["0x2", "0x1"]);
}

#[test]
fn clients_in_the_same_row_are_ordered_left_to_right() {
	// the right client is given first
	let clients = client_vec![(5, 1, 2, 3, 0, 0), (1, 1, 2, 3, 0, 0)];

	let sorted = sort_clients(clients, false, false);

	assert_eq!(addresses(&sorted), ["0x2", "0x1"]);
}

#[test]
fn monitors_are_ordered_by_monitor_id_not_by_position() {
	// the client on monitor 1 is further left than the client on monitor 0
	let clients = client_vec![(0, 0, 1, 1, 0, 1), (10, 0, 1, 1, 0, 0)];

	let sorted = sort_clients(clients, false, false);

	assert_eq!(addresses(&sorted), ["0x2", "0x1"]);
}

#[test]
fn workspaces_are_kept_apart_unless_they_are_ignored() {
	// 0x1 is on workspace 1 at x = 1, 0x2 is on workspace 0 at x = 5
	let clients = client_vec![(1, 1, 1, 1, 1, 0), (5, 1, 1, 1, 0, 0)];

	// workspace 0 comes first, even though its client is further right
	let sorted = sort_clients(clients.clone(), false, false);
	assert_eq!(addresses(&sorted), ["0x2", "0x1"]);

	// treated like one workspace they are ordered by position
	let sorted = sort_clients(clients, true, false);
	assert_eq!(addresses(&sorted), ["0x1", "0x2"]);
}
