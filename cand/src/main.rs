mod devices;
mod net;

use std::{fs::File, sync::Arc};

use lcp_proto::{ChannelDescriptor, ChannelFlags, DeviceDescriptor, RoomDescriptor};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let devices: devices::Toplevel = serde_yaml::from_reader(&mut File::open("devices.yaml")?)?;

    println!("Devices: {:?}", devices);

    let core = Arc::new(net::server::Core {
        rooms: devices
            .rooms
            .into_iter()
            .map(|room| RoomDescriptor {
                id: room.id.as_bytes().to_owned(),
                display_name: room.display_name,
            })
            .collect(),
        devices: devices
            .devices
            .into_iter()
            .map(|device| DeviceDescriptor {
                id: device.id.as_bytes().to_owned(),
                display_name: device.display_name,
                wiki_url: device.wiki_url,
                channels: device
                    .channels
                    .into_iter()
                    .map(|channel| ChannelDescriptor {
                        flags: ChannelFlags(0x00),
                        room: channel.room.into_bytes(),
                        display_name: channel.display_name,
                        value_type: channel.ty,
                        channel_kind: channel.kind,
                    })
                    .collect(),
            })
            .collect(),
    });

    net::server::listen("[::]:2342".parse()?, core).await?;

    Ok(())
}
