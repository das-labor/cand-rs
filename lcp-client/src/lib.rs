mod comm;
pub mod error;

use std::collections::HashMap;

pub use ciborium::Value;
use comm::{ToBackend, ToFrontend};
pub use error::*;

use lcp_proto::{DeviceDescriptor, Message, RoomDescriptor, ToClientPayload, ToServerPayload};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpStream, ToSocketAddrs},
    sync::mpsc,
};

pub struct Connection {
    tx: mpsc::Sender<comm::ToBackend>,
}

impl Connection {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> crate::Result<(Self, BackgroundTask)> {
        let mut connection = TcpStream::connect(addr).await?;

        Message {
            request_id: 1,
            payload: ToServerPayload::Hello,
        }
        .serialize_async(&mut connection)
        .await?;

        let res = tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            Message::deserialize_async(&mut connection),
        )
        .await
        .map_err(|_| crate::Error::ResponeTimeout)??;

        if res.request_id != 1 {
            return Err(crate::Error::InvalidResponseId);
        }

        match res.payload {
            ToClientPayload::Welcome => {}
            _ => return Err(crate::Error::UnexpectedResponseType),
        }

        let (tx, rx) = mpsc::channel(16);

        Ok((
            Connection { tx: tx.clone() },
            BackgroundTask { tx, rx, connection },
        ))
    }

    pub async fn list_devices(
        &self,
    ) -> crate::Result<(Vec<RoomDescriptor>, Vec<DeviceDescriptor>)> {
        let mut rx = self.request(ToServerPayload::GetDevices, false).await;

        loop {
            match rx.recv().await {
                Some(ToFrontend::RequestSent { .. }) => {}
                Some(ToFrontend::Response(payload)) => {
                    break match payload {
                        ToClientPayload::Devices { rooms, devices } => Ok((rooms, devices)),
                        ToClientPayload::Err { code, message } => {
                            Err(Error::ServerError { code, message })
                        }
                        _ => Err(crate::Error::UnexpectedResponseType),
                    }
                }
                None => panic!("Backend task crashed"),
            }
        }
    }

    pub async fn set_channel(
        &self,
        device: &[u8],
        room: &[u8],
        channel: &[u8],
        value: Value,
    ) -> crate::Result<()> {
        let mut rx = self
            .request(
                ToServerPayload::SetChannel {
                    device: device.to_owned(),
                    room: room.to_owned(),
                    channel: channel.to_owned(),
                    value,
                },
                false,
            )
            .await;

        loop {
            match rx.recv().await.expect("Backend task crashed") {
                ToFrontend::RequestSent { .. } => {}
                ToFrontend::Response(payload) => {
                    break match payload {
                        ToClientPayload::Ok => Ok(()),
                        ToClientPayload::Err { code, message } => {
                            Err(Error::ServerError { code, message })
                        }
                        _ => Err(crate::Error::UnexpectedResponseType),
                    }
                }
            }
        }
    }

    async fn request(
        &self,
        payload: ToServerPayload,
        multi_response: bool,
    ) -> mpsc::Receiver<ToFrontend> {
        let (tx, rx) = mpsc::channel(16);

        self.tx
            .send(ToBackend::SendRequest {
                reply: tx,
                payload,
                multi_response,
            })
            .await
            .unwrap();

        rx
    }
}

pub struct BackgroundTask {
    tx: mpsc::Sender<ToBackend>,
    rx: mpsc::Receiver<ToBackend>,
    connection: TcpStream,
}

impl BackgroundTask {
    pub async fn run(self) -> crate::Result<()> {
        let BackgroundTask { tx, rx, connection } = self;

        let (mut read, mut write) = tokio::io::split(connection);

        let read_tx = tx.clone();
        let (write_tx, write_rx) = mpsc::channel(16);

        futures::future::try_join3(
            write_task(&mut write, write_rx),
            read_task(&mut read, read_tx),
            state_task(rx, write_tx),
        )
        .await?;

        Ok(())
    }
}

async fn write_task<W: AsyncWrite + Unpin>(
    write: &mut W,
    mut rx: mpsc::Receiver<Message<ToServerPayload>>,
) -> crate::Result<()> {
    while let Some(msg) = rx.recv().await {
        log::debug!("Sending Message: {msg:?}");
        msg.serialize_async(write).await?;
        log::trace!("Message sent");
    }
    Ok(())
}

async fn read_task<R: AsyncRead + Unpin>(
    read: &mut R,
    tx: mpsc::Sender<ToBackend>,
) -> crate::Result<()> {
    loop {
        let message = Message::deserialize_async(read).await?;

        log::debug!("Received Message: {message:?}");

        if tx
            .send(ToBackend::MessageReceived { message })
            .await
            .is_err()
        {
            break Ok(());
        };
    }
}

async fn state_task(
    mut rx: mpsc::Receiver<ToBackend>,
    write_tx: mpsc::Sender<Message<ToServerPayload>>,
) -> crate::Result<()> {
    let mut subscriptions = HashMap::new();
    let mut next_req_id = 1;
    while let Some(message) = rx.recv().await {
        match message {
            ToBackend::SendRequest {
                payload,
                multi_response,
                reply,
            } => {
                let req_id = next_req_id;
                next_req_id += 1;

                let message = Message {
                    request_id: req_id,
                    payload,
                };

                write_tx.send(message).await.unwrap();
                reply
                    .send(ToFrontend::RequestSent { req_id })
                    .await
                    .unwrap();

                subscriptions.insert(
                    req_id,
                    Subscription {
                        multi: multi_response,
                        sender: reply,
                    },
                );
                log::debug!("Added subscription for request ID {req_id}, multi_response = {multi_response:?}")
            }
            ToBackend::Unregister { req_id } => {
                subscriptions.remove(&req_id);
                log::debug!("Removed subscription for request ID {req_id} if it existed")
            }
            ToBackend::MessageReceived { message } => {
                let remove = if let Some(sub) = subscriptions.get(&message.request_id) {
                    log::trace!("Informing Frontend of message {message:?}");
                    sub.sender
                        .send(ToFrontend::Response(message.payload))
                        .await
                        .unwrap();
                    !sub.multi
                } else {
                    log::trace!("Message ignored, because no receiver existed: {message:?}");
                    false
                };

                if remove {
                    log::debug!(
                        "Automatically removed subscription for request ID {}",
                        message.request_id
                    );
                    subscriptions.remove(&message.request_id);
                }
            }
        }
    }
    Ok(())
}

struct Subscription {
    sender: mpsc::Sender<ToFrontend>,
    multi: bool,
}
