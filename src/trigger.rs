use log::{debug, warn};

use crate::{error, config};
use crate::entity::Entity;

pub(crate) async fn trigger_off(device: String, friendly_name: String, config: config::HomeAssistant) -> error::Result<()> {
    debug!("Triggering off for {}", device);

    let client = reqwest::Client::new();

    // We only trigger off for known entities
    let url = format!("{}states/{}", config.url, urlencoding::encode(format!("binary_sensor.{}", device.replace(":", "_")).as_str()));
    let res = client.get(url)
        .bearer_auth(config.token.clone())
        .send()
        .await
        .or_else(|e| {
            Err(error::new(format!("could not call home assistant: {:?}", e)))
        })?;

    // We get out of here since we don't need to update unknown entities
    if res.status().as_u16() == 404 {
        return Ok(())
    } 

    let mut entity_state = res
        .json::<Entity>()
        .await
        .or_else(|e| {
            Err(error::new(format!("could not deserialize entity state: {:?}", e)))
        })
        .unwrap();

    entity_state.state = "off".to_string();
    entity_state.attributes.friendly_name = friendly_name;

    let url = format!("{}states/{}", config.url, urlencoding::encode(format!("binary_sensor.{}", device.replace(":", "_")).as_str()));
    debug!("Calling URL: {}", url);

    let _res = client.post(url)
        .bearer_auth(config.token.clone())
        .json(&entity_state)
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

    let client = reqwest::Client::new();

    // We only trigger off for known entities
    let url = format!("{}states/{}", config.url, urlencoding::encode(format!("binary_sensor.{}", device.replace(":", "_")).as_str()));
    let res = client.get(url)
        .bearer_auth(config.token.clone())
        .send()
        .await
        .or_else(|e| {
            Err(error::new(format!("could not call home assistant: {:?}", e)))
        })?;
    
    // We get out of here since we don't need to update unknown entities
    let mut entity_state = Entity::default();
    if res.status().as_u16() != 404 {
        entity_state = res
            .json::<Entity>()
            .await
            .or_else(|e| {
                Err(error::new(format!("could not deserialize entity state: {:?}", e)))
            })
            .unwrap();
    }

    entity_state.entity_id = format!("binary_sensor.{}", device.replace(":", "_"));
    entity_state.state = "on".to_string();
    entity_state.attributes.friendly_name = friendly_name;

    let url = format!("{}states/{}", config.url, urlencoding::encode(&entity_state.entity_id));
    debug!("Calling URL: {}", url);

    let _res = client.post(url)
        .bearer_auth(config.token)
        .json(&entity_state)
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