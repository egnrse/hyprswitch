use crate::config::config_structs::{
    Bind, Config, FilterBy, General, HoldBindConfig, Other, PressBindConfig, Reverse,
    SimpleBindConfig, ToKey,
};
use rand::RngExt;
use std::env;
use std::path::PathBuf;
use tracing::trace;

pub fn create_binds_and_submaps(
    exe: Option<PathBuf>,
    config: Config,
) -> anyhow::Result<Vec<String>> {
    let mut script_lines = Vec::<String>::new();
    let current_exe = if let Some(exe) = exe {
        exe.into_os_string()
    } else {
        let exe = env::current_exe()?;
        exe.into_os_string()
    };
    let current_exe = current_exe.to_string_lossy();
    trace!("current_exe: {}", current_exe);

    let rand_id = rand::rng().random_range(10..=99);

    generate_daemon_start(&mut script_lines, config.general, &current_exe);
    for (i, bind) in config.binds.into_iter().enumerate() {
        let submap_name = format!("hyprswitch-{rand_id}-{i}");
        match bind {
            Bind::Press(press) => {
                generate_press(&mut script_lines, &current_exe, press, submap_name)
            }
            Bind::Hold(hold) => generate_hold(&mut script_lines, &current_exe, hold, submap_name),
            Bind::Simple(simple) => generate_simple(&mut script_lines, &current_exe, simple),
        }
    }

    Ok(script_lines)
}

fn generate_common_gui(params: &mut Vec<String>, other: &Other) {
    params.push(format!("--max-switch-offset={}", other.max_switch_offset));
    params.push(format!(
        "--hide-active-window-border={}",
        other.hide_active_window_border
    ));
    if let Some(monitors) = &other.monitors {
        params.push(format!("--monitors={}", monitors.join(",")));
    }
    params.push(format!(
        "--show-workspaces-on-all-monitors={}",
        other.show_workspaces_on_all_monitors
    ));
}

fn generate_other(params: &mut Vec<String>, other: &Other) {
    params.push(format!(
        "--include-special-workspaces={}",
        other.include_special_workspaces
    ));
    if other.sort_by_recent {
        params.push("--sort-recent".to_string());
    }
    params.push(format!("--switch-type={}", other.switch_type));
    if let Some(filters) = &other.filter_by {
        for filter in filters {
            params.push(match filter {
                FilterBy::CurrentMonitor => "--filter-current-monitor".to_string(),
                FilterBy::CurrentWorkspace => "--filter-current-workspace".to_string(),
                FilterBy::SameClass => "--filter-same-class".to_string(),
            });
        }
    }
}

fn bind(mods: &str, k: &str, cmd: &str, rt: bool) -> String {
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
    format!("hl.bind('{}', hl.dsp.exec_cmd('{}'){})", keys, cmd, opts)
}

fn generate_simple(script_lines: &mut Vec<String>, current_exe: &str, simple: SimpleBindConfig) {
    let mut params = Vec::<String>::new();
    if simple.reverse {
        params.push("--reverse".to_string());
    }
    params.push(format!("--offset={}", simple.offset));
    generate_other(&mut params, &simple.other);

    script_lines.push(bind(
        &simple.open.modifier.to_string(),
        &simple.open.key.to_key(),
        &format!("{} simple {}", current_exe, params.join(" ")),
        false,
    ));
}

fn generate_press(
    script_lines: &mut Vec<String>,
    current_exe: &str,
    press: PressBindConfig,
    submap_name: String,
) {
    let mut params = Vec::<String>::new();
    params.push(format!("--submap={}", submap_name));
    params.push(format!("--reverse-key={}", press.navigate.reverse));
    generate_other(&mut params, &press.other);
    generate_common_gui(&mut params, &press.other);

    script_lines.push(bind(
        &press.open.modifier.to_string(),
        &press.open.key.to_key(),
        &format!("{} gui-no-submap {}", current_exe, params.join(" ")),
        false,
    ));

    script_lines.push(format!("hl.define_submap('{}', function()", submap_name));

    if press.close.escape {
        script_lines.push(format!(
            "    {}",
            bind(
                "",
                "escape",
                &format!("{} close --kill", current_exe),
                false
            )
        ));
    }
    if press.close.close_on_reopen {
        script_lines.push(format!(
            "    {}",
            bind(
                &press.open.modifier.to_string(),
                &press.open.key.to_key(),
                &format!("{} close --kill", current_exe),
                false
            )
        ));
    }
    script_lines.push(format!(
        "    {}",
        bind("", "return", &format!("{} close", current_exe), false)
    ));

    for i in 1..=9 {
        script_lines.push(format!(
            "    {}",
            bind(
                "",
                &i.to_string(),
                &format!(
                    "{} dispatch --offset={} && {} close",
                    current_exe, i, current_exe
                ),
                false
            )
        ));
        if let Reverse::Mod(modk) = &press.navigate.reverse {
            script_lines.push(format!(
                "    {}",
                bind(
                    &modk.to_string(),
                    &i.to_string(),
                    &format!(
                        "{} dispatch --offset={} --reverse && {} close",
                        current_exe, i, current_exe
                    ),
                    false
                )
            ));
        };
    }

    if press.navigate.arrow_keys {
        script_lines.push(format!(
            "    {}",
            bind("", "right", &format!("{} dispatch", current_exe), false)
        ));
        script_lines.push(format!(
            "    {}",
            bind(
                "",
                "left",
                &format!("{} dispatch --reverse", current_exe),
                false
            )
        ));
    }

    script_lines.push(format!(
        "    {}",
        bind(
            "",
            &press.navigate.forward.to_string(),
            &format!("{} dispatch", current_exe),
            false
        )
    ));
    match press.navigate.reverse {
        Reverse::Key(key) => script_lines.push(format!(
            "    {}",
            bind(
                "",
                &key.to_string(),
                &format!("{} dispatch --reverse", current_exe),
                false
            )
        )),
        Reverse::Mod(modk) => script_lines.push(format!(
            "    {}",
            bind(
                &modk.to_string(),
                &press.navigate.forward.to_string(),
                &format!("{} dispatch --reverse", current_exe),
                false
            )
        )),
    }

    script_lines.push("    hl.bind('escape', hl.dsp.submap('reset'))".to_string());
    script_lines.push("end)\n".to_string());
}

fn generate_hold(
    script_lines: &mut Vec<String>,
    current_exe: &str,
    hold: HoldBindConfig,
    submap_name: String,
) {
    let mut params = Vec::<String>::new();
    params.push(format!("--submap={}", submap_name));
    params.push(format!("--reverse-key={}", hold.navigate.reverse));
    generate_other(&mut params, &hold.other);
    generate_common_gui(&mut params, &hold.other);

    script_lines.push(bind(
        &hold.open.modifier.to_string(),
        &hold.navigate.forward.to_string(),
        &format!(
            "{} gui-no-submap {} && {} dispatch",
            current_exe,
            params.join(" "),
            current_exe
        ),
        false,
    ));

    match &hold.navigate.reverse {
        Reverse::Key(key) => script_lines.push(bind(
            &hold.open.modifier.to_string(),
            &key.to_string(),
            &format!(
                "{} gui-no-submap {} && {} dispatch --reverse",
                current_exe,
                params.join(" "),
                current_exe
            ),
            false,
        )),
        Reverse::Mod(modk) => script_lines.push(bind(
            &format!("{} {}", hold.open.modifier, modk),
            &hold.navigate.forward.to_string(),
            &format!(
                "{} gui-no-submap {} && {} dispatch --reverse",
                current_exe,
                params.join(" "),
                current_exe
            ),
            false,
        )),
    }

    script_lines.push(format!("hl.define_submap('{}', function()", submap_name));
    if hold.close.escape {
        script_lines.push(format!(
            "    {}",
            bind(
                "",
                "escape",
                &format!("{} close --kill", current_exe),
                false
            )
        ));
    }

    script_lines.push(format!(
        "    {}",
        bind(
            &hold.open.modifier.to_string(),
            &hold.navigate.forward.to_string(),
            &format!("{} dispatch", current_exe),
            false
        )
    ));
    match &hold.navigate.reverse {
        Reverse::Key(key) => script_lines.push(format!(
            "    {}",
            bind(
                &hold.open.modifier.to_string(),
                &key.to_string(),
                &format!("{} dispatch --reverse", current_exe),
                false
            )
        )),
        Reverse::Mod(modk) => script_lines.push(format!(
            "    {}",
            bind(
                &format!("{} {}", hold.open.modifier, modk),
                &hold.navigate.forward.to_string(),
                &format!("{} dispatch --reverse", current_exe),
                false
            )
        )),
    }

    script_lines.push(format!(
        "    {}",
        bind(
            &hold.open.modifier.to_string(),
            &hold.navigate.forward.to_string(),
            &format!("{} close", current_exe),
            true
        )
    ));
    if let Reverse::Mod(modk) = &hold.navigate.reverse {
        script_lines.push(format!(
            "    {}",
            bind(
                &format!("{} {}", hold.open.modifier, modk),
                &hold.navigate.forward.to_string(),
                &format!("{} close", current_exe),
                true
            )
        ));
    };

    for i in 1..=9 {
        script_lines.push(format!(
            "    {}",
            bind(
                &hold.open.modifier.to_string(),
                &i.to_string(),
                &format!(
                    "{} dispatch --offset={} && {} close",
                    current_exe, i, current_exe
                ),
                false
            )
        ));
        if let Reverse::Mod(modk) = &hold.navigate.reverse {
            script_lines.push(format!(
                "    {}",
                bind(
                    &format!("{} {}", hold.open.modifier, modk),
                    &i.to_string(),
                    &format!(
                        "{} dispatch --offset={} --reverse && {} close",
                        current_exe, i, current_exe
                    ),
                    false
                )
            ));
        };
    }

    if hold.navigate.arrow_keys {
        script_lines.push(format!(
            "    {}",
            bind(
                &hold.open.modifier.to_string(),
                "right",
                &format!("{} dispatch", current_exe),
                false
            )
        ));
        script_lines.push(format!(
            "    {}",
            bind(
                &hold.open.modifier.to_string(),
                "left",
                &format!("{} dispatch --reverse", current_exe),
                false
            )
        ));
    }

    script_lines.push("    hl.bind('escape', hl.dsp.submap('reset'))".to_string());
    script_lines.push("end)\n".to_string());
}

fn generate_daemon_start(script_lines: &mut Vec<String>, general: General, current_exe: &str) {
    let mut params = Vec::<String>::new();
    let mut envs = Vec::<String>::new();

    envs.push(format!("DISABLE_TOASTS={}", general.disable_toast));
    params.push(format!("--size_factor={}", general.size_factor));
    params.push(format!("--show-title={}", general.gui.show_title));
    params.push(format!(
        "--workspaces-per-row={}",
        general.gui.workspaces_per_row
    ));
    envs.push(format!(
        "REMOVE_HTML_FROM_WORKSPACE_NAME={}",
        general.gui.strip_html_from_title
    ));
    envs.push(format!("ICON_SIZE={}", general.gui.icon_size));
    envs.push(format!(
        "SHOW_DEFAULT_ICON={}",
        general.gui.show_default_icon
    ));

    if let Some(custom_css_path) = general.custom_css_path {
        params.push(format!("--custom-css={}", custom_css_path));
    }

    script_lines.push(format!(
        "hl.on('init', function() hl.dispatch(hl.dsp.exec_cmd('{} {} init {}')) end)\n",
        envs.join(" "),
        current_exe,
        params.join(" ")
    ));
}

pub fn export(list: Vec<String>) -> String {
    list.join("\n")
}
