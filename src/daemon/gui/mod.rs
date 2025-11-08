use crate::{GUISend, InitConfig, Payload, Share, SubmapConfig, UpdateCause, Warn};
use anyhow::Context;
use async_channel::{Receiver, RecvError, Sender};
use gtk4::gdk::{Display, Monitor};
use gtk4::glib::{clone, GString};
use gtk4::prelude::{
    ApplicationExt, ApplicationExtManual, EditableExt, GtkWindowExt, MonitorExt, WidgetExt,
};
use gtk4::{
    glib, style_context_add_provider_for_display, Application, ApplicationWindow, CssProvider,
    Entry, FlowBox, Label, ListBox, Overlay, STYLE_PROVIDER_PRIORITY_APPLICATION,
    STYLE_PROVIDER_PRIORITY_USER,
};
use gtk4_layer_shell::{Edge, LayerShell};
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use std::cmp::max;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;
use tracing::{debug, error, info, span, trace, warn, Level};

pub use debug::{debug_desktop_files, debug_list, debug_search_class};
pub use maps::reload_desktop_maps;

mod debug;
mod gui_handle;
mod icon;
mod maps;
mod windows;

use crate::daemon::gui::maps::init_icon_map;

pub(super) fn start_gui_blocking(
    share: Share,
    init_config: InitConfig,
    receiver: Receiver<Payload>,
    return_sender: Sender<Option<Payload>>,
) {
    #[cfg(debug_assertions)]
    let application = Application::builder()
        .application_id("com.github.h3rmt.hyprswitch.debug")
        .build();
    #[cfg(not(debug_assertions))]
    let application = Application::builder()
        .application_id("com.github.h3rmt.hyprswitch")
        .build();

    application.connect_activate(move |app| {
        trace!("start connect_activate");
        check_themes();

        // load all installed icons
        // https://github.com/H3rmt/hyprswitch/discussions/137
        init_icon_map();

        apply_css(init_config.custom_css.as_ref());


        let (visibility_sender, visibility_receiver) = async_channel::unbounded();
        let monitor_data_list: Rc<Mutex<HashMap<ApplicationWindow, (MonitorData, Monitor)>>> =
            Rc::new(Mutex::new(HashMap::new()));
        {
            let mut monitor_data_list = monitor_data_list.lock().expect("Failed to lock");
            windows::create_windows(
                app,
                &share,
                &mut monitor_data_list,
                init_config.workspaces_per_row as u32,
                visibility_sender.clone(),
            )
            .warn("Failed to create windows");
            drop(monitor_data_list);
        }

        glib::spawn_future_local(clone!(
            #[strong]
            share,
            #[strong]
            monitor_data_list,
            #[strong]
            init_config,
            #[strong]
            receiver,
            #[strong]
            return_sender,
            async move {
                loop {
                    trace!("Waiting for GUI update");
                    let mess = receiver.recv().await;
                    handle_update(
                        &share,
                        &init_config,
                        &mess,
                        monitor_data_list.clone(),
                        visibility_receiver.clone(),
                    )
                    .await;

                    return_sender
                        .send(mess.clone().ok())
                        .await
                        .expect("Failed to send return_sender");
                    trace!("GUI update finished: {mess:?}");
                }
            }
        ));
    });
    info!("Running application");
    application.run_with_args::<String>(&[]);
    error!("Application exited");
}

async fn handle_update(
    share: &Share,
    init_config: &InitConfig,
    mess: &Result<Payload, RecvError>,
    monitor_data: Rc<Mutex<HashMap<ApplicationWindow, (MonitorData, Monitor)>>>,
    visibility_receiver: Receiver<bool>,
) {
    let (shared_data, _, _) = share.deref();

    trace!("Received GUI update: {mess:?}");
    match mess {
        Ok((GUISend::New, ref update_cause)) => {
            let _span = span!(Level::TRACE, "new", cause = update_cause.to_string()).entered();
            let windows = {
                let data = shared_data.lock().expect("Failed to lock, shared_data");
                let mut monitor_data = monitor_data.lock().expect("Failed to lock, monitor_data");

                let mut windows = 0;
                for (window, (monitor_data, monitor)) in monitor_data.iter_mut() {
                    if let Some(monitors) = &data.gui_config.monitors {
                        if !monitors.iter().any(|m| *m == monitor_data.connector) {
                            continue;
                        }
                    }

					window.set_anchor(Edge::Bottom, false);
                    

                    trace!("Showing window {:?}", window);
                    windows += 1;
                    window.set_visible(true);

                    windows::init_windows(
                        share.clone(),
                        &data.hypr_data.workspaces,
                        &data.hypr_data.clients,
                        monitor_data,
                        init_config.show_title,
                        data.gui_config.show_workspaces_on_all_monitors,
                        init_config.size_factor,
                    );

                    trace!("Refresh window {:?}", window);
                    windows::update_windows(monitor_data, &data).warn("Failed to update windows");
                }

                drop(data);
                drop(monitor_data);
                windows // use scope to drop locks and prevent hold MutexGuard across await
            };
            // waits until all windows are visible
            trace!("Waiting for {windows} windows to show");
            for _ in 0..windows {
                // receive async not to block gtk event loop
                visibility_receiver.recv().await.expect("Failed to receive");
            }
        }
        Ok((GUISend::Refresh, ref update_cause)) => {
            let _span = span!(Level::TRACE, "refresh", cause = update_cause.to_string()).entered();
            let mut data = shared_data.lock().expect("Failed to lock, shared_data");
            let mut monitor_data = monitor_data.lock().expect("Failed to lock, monitor_data");

            for (window, (monitor_data, _)) in &mut monitor_data.iter_mut() {
                if let Some(monitors) = &data.gui_config.monitors {
                    if !monitors.iter().any(|m| *m == monitor_data.connector) {
                        continue;
                    }
                }
                trace!("Refresh window {:?}", window);
                windows::update_windows(monitor_data, &data).warn("Failed to update windows");
            }
        }
        Ok((GUISend::Hide, ref update_cause)) => {
            let _span = span!(Level::TRACE, "hide", cause = update_cause.to_string()).entered();
            let windows = {
                let data = shared_data.lock().expect("Failed to lock, shared_data");
                let monitor_data = monitor_data.lock().expect("Failed to lock, monitor_data");

                let mut windows = 0;
                for window in (*monitor_data).keys() {
                    trace!("Hiding window {:?}", window);
                    windows += 1;
                    window.set_visible(false);
                }

                drop(data);
                drop(monitor_data);
                windows // use scope to drop locks and prevent hold MutexGuard across await
            };
        }
        Ok((GUISend::Exit, ref update_cause)) => {
            let _span = span!(Level::TRACE, "exit", cause = update_cause.to_string()).entered();
            let monitor_data = monitor_data.lock().expect("Failed to lock, monitor_data");

            for window in (*monitor_data).keys() {
                trace!("Closing window {:?}", window);
                window.close();
            }
        }
        Err(e) => {
            warn!("Receiver closed: {e}");
        }
    }
}

fn apply_css(custom_css: Option<&PathBuf>) {
    let provider_app = CssProvider::new();
    provider_app.load_from_data(&format!(
        "{}\n{}",
        include_str!("defaults.css"),
        include_str!("windows/windows.css"),
    ));
    style_context_add_provider_for_display(
        &Display::default()
            .context("Could not connect to a display.")
            .expect("Could not connect to a display."),
        &provider_app,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    if let Some(custom_css) = custom_css {
        if !custom_css.exists() {
            warn!("Custom css file {custom_css:?} does not exist");
        } else {
            let provider_user = CssProvider::new();
            provider_user.load_from_path(custom_css);
            style_context_add_provider_for_display(
                &Display::default()
                    .context("Could not connect to a display.")
                    .expect("Could not connect to a display."),
                &provider_user,
                STYLE_PROVIDER_PRIORITY_USER,
            );
        }
    }
}


pub struct MonitorData {
    id: MonitorId,
    connector: GString,

    // used to store a ref to the FlowBox containing the workspaces
    workspaces_flow: FlowBox,
    // used to store a ref to the overlay over the whole monitor (parent of monitor index)
    workspaces_flow_overlay: (Overlay, Option<Label>),
    // used to store refs to the Overlays over the workspace Frames
    workspace_refs: HashMap<WorkspaceId, (Overlay, Option<Label>)>,
    // used to store refs to the Overlays containing the clients
    client_refs: HashMap<Address, (Overlay, Option<Label>)>,
}

pub fn start_gui_restarter(share: Share) {
    let mut event_listener = hyprland::event_listener::EventListener::new();
    event_listener.add_monitor_added_handler(clone!(
        #[strong]
        share,
        move |data| {
            debug!("Monitor added: {:#?}, restarting GUI", data);
            let (_, s, _) = share.deref();
            s.send_blocking((GUISend::Exit, UpdateCause::BackgroundThread(None)))
                .warn("Failed to send exit");
        }
    ));
    event_listener.add_monitor_removed_handler(clone!(
        #[strong]
        share,
        move |data| {
            debug!("Monitor removed: {:#?}, restarting GUI", data);
            let (_, s, _) = share.deref();
            s.send_blocking((GUISend::Exit, UpdateCause::BackgroundThread(None)))
                .warn("Failed to send exit");
        }
    ));
    event_listener
        .start_listener()
        .warn("Failed to start monitor added/removed listener");
}

pub fn check_themes() {
    if let Some(settings) = gtk4::Settings::default() {
        let theme_name = settings.gtk_theme_name();
        let icon_theme_name = settings.gtk_icon_theme_name();
        let icon_theme = gtk4::IconTheme::new();
        let search_path = icon_theme.search_path();
        info!("Using theme: {theme_name:?} and icon theme: {icon_theme_name:?}, please make sure both exist, else weird icon or graphical issues may occur");
        debug!("Icon theme search path: {search_path:?}");
    } else {
        warn!("Unable to check default settings for icon theme");
    }
}
