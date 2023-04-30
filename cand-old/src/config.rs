use crate::hook;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Deserialize)]
pub struct Config {
    pub backend: Backend,
    #[serde(default)]
    pub listen: Vec<Listen>,
    #[serde(rename = "hook")]
    #[serde(default)]
    pub hooks: Vec<hook::Hook>,
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
pub enum Backend {
    #[serde(rename = "socketcan")]
    SocketCAN { interface: String },
    #[serde(rename = "net")]
    Network { connect: SocketAddr },
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
pub enum Listen {
    #[serde(rename = "tcp")]
    TCP { bind: SocketAddr },
}
