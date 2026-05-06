use anyhow::Context;
use hyprland::data::{Workspace, WorkspaceBasic};
use hyprland::prelude::HyprDataActive;
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use tracing::{debug, span, warn, Level};

use crate::{global, Active, FindByFirst, HyprlandData};

pub fn switch_to_active(
	active: Option<&Active>,
	clients_data: &HyprlandData,
) -> anyhow::Result<()> {
	let _span = span!(Level::TRACE, "exec", active = ?active).entered();
	match active {
		Some(Active::Client(addr)) => {
			let data = clients_data
				.clients
				.find_by_first(addr)
				.context("Client data not found")?;
			debug!("Switching to client {}", data.title);
			let workspace_data = clients_data
				.workspaces
				.find_by_first(&data.workspace)
				.context("Workspace data not found")?;
			switch_workspace(
				&WorkspaceBasic {
					id: data.workspace,
					name: workspace_data.name.clone(),
				},
				*global::DRY.get().expect("DRY not set"),
			)
			.with_context(|| {
				format!("Failed to execute switch workspace with workspace_data {workspace_data:?}")
			})?;
			switch_client(addr, *global::DRY.get().expect("DRY not set"))
				.with_context(|| format!("Failed to execute with addr {addr:?}"))?;
		}
		Some(Active::Workspace(wid)) => {
			let workspace_data = clients_data
				.workspaces
				.find_by_first(wid)
				.context("Workspace data not found")?;
			switch_workspace(
				&WorkspaceBasic {
					id: *wid,
					name: workspace_data.name.clone(),
				},
				*global::DRY.get().expect("DRY not set"),
			)
			.with_context(|| {
				format!("Failed to execute switch workspace with workspace_data {workspace_data:?}")
			})?;
		}
		Some(Active::Monitor(mid)) => {
			switch_monitor(mid, *global::DRY.get().expect("DRY not set")).with_context(|| {
				format!("Failed to execute switch monitor with monitor_id {mid:?}")
			})?;
		}
		None => {
			warn!("Not executing switch (active = Unknown)");
		}
	};
	Ok(())
}

fn switch_monitor(monitor_id: &MonitorId, dry_run: bool) -> anyhow::Result<()> {
	if dry_run {
		#[allow(clippy::print_stdout)]
		{
			println!("switch to monitor {monitor_id}");
		}
	} else {
		debug!("[EXEC] switch to monitor {monitor_id}");
		std::process::Command::new("hyprctl")
			.args([
				"eval",
				&format!("hl.dispatch(hl.dsp.focus({{ monitor = {} }}))", monitor_id),
			])
			.output()?;
	}
	Ok(())
}

fn switch_workspace(next_workspace: &WorkspaceBasic, dry_run: bool) -> anyhow::Result<()> {
	// check if already on workspace (if so, don't switch because it throws an error `Previous workspace doesn't exist`)
	let current_workspace = Workspace::get_active();
	if let Ok(workspace) = current_workspace {
		if next_workspace.id == workspace.id {
			debug!("Already on workspace {}", next_workspace.id);
			return Ok(());
		}
	}

	if next_workspace.id < 0 {
		toggle_special_workspace(&next_workspace.name, dry_run).with_context(|| {
			format!(
				"Failed to execute toggle workspace with name {}",
				next_workspace.name
			)
		})?;
	} else {
		switch_normal_workspace(next_workspace.id, dry_run).with_context(|| {
			format!(
				"Failed to execute switch workspace with id {}",
				next_workspace.id
			)
		})?;
	}
	Ok(())
}

fn switch_client(address: &Address, dry_run: bool) -> anyhow::Result<()> {
	if dry_run {
		#[allow(clippy::print_stdout)]
		{
			println!("switch to next_client: {}", address);
		}
	} else {
		debug!("[EXEC] switch to next_client: {}", address);
		std::process::Command::new("hyprctl")
			.args([
				"eval",
				&format!(
					"hl.dispatch(hl.dsp.focus({{ window = 'address:{}' }}))",
					address
				),
			])
			.output()?;
		std::process::Command::new("hyprctl")
			.args(["eval", "hl.dispatch(hl.dsp.window.bring_to_top())"])
			.output()?;
	}

	Ok(())
}

fn switch_normal_workspace(workspace_id: WorkspaceId, dry_run: bool) -> anyhow::Result<()> {
	if dry_run {
		#[allow(clippy::print_stdout)]
		{
			println!("switch to workspace {workspace_id}");
		}
	} else {
		debug!("[EXEC] switch to workspace {workspace_id}");
		std::process::Command::new("hyprctl")
			.args([
				"eval",
				&format!(
					"hl.dispatch(hl.dsp.focus({{ workspace = '{}' }}))",
					workspace_id
				),
			])
			.output()?;
	}
	Ok(())
}

fn toggle_special_workspace(workspace_name: &str, dry_run: bool) -> anyhow::Result<()> {
	let name = workspace_name
		.strip_prefix("special:")
		.unwrap_or(workspace_name)
		.to_string();

	if dry_run {
		#[allow(clippy::print_stdout)]
		{
			println!("toggle workspace {name}");
		}
	} else {
		debug!("[EXEC] toggle workspace {name}");
		std::process::Command::new("hyprctl")
			.args([
				"eval",
				&format!("hl.dispatch(hl.dsp.workspace.toggle_special('{}'))", name),
			])
			.output()?;
	}
	Ok(())
}
