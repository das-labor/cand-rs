use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type ChannelType = lcp_proto::ValueType;
pub type ChannelKind = lcp_proto::ChannelKind;

#[derive(Serialize, Deserialize, Debug)]
pub struct Toplevel {
    pub rooms: Vec<Room>,
    pub devices: Vec<Device>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Room {
    pub id: String,
    pub display_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Device {
    pub id: String,
    pub display_name: String,
    pub wiki_url: String,
    pub channels: Vec<Channel>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Channel {
    pub id: String,
    pub display_name: String,
    pub room: String,
    pub kind: ChannelKind,
    pub driver: String,
    pub driver_options: HashMap<String, ciborium::value::Value>,
}
