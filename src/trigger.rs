use log::info;

use crate::error;


pub(crate) async fn trigger_off(device: String) -> error::Result<()> {
    info!("Triggering off for {}", device);

    Ok(())
}

pub(crate) async fn trigger_on(device: String) -> error::Result<()> {
    info!("Triggering on for {}", device);

    Ok(())
}