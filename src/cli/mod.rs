//! handle input args (using clap)

use clap::{Args, Parser, Subcommand};
use std::fmt::Debug;

pub use debug::DebugCommand;

mod debug;
mod dispatch;
mod gui;
mod init;
mod shared;
mod simple;

#[derive(Parser, Debug, Clone)]
#[command(
	author,
	version,
	about,
	long_about = "A CLI/GUI that allows switching between windows in Hyprland\nvisit https://github.com/egnrse/hyprswitch/wiki/Examples to see Example configs"
)]
pub struct App {
	#[clap(flatten)]
	pub global_opts: GlobalOpts,

	#[clap(subcommand)]
	pub command: Command,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalOpts {
	/// Print the command that would be executed instead of executing it
	#[arg(short = 'd', long, global = true)]
	pub dry_run: bool,

	/// Increase the verbosity level (-v: debug, -vv: trace)
	#[arg(short = 'v', action = clap::ArgAction::Count, global = true)]
	pub verbose: u8,

	/// Turn off all output
	#[arg(short = 'q', long, global = true)]
	pub quiet: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
	/// Initialize and start the Daemon
	Init {
		#[clap(flatten)]
		init_opts: init::InitOpts,
	},

	#[clap(hide = true)]
	Dispatch {
		#[clap(flatten)]
		dispatch_config: dispatch::DispatchConf,
	},

	#[clap(hide = true)]
	Gui {
		#[clap(flatten)]
		submap_conf: gui::SubmapConf,

		/// The key used for reverse switching. Format: reverse-key=mod=<MODIFIER> or reverse-key=key=<KEY> (e.g., --reverse-key=mod=shift, --reverse-key=key=grave)
		///
		/// Used for both --submap and --key, --mod-key, --close. --submap needs it to know if the reverse key is a modifier or a key (to display the correct keybinding in the GUI)
		#[arg(long, value_parser = clap::value_parser!(shared::InputReverseKey), default_value = "mod=shift")]
		reverse_key: shared::InputReverseKey,

		#[clap(flatten)]
		gui_conf: gui::GuiConf,

		#[clap(flatten)]
		simple_config: simple::SimpleConf,
	},
	/// Switch without using the GUI / Daemon (switches directly)
	Simple {
		#[clap(flatten)]
		dispatch_config: dispatch::DispatchConf,

		#[clap(flatten)]
		simple_conf: simple::SimpleConf,
	},

	#[clap(hide = true)]
	Close {
		/// Don't switch to the selected window, just close the GUI
		#[arg(long)]
		kill: bool,
	},
	/// Debug command to debug finding icons for the GUI, doesn't interact with the Daemon
	Debug {
		#[clap(subcommand)]
		command: DebugCommand,
	},
}

/// only show error if not caused by --help or --version
pub fn check_invalid_inputs(e: &clap::Error) -> bool {
	matches!(
		e.kind(),
		clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
	)
}

#[cfg(test)]
mod tests {
	use clap::error::ErrorKind;

	use super::*;

	fn parse(args: &[&str]) -> Result<App, clap::Error> {
		App::try_parse_from(std::iter::once("hyprswitch").chain(args.iter().copied()))
	}
	fn parse_ok(args: &[&str]) -> App {
		parse(args).unwrap_or_else(|e| panic!("{args:?} should be valid: {e}"))
	}
	fn error_kind(args: &[&str]) -> ErrorKind {
		match parse(args) {
			Ok(app) => panic!("{args:?} should be rejected, but parsed to {app:?}"),
			Err(e) => e.kind(),
		}
	}
	/// the arguments are invalid (without caring about the exact kind of error clap reports)
	fn assert_rejected(args: &[&str]) {
		error_kind(args);
	}

	/// the `simple` subcommand: `(dispatch options, simple options)`
	fn simple(args: &[&str]) -> (dispatch::DispatchConf, simple::SimpleConf) {
		let args = [&["simple"], args].concat();
		match parse_ok(&args).command {
			Command::Simple {
				dispatch_config,
				simple_conf,
			} => (dispatch_config, simple_conf),
			other => panic!("expected `simple`, got {other:?}"),
		}
	}

	/* simple */

	#[test]
	fn simple_accepts_the_short_options() {
		let (dispatch, simple) = simple(&["-r", "-o", "3", "-s", "-w", "-m"]);

		assert!(dispatch.reverse);
		assert_eq!(dispatch.offset, 3);
		assert!(simple.filter_same_class);
		assert!(simple.filter_current_workspace);
		assert!(simple.filter_current_monitor);
	}

	#[test]
	fn simple_accepts_the_long_options() {
		let (dispatch, simple) = simple(&[
			"--reverse",
			"--offset",
			"2",
			"--filter-same-class",
			"--filter-current-workspace",
			"--filter-current-monitor",
		]);

		assert!(dispatch.reverse);
		assert_eq!(dispatch.offset, 2);
		assert!(simple.filter_same_class);
		assert!(simple.filter_current_workspace);
		assert!(simple.filter_current_monitor);
	}

	#[test]
	fn simple_offset_must_be_a_number_from_1_to_255() {
		assert_rejected(&["simple", "--offset", "0"]);
		assert_rejected(&["simple", "--offset", "256"]);
		assert_rejected(&["simple", "--offset", "-1"]);
		assert_rejected(&["simple", "--offset", "x"]);
		assert_rejected(&["simple", "--offset"]);

		assert_eq!(simple(&["--offset", "1"]).0.offset, 1);
		assert_eq!(simple(&["--offset", "255"]).0.offset, 255);
	}

	/* global options */

	#[test]
	fn global_options_work_before_and_after_the_subcommand() {
		for args in [
			["--dry-run", "-vv", "--quiet", "simple"],
			["simple", "--dry-run", "-vv", "--quiet"],
		] {
			let app = parse_ok(&args);

			assert!(app.global_opts.dry_run, "{args:?}");
			assert!(app.global_opts.quiet, "{args:?}");
			assert_eq!(app.global_opts.verbose, 2, "{args:?}");
		}
	}

	#[test]
	fn global_options_work_on_every_subcommand() {
		for args in [
			&["close", "-d"][..],
			&["dispatch", "-d"],
			&["init", "-d"],
			&["debug", "list", "-d"],
			&["gui", "--mod-key", "ALT", "--key", "tab", "-d"],
		] {
			assert!(parse_ok(args).global_opts.dry_run, "{args:?}");
		}
	}

	#[test]
	fn verbosity_counts_the_flags() {
		assert_eq!(parse_ok(&["simple", "-v"]).global_opts.verbose, 1);
		assert_eq!(parse_ok(&["simple", "-vvv"]).global_opts.verbose, 3);
		assert_eq!(parse_ok(&["simple", "-v", "-v"]).global_opts.verbose, 2);
	}

	/* dispatch / close / debug */

	#[test]
	fn dispatch_subcommand_defaults_to_the_next_item() {
		match parse_ok(&["dispatch"]).command {
			Command::Dispatch { dispatch_config } => {
				assert!(!dispatch_config.reverse);
				assert_eq!(dispatch_config.offset, 1);
			}
			other => panic!("expected `dispatch`, got {other:?}"),
		}
	}

	#[test]
	fn debug_subcommands() {
		assert!(matches!(
			parse_ok(&["debug", "list"]).command,
			Command::Debug {
				command: DebugCommand::List
			}
		));
		assert!(matches!(
			parse_ok(&["debug", "desktop-files"]).command,
			Command::Debug {
				command: DebugCommand::DesktopFiles
			}
		));
		match parse_ok(&["debug", "search", "--class", "firefox"]).command {
			Command::Debug {
				command: DebugCommand::Search { class },
			} => assert_eq!(class, "firefox"),
			other => panic!("expected `debug search`, got {other:?}"),
		}
	}

	/* init */

	#[test]
	fn init_accepts_all_options() {
		let init = match parse_ok(&[
			"init",
			"--custom-css",
			"/tmp/custom.css",
			"--show-title",
			"false",
			"--workspaces-per-row",
			"3",
			"--size-factor",
			"2.5",
		])
		.command
		{
			Command::Init { init_opts } => init_opts,
			other => panic!("expected `init`, got {other:?}"),
		};

		assert_eq!(
			init.custom_css,
			Some(std::path::PathBuf::from("/tmp/custom.css"))
		);
		assert!(!init.show_title);
		assert_eq!(init.workspaces_per_row, 3);
		assert_eq!(init.size_factor, 2.5);
	}

	/* gui */

	#[test]
	fn gui_needs_a_mod_key_and_a_key() {
		assert_eq!(error_kind(&["gui"]), ErrorKind::MissingRequiredArgument);
		assert_eq!(
			error_kind(&["gui", "--mod-key", "SUPER"]),
			ErrorKind::MissingRequiredArgument
		);
		assert_eq!(
			error_kind(&["gui", "--key", "tab"]),
			ErrorKind::MissingRequiredArgument
		);
	}

	#[test]
	fn gui_accepts_the_options_of_simple_too() {
		let args = &[
			"gui",
			"--mod-key",
			"SUPER",
			"--key",
			"tab",
			"--switch-type",
			"workspace",
			"--filter-same-class",
			"--sort-recent",
		];
		let simple = match parse_ok(args).command {
			Command::Gui {
				submap_conf: _,
				reverse_key: _,
				gui_conf: _,
				simple_config,
			} => simple_config,
			other => panic!("expected `gui`, got {other:?}"),
		};

		assert!(matches!(
			simple.switch_type,
			shared::InputSwitchType::Workspace
		));
		assert!(simple.filter_same_class);
		assert!(simple.sort_recent);
	}

	/* invalid input */

	#[test]
	fn a_subcommand_is_required() {
		assert!(parse(&[]).is_err());
	}

	#[test]
	fn help_and_version_are_not_reported_as_invalid_inputs() {
		for args in [&["--help"][..], &["--version"], &["simple", "--help"]] {
			let err = parse(args).expect_err("--help/--version stop the parsing");
			assert!(check_invalid_inputs(&err), "{args:?}");
		}
	}
}
