extern crate dbus;

use dbus::blocking::Connection;
use std::time::Duration;

use super::errors;

pub fn request_gamemode_for_pid(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::new_session()?;

    let proxy = conn.with_proxy(
        "com.feralinteractive.GameMode",
        "/com/feralinteractive/GameMode",
        Duration::from_millis(5000),
    );

    let args: (i32,) = (pid as i32,);

    let (result,): (i32,) =
        proxy.method_call("com.feralinteractive.GameMode", "RegisterGame", args)?;

    if result != 0 {
        Err(Box::from(errors::Error::Other(
            "Failed to register pid for gamemode",
        )))
    } else {
        Ok(())
    }
}
