# hyprswitch (fork)

<!--
[![crates.io](https://img.shields.io/crates/v/hyprswitch.svg)](https://crates.io/crates/hyprswitch)
[![Docs](https://docs.rs/built/badge.svg)](https://docs.rs/hyprswitch)
-->
[![License](https://img.shields.io/github/license/egnrse/hyprswitch)](https://github.com/egnrse/hyprswitch/blob/main/LICENSE)
[![Tests](https://github.com/egnrse/hyprswitch/actions/workflows/rust.yml/badge.svg)](https://github.com/egnrse/hyprswitch/actions/workflows/rust.yml)
[![GitHub tag (latest SemVer pre-release)](https://img.shields.io/github/v/tag/egnrse/hyprswitch?label=version)](https://github.com/egnrse/hyprswitch/releases)

A rust CLI/GUI to switch between windows in [Hyprland](https://github.com/hyprwm/Hyprland). This repo is a fork of [H3rmt/hyprshell](https://github.com/H3rmt/hyprshell) (look at the `*-hyprswitch` branches).


## Features
- cycle through windows using keyboard shortcuts or/and a GUI
- filter windows by class or workspace
- sort windows by their position on the screen
- customize looks with CSS
- customize keybindings

### Experimental Features

- support for plugging in new monitors while running [only when run as systemd service]
- automatically restart when version changes [only when run as systemd service]
- create all binds and configs from a single config file
- TODO: add more experimental features to this list


![image.png](imgs/image_4.png)

## Table of Contents

- [Install](#install)
- [Usage](#usage)
- [Parameters](#parameters)
- [Theming](#theming)
- [Other](#other)
	- [Experimental Environment Variables](#experimental-environment-variables)
	- [Migration to 3.0.0](#migration-to-300)


## Install
[![Packaging status](https://repology.org/badge/vertical-allrepos/hyprswitch.svg)](https://repology.org/project/hyprswitch/versions)  
(Hyprland >= 0.42 required)  

### From Source

Build using cargo:
```sh
git clone https://github.com/egnrse/hyprswitch.git
cd hyprswitch
cargo build --locked --release
```

The executable will be in `./target/debug/hyprswitch`.

Runtime dependencies:  
gtk4 [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell)

<!-- `cargo install hyprswitch` -->

### Arch Linux

```sh
paru -S hyprswitch
# or
yay -S hyprswitch
```

### Nixos

(I dont use nixos, this is not tested.)

- add ``hyprswitch.url = "github:egnrse/hyprswitch/release";`` to flake inputs
- add `specialArgs = { inherit inputs; };` to `nixpkgs.lib.nixosSystem`
- add `inputs.hyprswitch.packages.x86_64-linux.default` to your `environment.systemPackages`
- available systems: `aarch64-linux`, `i686-linux`, `riscv32-linux`, `riscv64-linux`, `x86_64-linux`


## Usage

To use the GUI, you need to start the daemon first with eg. `hyprswitch init`. It is recommended to start the daemon through hyprland by putting `exec-once = hyprswitch init &` into your [hyprland config](https://wiki.hypr.land/Configuring/).

Subsequent calls to hyprswitch (with the  `gui`/`dispatch`/`close` commands) will send the command to the daemon which will execute the command and update the GUI.

The following example opens hyprswitch with `SUPER+TAB`, put it in you hyprland config. (prob. in `~/.config/hypr/hyprland.conf`)
```ini
exec-once = hyprswitch init --show-title --size-factor 6 --workspaces-per-row 5 &

$key = tab
$mod = super
bind = $mod, $key, exec, hyprswitch gui --mod-key $mod --key $key
```

See the [Wiki](https://github.com/egnrse/hyprswitch/wiki/Home#usage) for more infos. You can also find [some examples](https://github.com/egnrse/hyprswitch/wiki/02-%E2%80%90-Examples) in it.


## Parameters

**For the full updated list use the `--help` argument (eg. `hyprswitch gui --help`).**  
For a fuller list see the [Wiki](https://github.com/egnrse/hyprswitch/wiki/Home#parameters).

- `--help`/`-h` show help (also works with subcommands)
- `--dry-run`/`-d` print the command that would be executed instead of executing it (daemon/simple doesn't switch, client doesn't send command to daemon)
- `--quiet`/`-q` turn off all output (except when using `--dry-run`)

- `init` initialize and start the daemon
    - `--custom-css <PATH>` specify a path to custom CSS file
    - `--size-factor` [default=6] the size factor (float) for the GUI (original_size / 30 \* size_factor)

- `gui` opens the GUI
    - `--mod-key <MODIFIER>` [required] the modifier key used to open the GUI  
	Options: super/super_l/super_r, alt/alt_l/alt_r, ctrl/ctrl_l/ctrl_r  
	(You might want to use a variable, see [Examples](#examples))
    - `--key <KEY>` [required] the key to used to open the GUI (eg. tab)  
	(You might want to use a variable, see [Examples](#examples))
    - `--reverse-key <KEYTYPE>=<KEY>` [default=`mod=shift`] the key used for reverse switching  
	Format: `reverse-key=mod=<MODIFIER>` or `reverse-key=key=<KEY>`  
	(eg. `--reverse-key=mod=shift`, `--reverse-key=key=grave`)
	- gui also supports most options from `simple` (except `--offset` and `--reverse`)

- `simple` switch without using the GUI / Daemon (switches directly)
    - `--reverse`/`-r` reverse the order of windows / switch backwards
    - `--offset`/`-o <OFFSET>` switch to a specific window offset (default 1) 
      window
    - `--sort-recent` sort windows by most recently focused (only works with `--switch-type client`)


## Theming
Point the daemon to you custom css file with the `--custom-css` argument. (eg. `hyprswitch init --custom-css $HOME/test.css`)

CSS Variables:
```css
:root {
    --border-color: rgba(90, 90, 120, 0.4);
    --border-color-active: rgba(239, 9, 9, 0.9);
    --bg-color: rgba(20, 20, 20, 1);
    --bg-color-hover: rgba(40, 40, 50, 1);
    --index-border-color: rgba(20, 170, 170, 0.7);
    --border-radius: 12px;
    --border-size: 3px;
}
```

See the [Wiki](https://github.com/egnrse/hyprswitch/wiki/01-%E2%80%90-Theming) for more info or look at [default.css](src/daemon/gui/defaults.css) and [windows.css](src/daemon/gui/windows/windows.css) for the default CSS styles.


## Other

### Experimental Environment Variables

These variables are subject to change and might be removed in the future (activate debug mode with -v and look for `ENV dump:` in the logs to see the current values or inside the [envs.rs](./src/envs.rs) file)

- `REMOVE_HTML_FROM_WORKSPACE_NAME` bool [default: true]: Remove HTML tag (currently only `<span>{}</span>`) from workspace name
- `DISABLE_TOASTS` bool [default: false]: Disable toasts when errors in the daemon or keybinds are detected

### Migration to 3.0.0

1. The complex Config has been removed in favor of a simpler config.
2. More GUI - CLI options added. (`--mod-key` / `--switch-type` / ...)
3. Removed some cli args. (`--do-initial-execute`, `--stay-open-on-close`)

See [Wiki](https://github.com/egnrse/hyprswitch/wiki/Migration-from-2.x.x-to-3.0.0) for more details.
