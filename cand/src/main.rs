mod can;
mod config;
mod devices;
mod driver;
mod net;

use std::{fs::File, sync::Arc};

use lcp_proto::{ChannelDescriptor, ChannelFlags, DeviceDescriptor, RoomDescriptor};
use tokio::sync::mpsc;

use crate::{devices::Room, net::server::LoadedDriver};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let device_config: devices::Toplevel =
        serde_yaml::from_reader(&mut File::open("devices.yaml")?)?;

    log::info!(
        "Configured {} devices with {} channels",
        device_config.devices.len(),
        device_config
            .devices
            .iter()
            .map(|device| device.channels.len())
            .sum::<usize>()
    );

    let driver_manager = driver::init()?;

    log::info!("Loaded {} drivers", driver_manager.len());

    let mut drivers = vec![];

    let devices = device_config
        .devices
        .into_iter()
        .map(|device| DeviceDescriptor {
            id: device.id.clone().into_bytes(),
            display_name: device.display_name,
            wiki_url: device.wiki_url,
            channels: device
                .channels
                .into_iter()
                .filter_map(|channel| {
                    let (tx, commands) = mpsc::channel(16);
                    let channel_id = channel.id.clone();
                    let channel_room = channel.id.clone();
                    match driver_manager.init_driver(channel, commands) {
                        Ok(channel_descriptor) => {
                            drivers.push(LoadedDriver {
                                channel: Vec::from(channel_id.as_bytes()),
                                room: Vec::from(channel_room.as_bytes()),
                                device: Vec::from(device.id.as_bytes()),
                                driver: tx,
                            });
                            Some(channel_descriptor)
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to initialize channel driver for {}/{}/{}: {}",
                                channel_room,
                                device.id,
                                channel_id,
                                e
                            );
                            None
                        }
                    }
                })
                .collect(),
        })
        .collect();

    let core = Arc::new(net::server::Core {
        rooms: device_config
            .rooms
            .into_iter()
            .map(|room| RoomDescriptor {
                id: room.id.into_bytes(),
                display_name: room.display_name,
            })
            .collect(),
        drivers,
        devices,
    });

    net::server::listen("[::]:2342".parse()?, core).await?;

    Ok(())
}
