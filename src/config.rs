use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use derive_more::Display;

#[derive(Debug, PartialEq, Serialize, Deserialize, Display, Clone)]
#[display(fmt = "url: {}", url)]
pub(crate) struct HomeAssistant {
    pub(crate) url: String,
    pub(crate) token: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Display, Clone)]
#[display(fmt = "key: {}, allowed_times: [{:?}]", key, allowed_times)]
pub(crate) struct Device {
    pub(crate) key: String,
    pub(crate) name: String,
    pub(crate) device_id: String,
    pub(crate) manufacture: u16,
    pub(crate) allowed_times: HashMap<String, Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Display, Clone)]
#[display(fmt = "home_assistant: {}, allowed_skew: {}, devices: [{:?}]", home_assistant, allowed_skew, devices)]
pub(crate) struct Config {
    pub(crate) database_path: String,
    pub(crate) home_assistant: HomeAssistant,
    pub(crate) allowed_skew: u32,
    pub(crate) devices: HashMap<String, Device>,
}
