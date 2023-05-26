mod lamp;

use std::collections::HashMap;

use lcp_proto::ChannelDescriptor;
use tokio::sync::{mpsc, oneshot};

use crate::devices::Channel;

pub trait Driver {
    fn create_instance(
        &self,
        ch: Channel,
        commands: mpsc::Receiver<DriverMessage>,
    ) -> anyhow::Result<ChannelDescriptor>;
}

pub enum DriverMessage {
    Subscribe(mpsc::Sender<ciborium::value::Value>),
    GetValue(oneshot::Sender<ciborium::value::Value>),
    SetValue(ciborium::value::Value, oneshot::Sender<()>),
}

pub fn init() -> anyhow::Result<DriverManager> {
    let mut manager = DriverManager {
        drivers: HashMap::new(),
    };

    manager.register_driver("lamp".to_owned(), lamp::Lamp);

    Ok(manager)
}

pub struct DriverManager {
    drivers: HashMap<String, Box<dyn Driver>>,
}

impl DriverManager {
    pub fn register_driver<D: Driver + 'static>(&mut self, driver_id: String, driver: D) {
        self.drivers.insert(driver_id, Box::new(driver));
    }

    pub fn init_driver(
        &self,
        ch: Channel,
        commands: mpsc::Receiver<DriverMessage>,
    ) -> anyhow::Result<ChannelDescriptor> {
        if let Some(driver) = self.drivers.get(&ch.driver) {
            driver.create_instance(ch, commands)
        } else {
            Err(anyhow::Error::msg(format!(
                "Could not find driver {}",
                ch.driver
            )))
        }
    }

    pub fn len(&self) -> usize {
        self.drivers.len()
    }
}
