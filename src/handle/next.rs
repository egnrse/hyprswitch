//! calculate the next monitor/workspace/client to switch to
//!
//! using [DispatchConfig] and a list of items

use crate::{ClientAddress, WorkspaceAddress};
use crate::{ClientData, DispatchConfig, MonitorData, WorkspaceData};
use hyprland::shared::MonitorId;
use tracing::{trace, warn};

/* private: not visible outside this module */

/// helper for the generic function [find_next]
trait Enabled {
	fn is_enabled(&self) -> bool;
}

#[rustfmt::skip] impl Enabled for MonitorData   { fn is_enabled(&self) -> bool { self.enabled } }
#[rustfmt::skip] impl Enabled for WorkspaceData { fn is_enabled(&self) -> bool { self.enabled } }
#[rustfmt::skip] impl Enabled for ClientData    { fn is_enabled(&self) -> bool { self.enabled } }

/// private generic implementation
fn find_next<'a, K, D>(
	dispatch_config: &DispatchConfig,
	items: &'a [(K, D)],
	selected: Option<&K>,
	kind: &str,
) -> anyhow::Result<&'a (K, D)>
where
	K: PartialEq,
	D: Enabled,
{
	let filtered: Vec<_> = items.iter().filter(|(_, d)| d.is_enabled()).collect();
	let len = filtered.len();
	if len == 0 {
		anyhow::bail!("No next {kind} found");
	}

	let step = dispatch_config.offset as usize % len;
	let current = selected.and_then(|s| filtered.iter().position(|(k, _)| k == s));
	if selected.is_some() && current.is_none() {
		warn!("selected {kind} not found");
	}

	let index = match (current, dispatch_config.reverse) {
		(Some(i), false) => (i + step) % len,
		(Some(i), true) => (i + len - step) % len,
		(None, false) => (step + len - 1) % len,
		(None, true) => (len - step) % len,
	};
	trace!("index: {}", index);

	Ok(filtered[index])
}

/* public entry points */

pub(crate) fn find_next_monitor<'a>(
	cfg: &DispatchConfig,
	data: &'a [(MonitorId, MonitorData)],
	selected: Option<&MonitorId>,
) -> anyhow::Result<&'a (MonitorId, MonitorData)> {
	find_next(cfg, data, selected, "monitor")
}

pub(crate) fn find_next_workspace<'a>(
	cfg: &DispatchConfig,
	data: &'a [(WorkspaceAddress, WorkspaceData)],
	selected: Option<&WorkspaceAddress>,
) -> anyhow::Result<&'a (WorkspaceAddress, WorkspaceData)> {
	find_next(cfg, data, selected, "workspace")
}

pub(crate) fn find_next_client<'a>(
	cfg: &DispatchConfig,
	data: &'a [(ClientAddress, ClientData)],
	selected: Option<&ClientAddress>,
) -> anyhow::Result<&'a (ClientAddress, ClientData)> {
	find_next(cfg, data, selected, "client")
}

#[cfg(test)]
mod tests {
	use hyprland::shared::Address;

	use super::*;
	use crate::handle::sort::tests::{client_vec, monitor_map, workspace_map};

	fn cfg(reverse: bool, offset: u8) -> DispatchConfig {
		DispatchConfig { reverse, offset }
	}

	/// `n` (max 5) enabled clients with the addresses `0x1..=0xn`
	fn make_clients(n: usize) -> Vec<(ClientAddress, ClientData)> {
		let all = client_vec![
			(0, 0, 1, 1, 0, 0),
			(1, 0, 1, 1, 0, 0),
			(2, 0, 1, 1, 0, 0),
			(3, 0, 1, 1, 0, 0),
			(4, 0, 1, 1, 0, 0),
		];
		all.into_iter().take(n).collect()
	}

	/// `n` (max 5) enabled workspaces with the addresses `0x0..=0x(n-1)`
	fn make_workspaces(n: usize) -> Vec<(WorkspaceAddress, WorkspaceData)> {
		let all = workspace_map![(0, 0, 0), (1, 0, 0), (2, 0, 0), (3, 0, 0), (4, 0, 0)];
		all.into_iter().take(n).collect()
	}

	/// `n` (max 5) enabled monitors with the ids `0..n`
	fn make_monitors(n: usize) -> Vec<(MonitorId, MonitorData)> {
		let all = monitor_map![
			(0, 0, 1, 1),
			(1, 0, 1, 1),
			(2, 0, 1, 1),
			(3, 0, 1, 1),
			(4, 0, 1, 1)
		];
		all.into_iter().take(n).collect()
	}

	/// The three `find_next_*` functions are copies of each other, so they share one test suite.
	///
	/// * `$make`: `fn(usize) -> Vec<(Id, Data)>` creating `n` enabled items
	/// * `$find`: the function under test
	/// * `$unknown`: an id that is never part of the items made by `$make`
	macro_rules! next_tests {
		($kind:ident, $make:ident, $find:ident, $unknown:expr) => {
			mod $kind {
				use super::*;

				#[test]
				fn forward_selects_the_following_item() {
					let items = $make(4);
					let next = $find(&cfg(false, 1), &items, Some(&items[0].0)).unwrap();
					assert_eq!(next.0, items[1].0);
				}

				#[test]
				fn forward_wraps_from_last_to_first() {
					let items = $make(4);
					let next = $find(&cfg(false, 1), &items, Some(&items[3].0)).unwrap();
					assert_eq!(next.0, items[0].0);
				}

				#[test]
				fn forward_offset_skips_items() {
					let items = $make(5);
					let next = $find(&cfg(false, 3), &items, Some(&items[1].0)).unwrap();
					assert_eq!(next.0, items[4].0);
				}

				#[test]
				fn forward_offset_wraps_around() {
					let items = $make(4);
					// 3 -> 0 -> 1 -> 2
					let next = $find(&cfg(false, 3), &items, Some(&items[3].0)).unwrap();
					assert_eq!(next.0, items[2].0);
				}

				#[test]
				fn forward_offset_larger_than_the_list_wraps_around() {
					let items = $make(3);
					// 1 + 5 = 6 => 6 % 3 = 0
					let next = $find(&cfg(false, 5), &items, Some(&items[1].0)).unwrap();
					assert_eq!(next.0, items[0].0);
				}

				#[test]
				fn reverse_selects_the_previous_item() {
					let items = $make(4);
					let next = $find(&cfg(true, 1), &items, Some(&items[2].0)).unwrap();
					assert_eq!(next.0, items[1].0);
				}

				#[test]
				fn reverse_wraps_from_first_to_last() {
					let items = $make(4);
					let next = $find(&cfg(true, 1), &items, Some(&items[0].0)).unwrap();
					assert_eq!(next.0, items[3].0);
				}

				#[test]
				fn reverse_offset_skips_items() {
					let items = $make(5);
					let next = $find(&cfg(true, 2), &items, Some(&items[3].0)).unwrap();
					assert_eq!(next.0, items[1].0);
				}

				#[test]
				fn reverse_offset_from_first_wraps_around() {
					let items = $make(5);
					let next = $find(&cfg(true, 2), &items, Some(&items[0].0)).unwrap();
					assert_eq!(next.0, items[3].0);
				}

				#[test]
				fn without_selection_forward_starts_at_the_first_item() {
					let items = $make(4);
					let next = $find(&cfg(false, 1), &items, None).unwrap();
					assert_eq!(next.0, items[0].0);
				}

				#[test]
				fn without_selection_forward_offset_counts_from_the_first_item() {
					let items = $make(4);
					let next = $find(&cfg(false, 3), &items, None).unwrap();
					assert_eq!(next.0, items[2].0);
				}

				#[test]
				fn without_selection_reverse_starts_at_the_last_item() {
					let items = $make(4);
					let next = $find(&cfg(true, 1), &items, None).unwrap();
					assert_eq!(next.0, items[3].0);
				}

				#[test]
				fn without_selection_reverse_offset_counts_from_the_last_item() {
					let items = $make(4);
					let next = $find(&cfg(true, 3), &items, None).unwrap();
					assert_eq!(next.0, items[1].0);
				}

				#[test]
				fn single_item_selects_itself() {
					let items = $make(1);
					for reverse in [false, true] {
						let with_selection =
							$find(&cfg(reverse, 1), &items, Some(&items[0].0)).unwrap();
						assert_eq!(with_selection.0, items[0].0);
						let without_selection = $find(&cfg(reverse, 1), &items, None).unwrap();
						assert_eq!(without_selection.0, items[0].0);
					}
				}

				#[test]
				fn disabled_items_are_skipped_going_forward() {
					let mut items = $make(4);
					items[1].1.enabled = false;
					items[2].1.enabled = false;
					let next = $find(&cfg(false, 1), &items, Some(&items[0].0)).unwrap();
					assert_eq!(next.0, items[3].0);
				}

				#[test]
				fn disabled_items_are_skipped_going_in_reverse() {
					let mut items = $make(4);
					items[1].1.enabled = false;
					items[2].1.enabled = false;
					let next = $find(&cfg(true, 1), &items, Some(&items[3].0)).unwrap();
					assert_eq!(next.0, items[0].0);
				}

				#[test]
				fn disabled_items_do_not_count_towards_the_offset() {
					let mut items = $make(5);
					items[1].1.enabled = false;
					// enabled: 0, 2, 3, 4 => two steps from 0 is 3
					let next = $find(&cfg(false, 2), &items, Some(&items[0].0)).unwrap();
					assert_eq!(next.0, items[3].0);
				}

				#[test]
				fn disabled_items_are_never_returned_without_selection() {
					let mut items = $make(3);
					items[0].1.enabled = false;
					let forward = $find(&cfg(false, 1), &items, None).unwrap();
					assert_eq!(forward.0, items[1].0);
					items[2].1.enabled = false;
					let reverse = $find(&cfg(true, 1), &items, None).unwrap();
					assert_eq!(reverse.0, items[1].0);
				}

				#[test]
				fn unknown_selection_still_returns_an_enabled_item() {
					let mut items = $make(4);
					items[0].1.enabled = false;
					for reverse in [false, true] {
						let next = $find(&cfg(reverse, 1), &items, Some(&$unknown)).unwrap();
						assert!(next.1.enabled);
					}
				}

				#[test]
				fn disabled_selection_is_treated_like_an_unknown_one() {
					let mut items = $make(4);
					items[1].1.enabled = false;
					for reverse in [false, true] {
						let next = $find(&cfg(reverse, 1), &items, Some(&items[1].0)).unwrap();
						assert!(next.1.enabled);
					}
				}

				#[test]
				fn empty_list_forward_is_an_error() {
					let items = $make(0);
					assert!($find(&cfg(false, 1), &items, None).is_err());
					assert!($find(&cfg(false, 1), &items, Some(&$unknown)).is_err());
				}

				#[test]
				fn list_with_only_disabled_items_forward_is_an_error() {
					let mut items = $make(3);
					for item in items.iter_mut() {
						item.1.enabled = false;
					}
					assert!($find(&cfg(false, 1), &items, None).is_err());
					assert!($find(&cfg(false, 1), &items, Some(&items[0].0)).is_err());
				}

				// The next tests describe the *intended* behaviour, which is currently not met:
				// `len - offset` / `index - offset` are computed on `usize` and underflow, which
				// panics in debug builds (and wraps to a huge index in release builds).
				// Remove the `#[ignore]` once that is fixed.

				#[test]
				#[ignore = "known bug: `index - offset` underflows when reverse offset > index"]
				fn reverse_offset_larger_than_the_index_wraps_around() {
					let items = $make(4);
					// 1 -> 0 -> 3 -> 2
					let next = $find(&cfg(true, 3), &items, Some(&items[1].0)).unwrap();
					assert_eq!(next.0, items[2].0);
				}

				#[test]
				#[ignore = "known bug: `len - offset` underflows when reverse offset > len"]
				fn reverse_offset_larger_than_the_list_wraps_around() {
					let items = $make(4);
					// -5 % 4 => 3
					let next = $find(&cfg(true, 5), &items, Some(&items[0].0)).unwrap();
					assert_eq!(next.0, items[3].0);
				}

				#[test]
				#[ignore = "known bug: `len - offset` underflows on an empty list when reversing"]
				fn empty_list_reverse_is_an_error_instead_of_a_panic() {
					let items = $make(0);
					assert!($find(&cfg(true, 1), &items, None).is_err());
				}
			}
		};
	}

	next_tests!(
		client,
		make_clients,
		find_next_client,
		ClientAddress::from(Address::new(999))
	);
	next_tests!(
		workspace,
		make_workspaces,
		find_next_workspace,
		WorkspaceAddress::from(Address::new(999))
	);
	next_tests!(monitor, make_monitors, find_next_monitor, 999 as MonitorId);
}
