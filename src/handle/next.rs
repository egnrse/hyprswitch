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
