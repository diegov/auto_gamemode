extern crate ctrlc;
extern crate x11rb;

mod errors;
mod gamemode;
mod x11;

use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use x11::WindowWatcher;

fn main() -> Result<(), Box<dyn Error>> {
    let do_run = Arc::new(AtomicBool::new(true));

    let do_run_store = do_run.clone();
    ctrlc::set_handler(move || {
        println!("Exiting...");
        do_run_store.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let (conn, screen_num) = x11rb::connect(None)?;

    conn.monitor_new_windows(screen_num, do_run, |conn, window| {
        let pid = conn.get_window_pid(window)?;
        gamemode::request_gamemode_for_pid(pid)
    })?;

    drop(conn);

    Ok(())
}
