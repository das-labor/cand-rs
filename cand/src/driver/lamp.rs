use lcp_proto::{ChannelDescriptor, ChannelFlags};
use tokio::sync::mpsc;

use super::Driver;

pub struct Lamp;

impl Driver for Lamp {
    fn create_instance(
        &self,
        ch: crate::devices::Channel,
        commands: tokio::sync::mpsc::Receiver<super::DriverMessage>,
    ) -> anyhow::Result<lcp_proto::ChannelDescriptor> {
        tokio::task::spawn(background_task(commands));
        Ok(ChannelDescriptor {
            flags: ChannelFlags(0x7),
            room: ch.room.into_bytes(),
            display_name: ch.display_name,
            value_type: lcp_proto::ValueType::U8,
            channel_kind: ch.kind,
        })
    }
}

pub async fn background_task(commands: mpsc::Receiver<super::DriverMessage>) {
    log::warn!("TODO: Generalize");
    match background_task_inner(commands).await {
        Ok(()) => log::warn!("Lamp Driver exiting"),
        Err(_) => todo!(),
    }
}

pub async fn background_task_inner(
    mut commands: mpsc::Receiver<super::DriverMessage>,
) -> anyhow::Result<()> {
    while let Some(message) = commands.recv().await {
        match message {
            super::DriverMessage::Subscribe(reply) => todo!(),
            super::DriverMessage::GetValue(_) => todo!(),
            super::DriverMessage::SetValue(_, _) => todo!(),
        }
    }
    Ok(())
}
