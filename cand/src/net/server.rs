use lcp_proto::{DeviceDescriptor, Message, RoomDescriptor, ToClientPayload, ToServerPayload};
use std::{fmt, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
};

use crate::driver::{Driver, DriverMessage};

pub struct Core {
    pub rooms: Vec<RoomDescriptor>,
    pub devices: Vec<DeviceDescriptor>,
    pub drivers: Vec<LoadedDriver>,
}

impl Core {
    pub fn find_device(&self, name: &[u8]) -> Option<&DeviceDescriptor> {
        self.devices.iter().find(|dev| &dev.id == name)
    }

    pub fn find_driver(
        &self,
        device: &[u8],
        room: &[u8],
        channel: &[u8],
    ) -> Option<mpsc::Sender<DriverMessage>> {
        self.drivers
            .iter()
            .find(|ld| ld.device == device && ld.room == room && ld.channel == channel)
            .map(|ld| ld.driver.clone())
    }
}

pub struct LoadedDriver {
    pub device: Vec<u8>,
    pub room: Vec<u8>,
    pub channel: Vec<u8>,
    pub driver: mpsc::Sender<DriverMessage>,
}

pub async fn listen(addr: SocketAddr, core: Arc<Core>) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    log::info!("Listening on {addr}...");

    loop {
        let (stream, addr) = listener.accept().await?;
        let core = core.clone();

        tokio::task::spawn(async move {
            match run_client(stream, addr, core).await {
                Ok(()) => {}
                Err(e) => {
                    log::error!("Client Error: {e}");
                    log::debug!("Details: {e:?}");
                }
            };
        });
    }
}

async fn run_client(stream: TcpStream, addr: SocketAddr, core: Arc<Core>) -> anyhow::Result<()> {
    log::debug!("Connection from {addr}");
    let (mut read, mut write) = tokio::io::split(stream);
    let (tx, rx) = mpsc::channel(16);

    futures::future::try_join(write_task(&mut write, rx), read_task(&mut read, tx, core)).await?;
    Ok(())
}

async fn write_task<W: AsyncWrite + Unpin>(
    write: &mut W,
    mut rx: mpsc::Receiver<Message<ToClientPayload>>,
) -> anyhow::Result<()> {
    while let Some(msg) = rx.recv().await {
        log::trace!("Sent message: {msg:#?}");
        msg.serialize_async(write).await?;
    }
    Ok(())
}

async fn read_task<R: AsyncRead + Unpin>(
    read: &mut R,
    tx: mpsc::Sender<Message<ToClientPayload>>,
    core: Arc<Core>,
) -> anyhow::Result<()> {
    loop {
        let message: Message<ToServerPayload> = Message::deserialize_async(read).await?;
        log::trace!("Received message: {message:#?}");

        match &message.payload {
            ToServerPayload::Hello => {
                tx.send(message.new_response(ToClientPayload::Welcome))
                    .await?;
            }
            ToServerPayload::GetDevices => {
                tx.send(message.new_response(ToClientPayload::Devices {
                    rooms: core.rooms.clone(),
                    devices: core.devices.clone(),
                }))
                .await?;
            }
            ToServerPayload::SetChannel {
                device,
                room,
                channel,
                value,
            } => {
                if let Some(device) = core.find_driver(&device, &room, &channel) {
                    let (sender, receiver) = oneshot::channel();
                    device
                        .send(DriverMessage::SetValue(value.clone(), sender))
                        .await
                        .unwrap();

                    send_delayed_response(
                        receiver,
                        tx.clone(),
                        message.new_response(ToClientPayload::Ok),
                    )
                } else {
                    tx.send(message.new_response(ToClientPayload::Err {
                        code: lcp_proto::ErrorCode::NoSuchChannel,
                        message: "Could not find Device, Room or Channel".to_owned(),
                    }))
                    .await
                    .unwrap()
                }
            }
            ToServerPayload::GetChannel {
                device,
                room,
                channel,
            } => {
                if let Some(device) = core.find_driver(&device, &room, &channel) {
                    let (sender, receiver) = oneshot::channel();
                    device.send(DriverMessage::GetValue(sender)).await.unwrap();
                    let message = message.clone();

                    let tx = tx.clone();

                    tokio::spawn(async move {
                        let value = receiver.await.unwrap();

                        let response =
                            message.new_response(ToClientPayload::ChannelValue { flags: 0, value });

                        tx.send(response).await.unwrap();
                    });
                } else {
                    tx.send(message.new_response(ToClientPayload::Err {
                        code: lcp_proto::ErrorCode::NoSuchChannel,
                        message: "Could not find Device, Room or Channel".to_owned(),
                    }))
                    .await
                    .unwrap()
                }
            }
            ToServerPayload::SubscribeChannel {
                device,
                room,
                channel,
            } => {
                if let Some(device) = core.find_driver(&device, &room, &channel) {
                    let (sender, mut receiver) = mpsc::channel(16);
                    device.send(DriverMessage::Subscribe(sender)).await.unwrap();
                    let message = message.clone();

                    let tx = tx.clone();

                    tokio::spawn(async move {
                        while let Some(value) = receiver.recv().await {
                            let response = message
                                .new_response(ToClientPayload::ChannelValue { flags: 0, value });

                            tx.send(response).await.unwrap();
                        }
                    });
                } else {
                    tx.send(message.new_response(ToClientPayload::Err {
                        code: lcp_proto::ErrorCode::NoSuchChannel,
                        message: "Could not find Device, Room or Channel".to_owned(),
                    }))
                    .await
                    .unwrap()
                }
            }
        }
    }
}

/// Send a response as soon as the passed future resolved
fn send_delayed_response<T: Send + Sync + fmt::Debug + 'static>(
    future: oneshot::Receiver<()>,
    sender: mpsc::Sender<T>,
    payload: T,
) {
    tokio::spawn(async move {
        // We can ignore errors here, they likely mean that the client has disconnected
        let _ = future.await;
        sender.send(payload).await.unwrap();
    });
}
