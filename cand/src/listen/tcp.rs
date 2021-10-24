use std::net::SocketAddr;
use failure::ResultExt;
use futures::{Sink, SinkExt, Stream, StreamExt};
use futures::channel::mpsc;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use labctl::cand;
use labctl::cand::Message;
use tokio::net::{TcpListener, TcpStream};
use tokio::{io, task};
use tokio::io::{ReadHalf, WriteHalf};
use crate::reactor::ReactorHandle;

pub async fn listen(addr: SocketAddr, handle: ReactorHandle) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (conn, addr) = listener.accept().await?;
        log::debug!("New connection from {}", addr);
    }
}

async fn handle_client(client: TcpStream) -> (impl Stream<Item=cand::Message>, impl Sink<cand::Message>, task::JoinHandle<()>) {
    let (read, write) = io::split(client);

    let (sender, stream) = mpsc::unbounded();
    let (sink, receiver) = mpsc::unbounded();

    let task = task::spawn(async {
        let res = futures::future::try_join(
            read_from_client(read, sender),
            write_to_client(write, receiver)
        ).await;
        match res {
            Ok(_) => {},
            Err(e) => {
                log::error!("Client Error: {}", e);
                log::debug!("Details: {:?}", e);
            }
        }
    });

    (stream, sink, task)
}

async fn read_from_client(mut read: ReadHalf<TcpStream>, mut sender: UnboundedSender<cand::Message>) -> anyhow::Result<()> {
    while let Some(msg) = cand::read_packet_async(&mut read).await.compat()? {
        sender.send(msg).await?;
    }
    Ok(())
}

async fn write_to_client(mut write: WriteHalf<TcpStream>, mut receiver: UnboundedReceiver<cand::Message>) -> anyhow::Result<()> {
    while let Some(msg) = receiver.next().await {
        cand::write_packet_to_cand_async(&mut write, &msg).await.compat()?;
    }
    Ok(())
}