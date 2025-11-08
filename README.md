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

- launch applications from the GUI <!-- update -->
- support for plugging in new monitors while running [only when run as systemd service]
- automatically restart when version changes [only when run as systemd service]
- create all binds and configs from a single config file
- TODO: add more experimental features to this list


![image.png](imgs/image_4.png)

## Table of Contents

- [Install](#install)
- [Usage](#usage)
- [Theming](#theming---custom-css)
- [Other](#other)
	- [Sorting of windows](#sorting-of-windows)
	- [Experimental Environment Variables](#experimental-environment-variables)
	- [Migration to 3.0.0](#migration-to-300)


## Install
[![Packaging status](https://repology.org/badge/vertical-allrepos/hyprswitch.svg)](https://repology.org/project/hyprswitch/versions)  
(Hyprland >= 0.42 required)  

### From Source


Build using cargo (in the root directory of this project):
```sh
cargo build --locked --release
```

runtime dependencies:  
gtk4 [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell)


<!-- `cargo install hyprswitch` -->

### Arch Linux

```sh
paru -S hyprswitch
# or
yay -S hyprswitch
```

### Nixos

- add ``hyprswitch.url = "github:egnrse/hyprswitch/release";`` to flake inputs
- add `specialArgs = { inherit inputs; };` to `nixpkgs.lib.nixosSystem`
- add `inputs.hyprswitch.packages.x86_64-linux.default` to your `environment.systemPackages`
- available systems: `aarch64-linux`, `i686-linux`, `riscv32-linux`, `riscv64-linux`, `x86_64-linux`


## Usage

To use the GUI, you need to start the daemon first with eg. `hyprswitch init`. It is recommended to start the daemon through hyprland by putting `exec-once = hyprswitch init &` into your [hyprland config](https://wiki.hypr.land/Configuring/).

Subsequent calls to hyprswitch (with the  `gui`/`dispatch`/`close` commands) will send the command to the daemon which will execute the command and update the GUI.


## Parameters

**For a full updated list use the `--help` argument (eg. `hyprswitch gui --help`).**

- `--help`/`-h` shows help (also works with subcommands)
- `--dry-run`/`-d` print the command that would be executed instead of executing it (daemon/simple doesn't switch, client doesn't send command to daemon)
- `-v` increase the verbosity level (`-v`: debug, `-vv`: trace) (use the [RUST_LOG](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) env-var for more control)
- `--quiet`/`-q` turn off all output (except when using `--dry-run`)

- `init` initialize and start the daemon
    - `--custom-css <PATH>` specify a path to custom CSS file
    - `--show-title` [default=true] show the window title instead of its class in overview (fallback to class if title is empty)
    - `--workspaces-per-row` [default=5] limit amount of workspaces in one row (overflows to next row)
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
    - `--close <TYPE>` how to close hyprswitch  (`Return` or clicking a window always closes, `ESC` always kills)  
	Options:
        - `default` [default] close when pressing the `mod key` + `key` again (eg. `SUPER + TAB`) or an index key (eg. 1, 2, 3)
        - `mod-key-release` close when releasing the `mod key` (eg. `SUPER`)
    - `--max-switch-offset <MAX_SWITCH_OFFSET>` [default=6] the maximum offset you can switch to, with number keys (0 disables number keys switching and hides indexes in the GUI)
    - `--hide-active-window-border` hide the active window border in the GUI (also hides the border for the selected workspace and monitor)
    - `--monitors` show the GUI only on this monitor [default: display on all monitors]  
	Get the available values by executing: `hyprctl monitors -j | jq '.[].name'`  
	Examples: `--monitors=HDMI-0,DP-1` / `--monitors=eDP-1`  
	(You might want to use this together `-show-workspaces-on-all-monitors` as using arrow keys to select windows on different monitors will still be possible. Or use `--filter-current-monitor` to only show windows of the current monitor.)
    - `--show-workspaces-on-all-monitors` show all workspaces on all monitors [default: only show workspaces on the corresponding monitor]
	- gui also supports the options from `simple` (except `--offset` and `--reverse`)

- `simple` switch without using the GUI / Daemon (switches directly)
    - `--reverse`/`-r` reverse the order of windows / switch backwards
    - `--offset`/`-o <OFFSET>` switch to a specific window offset (default 1)
    - `--include-special-workspaces` include special workspaces (eg. scratchpad)
    - `--filter-same-class`/`-s` only switch between windows that have the same class/type as the currently focused
      window
    - `--filter-current-workspace`/`-w` only switch between windows that are on the same workspace as the currently
      focused window
    - `--filter-current-monitor`/`-m` only switch between windows that are on the same monitor as the currently focused
      window
    - `--sort-recent` sort windows by most recently focused (only works with `--switch-type client`)
    - `--switch-type` switches to next/previous workspace/client/monitor
        - `client` [default] switch to next/previous client
        - `workspace` switch to next/previous workspace
        - `monitor` switch to next/previous monitor


## Examples

The following code blocks belong into your [hyprland config](https://wiki.hypr.land/Configuring/). Modify the $... variables to use the keys you prefer.

### GUI

**Simple**: Press `super` + `$key(tab)` to open the GUI, use mouse to click on window or press `1` / `2` / ... to switch to index.

```ini
exec-once = hyprswitch init --show-title --size-factor 5.5 --workspaces-per-row 5 &

$key = tab
$mod = super
bind = $mod , $key, exec, hyprswitch gui --mod-key $mod --key $key --max-switch-offset 9 --hide-active-window-border
```

**Simple Arrow keys**: Press `super` + `$key(tab)` to open the GUI, or press `1` / `2` / ... or arrow keys to change selected window, `return` to switch.

```ini
exec-once = hyprswitch init --show-title --size-factor 5.5 --workspaces-per-row 5 &

$key = tab
$mod = super
bind = $mod, $key, exec, hyprswitch gui --mod-key $mod --key $key --max-switch-offset 9
```

**Keyboard (reverse = grave / \` )**: Press `alt` + `$key(tab)` to open the GUI _(and switch to next window)_, hold `alt`, press `$key(tab)` repeatedly to switch to the next window, press ``$reverse(`)`` to switch backwards, release alt to switch.

```ini
exec-once = hyprswitch init --show-title &
$key = tab
$mod = alt
$reverse = grave

bind = $mod, $key, exec, hyprswitch gui --mod-key $mod --key $key --close mod-key-release --reverse-key=key=$reverse && hyprswitch dispatch
bind = $mod, $reverse, exec, hyprswitch gui --mod-key $mod --key $key --close mod-key-release --reverse-key=key=$reverse && hyprswitch dispatch -r

# use the following, if switching to the next window with the opening keypress is unwanted
#bind = alt, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=key=$reverse
#bind = $mod, $reverse, exec, hyprswitch gui --mod-key $mod --key $key --close mod-key-release --reverse-key=key=$reverse
```

**Keyboard recent (reverse = grave / \` )**: Press `alt` + `$key(tab)` to open the GUI _(and switch to previously used window)_, hold `alt`, press `$key(tab)` repeatedly to switch to the less and less previously used window, press ``$reverse(`)`` to switch to more recent used windows, release alt to switch.

```ini
exec-once = hyprswitch init --show-title &
$key = tab
$mod = alt
$reverse = grave

bind = $mod, $key, exec, hyprswitch gui --mod-key $mod --key $key --close mod-key-release --reverse-key=key=$reverse --sort-recent && hyprswitch dispatch
bind = $mod $reverse, $key, exec, hyprswitch gui --mod-key $mod --key $key --close mod-key-release --reverse-key=key=$reverse --sort-recent && hyprswitch dispatch -r

# use the following, if switching to the next window with the opening keypress is unwanted
#bind = $mod, $key, exec, hyprswitch gui --mod-key $mod --key $key --close mod-key-release --reverse-key=key=$reverse
#bind = alt $reverse, $key, exec, hyprswitch gui --mod-key $mod --key $key --close mod-key-release --reverse-key=key=$reverse
```

You can find more examples in the [Wiki](https://github.com/egnrse/hyprswitch/wiki/Examples).


## Theming (`--custom-css`)

### CSS Variables

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

### Example custom CSS for 4K screen to override default CSS values:

```css
/* light blue borders for active, more transparent bg and more border-radius */
:root {
    --border-color-active: rgba(17, 170, 217, 0.9);
    --bg-color: rgba(20, 20, 20, 0.8);
    --border-radius: 15px;
}

/* more margin around image for 4K screen */
.client-image {
    margin: 15px;
}

/* increased index for 4K screen */
.index {
    margin: 10px;
    font-size: 25px;
}

/* increased font size for 4K screen */
.workspace {
    font-size: 35px;
}

/* increased font size for 4K screen */
.client {
    font-size: 25px;
}
```

See [Wiki](https://github.com/egnrse/hyprswitch/wiki/CSS) for more info or look at [default.css](src/daemon/gui/defaults.css), [windows.css](src/daemon/gui/windows/windows.css) and [launcher.css](src/daemon/gui/launcher/launcher.css) for the default CSS styles.

## Other

### Sorting of windows
Here some examples of the order in which windows will appear in the hyprswitch GUI.

```
   1      2  3      4
1  +------+  +------+
2  |  1   |  |  2   |
3  |      |  +------+
4  +------+  +------+
5  +------+  |  4   |
6  |  3   |  |      |
7  +------+  +------+
   1      2  3      4
```

```
      Workspace 1           Workspace 2
1  +------+  +------+ | +------+  +------+
2  |  1   |  |  2   |   |  5   |  |  6   |
3  |      |  |      | | |      |  +------+
4  +------+  +------+   +------+  +------+
5  +------+  +------+ | +------+  |  8   |
6  |  3   |  |  4   |   |  7   |  |      |
7  +------+  +------+ | +------+  +------+
   1      2  3      4   1      2  3      4
```

```
      1       3    5   6     8   10  11  12
   +----------------------------------------+
1  |  +-------+                      +---+  |
2  |  |   1   |              +---+   | 5 |  |
3  |  |       |    +---+     | 3 |   |   |  |
4  |  +-------+    | 2 |     +---+   |   |  |
5  |               +---+     +---+   |   |  |
6  |                         | 4 |   |   |  |
7  |    +-------+            +---+   +---+  |
8  |    |   6   |         +----+            |
9  |    |       |         | 7  |            |
10 |    +-------+         +----+            |
   +----------------------------------------+
        2       4         7    9
```

### Experimental Environment Variables

These variables are subject to change and might be removed in the future (activate debug mode with -v and look for `ENV dump:` in the logs to see the current values or inside the [envs.rs](./src/envs.rs) file)

- `REMOVE_HTML_FROM_WORKSPACE_NAME` bool [default: true]: Remove HTML tag (currently only `<span>{}</span>`) from workspace name
- `DISABLE_TOASTS` bool [default: false]: Disable toasts when errors in the daemon or keybinds are detected

### Migration to 3.0.0

1. The complex Config has been removed in favor of a simpler config.
2. More GUI - CLI options added. (`--mod-key` / `--switch-type` / ...)
3. Removed some cli args. (`--do-initial-execute`, `--stay-open-on-close`)

See [Wiki](https://github.com/egnrse/hyprswitch/wiki/Migration-from-2.x.x-to-3.0.0) for more details.
