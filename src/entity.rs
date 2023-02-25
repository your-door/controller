use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Entity {
    #[serde(default)]
    pub entity_id: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub attributes: EntityAttributes,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct EntityAttributes {
    #[serde(default)]
    pub friendly_name: String,
    #[serde(default)]
    pub last_seen_local: u64,
    #[serde(default)]
    pub last_seen: u32,
    #[serde(default)]
    pub restart_count: u16,
}
