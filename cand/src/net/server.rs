use lcp_proto::{DeviceDescriptor, Message, RoomDescriptor, ToClientPayload, ToServerPayload};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

pub struct Core {
    pub rooms: Vec<RoomDescriptor>,
    pub devices: Vec<DeviceDescriptor>,
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

        match message.payload {
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
            } => todo!(),
            ToServerPayload::GetChannel {
                device,
                room,
                channel,
            } => todo!(),
            ToServerPayload::SubscribeChannel {
                device,
                room,
                channel,
            } => todo!(),
        }
    }
}
