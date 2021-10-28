use std::net::SocketAddr;
use serde::Deserialize;
use crate::hook;

#[derive(Deserialize)]
pub struct Config {
    pub backend: Backend,
    pub listen: Vec<Listen>,
    #[serde(rename = "hook")]
    pub hooks: Vec<hook::Hook>
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
pub enum Backend {
    #[serde(rename = "socketcan")]
    SocketCAN {
        interface: String
    },
    #[serde(rename = "net")]
    Network {
        connect: SocketAddr
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
pub enum Listen {
    #[serde(rename = "tcp")]
    TCP {
        bind: SocketAddr
    }
}