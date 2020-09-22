use super::errors;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};
use x11rb::connection::Connection;
use x11rb::protocol::res;
use x11rb::protocol::xproto;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ChangeWindowAttributesAux, ConnectionExt, EventMask, Window,
};
use x11rb::protocol::Event;

const SLEEP_TIME: time::Duration = time::Duration::from_millis(50);

pub trait WindowWatcher {
    fn monitor_new_windows<F: Fn(&Self, Window) -> Result<(), Box<dyn std::error::Error>>>(
        &self,
        screen_num: usize,
        keep_running: Arc<AtomicBool>,
        callback: F,
    ) -> Result<(), Box<dyn Error>>;
    fn get_window_pid(&self, window: Window) -> Result<u32, errors::Error>;
}

impl<C: Connection> WindowWatcher for C {
    fn monitor_new_windows<F: Fn(&Self, Window) -> Result<(), Box<dyn std::error::Error>>>(
        &self,
        screen_num: usize,
        keep_running: Arc<AtomicBool>,
        callback: F,
    ) -> Result<(), Box<dyn Error>> {
        let screen = &self.setup().roots[screen_num];

        let window_attributes = ChangeWindowAttributesAux::default()
            .event_mask(EventMask::PropertyChange | EventMask::SubstructureNotify);
        self.change_window_attributes(screen.root, &window_attributes)?
            .check()?;

        self.flush()?;

        let wanted_prop = xproto::intern_atom(self, false, b"STEAM_GAME")?
            .reply()?
            .atom;

        while keep_running.load(Ordering::SeqCst) {
            loop {
                let poll_result = self.poll_for_event();
                let no_more_events = match poll_result {
                    Ok(Some(event)) => {
                        process_event(self, &wanted_prop, &event, &callback);
                        false
                    }
                    Ok(None) => true,
                    Err(_) => {
                        eprintln!("Failed to wait for event");
                        true
                    }
                };

                if no_more_events {
                    thread::sleep(SLEEP_TIME);
                    break;
                }
            }
        }

        Ok(())
    }

    fn get_window_pid(&self, window: Window) -> Result<u32, errors::Error> {
        // Try _NET_WM_PID first, then try XResQueryClientIds
        let pid_prop = xproto::intern_atom(self, false, b"_NET_WM_PID")?.reply()?;

        let net_wm_pid = get_cardinal(self, &pid_prop.atom, window);
        match net_wm_pid {
            Ok(pid) => Ok(pid),
            Err(e) => {
                let query = res::ClientIdSpec {
                    client: window,
                    mask: 0,
                };
                let reply = res::query_client_ids(self, &[query])?.reply()?;
                let id = reply
                    .ids
                    .iter()
                    .filter(|v| {
                        v.spec.mask == res::ClientIdMask::LocalClientPID.into()
                            && !v.value.is_empty()
                    })
                    .map(|v| v.value[0])
                    .next();

                if let Some(pid) = id {
                    Ok(pid)
                } else {
                    eprintln!("Reply: {:?}", reply);
                    Err(e)
                }
            }
        }
    }
}

fn process_event<C, F>(conn: &C, wanted_prop: &Atom, event: &Event, callback: F)
where
    C: Connection,
    F: Fn(&C, Window) -> Result<(), Box<dyn std::error::Error>>,
{
    match event {
        Event::CreateNotify(create_event) => {
            process_create_event(conn, &wanted_prop, &create_event, &callback);
        }
        Event::PropertyNotify(property_event) => {
            process_property_event(conn, &wanted_prop, &property_event, &callback);
        }
        // We should not get any other events, but if we do, we don't care
        _ => (),
    };
}

fn process_create_event<C, F>(
    conn: &C,
    wanted_prop: &Atom,
    event: &xproto::CreateNotifyEvent,
    callback: F,
) where
    C: Connection,
    F: Fn(&C, Window) -> Result<(), Box<dyn std::error::Error>>,
{
    // TODO: We can probably just use get_property here instead of listing them all, I _think_ we get a response
    // with format == 0 when the property doesn't exist
    if let Ok(props_req) = xproto::list_properties(conn, event.window) {
        // Some short lived windows will result in errors here, but that means the windows are
        // gone so we don't care about their props
        if let Ok(props) = props_req.reply() {
            for atom in props.atoms {
                if atom == *wanted_prop {
                    let result = callback(conn, event.window);
                    errors::log_if_failed(&result);
                    break;
                }
            }
        }
    }
}

fn process_property_event<C, F>(
    conn: &C,
    wanted_prop: &Atom,
    event: &xproto::PropertyNotifyEvent,
    callback: F,
) where
    C: Connection,
    F: Fn(&C, Window) -> Result<(), Box<dyn std::error::Error>>,
{
    if event.atom == *wanted_prop {
        let result = callback(conn, event.window);
        errors::log_if_failed(&result);
    }
}

fn get_cardinal<C: Connection>(
    conn: &C,
    prop: &Atom,
    window: Window,
) -> Result<u32, errors::Error> {
    let prop_req = xproto::get_property(conn, false, window, *prop, AtomEnum::CARDINAL, 0, 1024)?;

    let prop = prop_req.reply()?;

    if prop.format != 32 {
        eprintln!("Window: {:?}, property: {:?}", window, prop);
        Err(errors::Error::Other(&"Invalid property format"))
    } else if let Some(mut it) = prop.value32() {
        if let Some(pid) = it.next() {
            Ok(pid)
        } else {
            Err(errors::Error::Other(&"Empty response"))
        }
    } else {
        Err(errors::Error::Other(&"Cannot iterate u32 values"))
    }
}
