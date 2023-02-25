use std::collections::HashMap;

use log::{debug, warn};
use serde::Serialize;

use crate::{error, config};

#[derive(Debug, PartialEq, Serialize)]
struct Entity {
    entity_id: String,
    state: String,
    attributes: HashMap<String, String>,
}

pub(crate) async fn trigger_off(device: String, friendly_name: String, config: config::HomeAssistant) -> error::Result<()> {
    debug!("Triggering off for {}", device);

    let client = reqwest::Client::new();

    // We only trigger off for known entities
    let url = format!("{}states/{}", config.url, urlencoding::encode(format!("binary_sensor.{}", device.replace(":", "_")).as_str()));
    let res = client.get(url)
        .bearer_auth(config.token)
        .send()
        .await?;

    // We get out of here since we don't need to update unknown entities
    if res.status().as_u16() == 404 {
        return Ok(())
    } 

    let mut attributes = HashMap::new();
    attributes.insert("friendly_name".to_string(), friendly_name);

    let entity = Entity {
        entity_id: format!("binary_sensor.{}", device),
        state: "off".to_string(),
        attributes,
    };

    let url = format!("{}states/{}", config.url, urlencoding::encode(format!("binary_sensor.{}", device.replace(":", "_")).as_str()));
    debug!("Calling URL: {}", url);

    let _res = client.post(url)
        .bearer_auth(config.token)
        .json(&entity)
        .send()
        .await
        .or_else(|e| {
            Err(error::new(format!("could not call home assistant: {:?}", e)))
        });

    if let Err(err) = _res {
        warn!("Could not trigger offline state for {}", device);
        warn!("{}", err);
    }

    Ok(())
}

pub(crate) async fn trigger_on(device: String, friendly_name: String, config: config::HomeAssistant) -> error::Result<()> {
    debug!("Triggering on for {}", device);

    let mut attributes = HashMap::new();
    attributes.insert("friendly_name".to_string(), friendly_name);

    let entity = Entity {
        entity_id: format!("binary_sensor.{}", device),
        state: "on".to_string(),
        attributes,
    };

    let url = format!("{}states/{}", config.url, urlencoding::encode(format!("binary_sensor.{}", device.replace(":", "_")).as_str()));
    debug!("Calling URL: {}", url);

    let client = reqwest::Client::new();
    let _res = client.post(url)
        .bearer_auth(config.token)
        .json(&entity)
        .send()
        .await
        .or_else(|e| {
            Err(error::new(format!("could not call home assistant: {:?}", e)))
        });
       
    if let Err(err) = _res {
        warn!("Could not trigger online state for {}", device);
        warn!("{}", err);
    }

    Ok(())
}