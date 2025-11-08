use crate::daemon::deactivate_submap;
use crate::daemon::gui::reload_desktop_maps;
use crate::handle::{clear_recent_clients, switch_to_active};
use crate::{global, Active, GUISend, Share, UpdateCause, Warn};
use gtk4::glib::clone;
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use std::ops::Deref;
use std::thread;
use tracing::trace;

pub(crate) fn gui_set_client(share: &Share, address: Address) {
    let (latest, _, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Some(Active::Client(address));
        drop(lock);
    }
}

pub(crate) fn gui_set_workspace(share: &Share, id: WorkspaceId) {
    let (latest, _, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Some(Active::Workspace(id));
        drop(lock);
    }
}

pub(crate) fn gui_set_monitor(share: &Share, id: MonitorId) {
    let (latest, _, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Some(Active::Monitor(id));
        drop(lock);
    }
}


pub(crate) fn gui_close(share: &Share) {
    thread::spawn(clone!(
        #[strong]
        share,
        move || {
            deactivate_submap();
            *(global::OPEN
                .get()
                .expect("ACTIVE not set")
                .lock()
                .expect("Failed to lock")) = false;

            let (latest, send, receive) = share.deref();

            trace!("Sending hide to GUI");
            send.send_blocking((GUISend::Hide, UpdateCause::GuiClick))
                .warn("Unable to hide the GUI");
            let rec = receive.recv_blocking().warn("Unable to receive GUI update");
            trace!("Received hide finish from GUI: {rec:?}");

            {
                let lock = latest.lock().expect("Failed to lock");
                switch_to_active(lock.active.as_ref(), &lock.hypr_data).warn("Failed to switch");
                drop(lock);
            }

            clear_recent_clients();
            reload_desktop_maps();
        }
    ));
}
