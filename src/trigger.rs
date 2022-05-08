use log::debug;

use crate::error;


pub(crate) async fn trigger_off(device: String) -> error::Result<()> {
    debug!("Triggering off for {}", device);

    Ok(())
}

pub(crate) async fn trigger_on(device: String) -> error::Result<()> {
    debug!("Triggering on for {}", device);

    Ok(())
}