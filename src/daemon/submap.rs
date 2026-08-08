use std::env;
use std::process::Command;

use anyhow::Context;
use tracing::{debug, span, trace, Level};

use crate::{CloseType, ModKey, ReverseKey, Warn};

pub(super) fn activate_submap(submap_name: &str) -> anyhow::Result<()> {
	let _span = span!(Level::TRACE, "submap").entered();
	let script = format!("hl.dispatch(hl.dsp.submap('{}'))", submap_name);
	Command::new("hyprctl")
		.args(["eval", &script])
		.output()
		.context("failed to execute hyprctl eval")?
		.status
		.success()
		.then_some(())
		.context("hyprctl eval failed")
		.warn("unable to activate submap");
	debug!("Activated submap: {}", submap_name);
	Ok(())
}

fn generate_submap_name() -> String {
	format!("hyprswitch-{}", rand::random::<u16>())
}

pub(super) fn generate_submap(
	mod_key: ModKey,
	key: String,
	reverse_key: ReverseKey,
	close: CloseType,
) -> anyhow::Result<()> {
	let _span = span!(Level::TRACE, "submap").entered();
	let mut lua_binds = Vec::<String>::new();
	(|| -> anyhow::Result<()> {
		let current_exe = env::current_exe()?;
		let current_exe = current_exe
			.to_str()
			.with_context(|| format!("unable to convert path {:?} to string", current_exe))?
			.trim_end_matches(" (deleted)");
		let main_mod = get_mod_from_mod_key(mod_key.clone());
		trace!("current_exe: {}", current_exe);

		// Helper closure to format bind easily to lua
		let mut bind = |mods: &str, k: &str, cmd: &str, rt: bool| {
			let opts = if rt {
				", { release = true, transparent = true }"
			} else {
				""
			};
			let keys = if mods.is_empty() {
				k.to_string()
			} else {
				format!("{} + {}", mods, k)
			};
			lua_binds.push(format!(
				"hl.bind('{}', hl.dsp.exec_cmd('{}'){})",
				keys, cmd, opts
			));
		};

		// always bind escape to kill
		bind(
			"",
			"escape",
			&format!("{} close --kill", current_exe),
			false,
		);
		bind(
			main_mod,
			"escape",
			&format!("{} close --kill", current_exe),
			false,
		);

		// repeatable presses
		match close {
			CloseType::ModKeyRelease => {
				// allow repeatable presses to switch to next
				bind(main_mod, &key, &format!("{} dispatch", current_exe), false);
				match reverse_key.clone() {
					ReverseKey::Mod(modkey) => {
						bind(
							&format!("{} {}", main_mod, modkey),
							&key,
							&format!("{} dispatch -r", current_exe),
							false,
						);
					}
					ReverseKey::Key(rkey) => {
						bind(
							main_mod,
							&rkey,
							&format!("{} dispatch -r", current_exe),
							false,
						);
					}
				};
			}
			CloseType::Default => {
				bind(
					main_mod,
					&key,
					&format!("{} close --kill", current_exe),
					false,
				);
			}
		};

		// close on release of the mod key
		match close {
			CloseType::ModKeyRelease => {
				bind(
					main_mod,
					&mod_key.to_string(),
					&format!("{} close", current_exe),
					true,
				);
				if let ReverseKey::Mod(modkey) = reverse_key.clone() {
					bind(
						&format!("{} {}", main_mod, modkey),
						&mod_key.to_string(),
						&format!("{} close", current_exe),
						true,
					);
				};
			}
			CloseType::Default => {
				// bind return to close
				bind("", "return", &format!("{} close", current_exe), false);
			}
		};

		// jump to index
		match close {
			CloseType::ModKeyRelease => {
				for i in 1..=9 {
					bind(
						main_mod,
						&i.to_string(),
						&format!("{} dispatch -o={}", current_exe, i),
						false,
					);
					if let ReverseKey::Mod(modkey) = reverse_key.clone() {
						bind(
							&format!("{} {}", main_mod, modkey),
							&i.to_string(),
							&format!("{} dispatch -o={} -r", current_exe, i),
							false,
						);
					};
				}
			}
			CloseType::Default => {
				for i in 1..=9 {
					bind(
						"",
						&i.to_string(),
						&format!("{} dispatch -o={} && {} close", current_exe, i, current_exe),
						false,
					);
					if let ReverseKey::Mod(modkey) = reverse_key.clone() {
						bind(
							&modkey.to_string(),
							&i.to_string(),
							&format!(
								"{} dispatch -o={} -r && {} close",
								current_exe, i, current_exe
							),
							false,
						);
					};
				}
			}
		};

		// use arrow keys to navigate
		match close {
			CloseType::Default => {
				bind("", "right", &format!("{} dispatch", current_exe), false);
				bind("", "left", &format!("{} dispatch -r", current_exe), false);
			}
			CloseType::ModKeyRelease => {
				bind(
					main_mod,
					"right",
					&format!("{} dispatch", current_exe),
					false,
				);
				bind(
					main_mod,
					"left",
					&format!("{} dispatch -r", current_exe),
					false,
				);
			}
		}

		#[cfg(debug_assertions)]
		bind(
			get_mod_from_mod_key(ModKey::AltR),
			"o",
			"kill $(pidof hyprswitch) && hyprctl dispatch submap reset",
			false,
		);

		let name = generate_submap_name();

		lua_binds.push("hl.bind('escape', hl.dsp.submap('reset'))".to_string());

		// Construct the full define_submap lua code
		let lua_script = format!(
			"hl.define_submap('{}', function()\n    {}\nend)",
			name,
			lua_binds.join("\n    ")
		);

		trace!("lua_script: \n{}", lua_script);

		Command::new("hyprctl")
			.args(["eval", &lua_script])
			.output()
			.context("failed to execute hyprctl eval")?
			.status
			.success()
			.then_some(())
			.context("hyprctl eval failed")
			.warn("unable to generate submap");

		// activate the submap
		let script = format!("hl.dispatch(hl.dsp.submap('{}'))", name);
		Command::new("hyprctl").args(["eval", &script]).output()?;

		Ok(())
	})()
	.inspect_err(|_| {
		// reset submap if failed
		let _ = Command::new("hyprctl")
			.args(["eval", "hl.dispatch(hl.dsp.submap('reset'))"])
			.output();
	})?;

	Ok(())
}

pub fn deactivate_submap() {
	let _span = span!(Level::TRACE, "submap").entered();
	Command::new("hyprctl")
		.args(["eval", "hl.dispatch(hl.dsp.submap('reset'))"])
		.output()
		.map(|out| out.status.success().then_some(()))
		.unwrap_or(None)
		.context("hyprctl eval failed")
		.warn("unable to deactivate submap");
	debug!("Deactivated submap");
}

fn get_mod_from_mod_key(mod_key: ModKey) -> &'static str {
	match mod_key {
		ModKey::SuperL | ModKey::SuperR => "SUPER",
		ModKey::AltL | ModKey::AltR => "ALT",
		ModKey::CtrlL | ModKey::CtrlR => "CTRL",
		ModKey::ShiftL | ModKey::ShiftR => "SHIFT",
	}
}
