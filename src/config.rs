use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use derive_more::Display;

#[derive(Debug, PartialEq, Serialize, Deserialize, Display, Clone)]
#[display(fmt = "key: {}, times: [{:?}]", key, times)]
pub(crate) struct Device {
    pub(crate) key: String,
    pub(crate) device_id: String,
    pub(crate) manufacture: u16,
    pub(crate) times: HashMap<String, Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Display, Clone)]
#[display(fmt = "ha_url: {}, devices: [{:?}]", ha_url, devices)]
pub(crate) struct Config {
    pub(crate) ha_url: String,
    pub(crate) allowed_skew: u32,
    pub(crate) devices: HashMap<String, Device>,
}
