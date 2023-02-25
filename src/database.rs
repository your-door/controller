use crate::{error, config, entity::EntityAttributes};
use log::debug;
use crate::entity::Entity;

pub(crate) async fn get_entity_state(name: String, device: String, config: config::HomeAssistant) -> error::Result<EntityAttributes> {
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
        let mut entity_state = Entity::default();
        entity_state.attributes.friendly_name = name;
        return Ok(entity_state.attributes)
    } 

    let entity_state = res
        .json::<Entity>()
        .await
        .or_else(|e| {
            Err(error::new(format!("could not deserialize entity state: {:?}", e)))
        })
        .unwrap();

    debug!("{:?}", entity_state.attributes);
    Ok(entity_state.attributes)
}

pub(crate) async fn update_entity_state(entity_attributes: EntityAttributes, device: String, config: config::HomeAssistant) -> error::Result<()> {
    let client = reqwest::Client::new();

    // We only trigger off for known entities
    let url = format!("{}states/{}", config.url, urlencoding::encode(format!("binary_sensor.{}", device.replace(":", "_")).as_str()));
    let res = client.get(url.clone())
        .bearer_auth(config.token.clone())
        .send()
        .await
        .or_else(|e| {
            Err(error::new(format!("could not call home assistant: {:?}", e)))
        })?;

    // We get out of here since we don't need to update unknown entities
    let mut entity_state = Entity::default();
    if res.status().as_u16() == 404 {
        entity_state.entity_id = format!("binary_sensor.{}", device.replace(":", "_"));
        entity_state.state = "off".to_owned();
    } else {
        entity_state = res
            .json::<Entity>()
            .await
            .or_else(|e| {
                Err(error::new(format!("could not deserialize entity state: {:?}", e)))
            })
            .unwrap();
    }

    entity_state.attributes = entity_attributes;

    client.post(url.clone())
        .bearer_auth(config.token.clone())
        .json(&entity_state)
        .send()
        .await
        .or_else(|e| {
            Err(error::new(format!("could not call home assistant: {:?}", e)))
        })
        .unwrap();
            
    return Ok(())
}