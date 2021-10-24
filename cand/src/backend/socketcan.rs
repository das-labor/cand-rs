use failure::Fail;
use futures::{Sink, Stream, channel::mpsc, StreamExt, SinkExt};
use labctl::can::{self, CanPacket};
use labctl::cand;
use labctl::cand::Message;
use semver::Version;
use tokio::task;
use tokio_socketcan::{CANFrame, CANSocket};
use crate::util;

pub fn connect(interface: &str) -> anyhow::Result<(impl Stream<Item=cand::Message>, impl Sink<cand::Message>, task::JoinHandle<()>)> {
    let read = CANSocket::open(interface)?;
    let write = CANSocket::open(interface)?;

    let (sender, stream) = mpsc::unbounded();
    let (sink, receiver) = mpsc::unbounded();

    let rx = task::spawn(read_can_frames(read, sender));
    let tx = task::spawn(write_can_frames(write, receiver, sink.clone()));

    let task = task::spawn(async {
        match futures::future::try_join(tx, rx).await {
            Ok(_) => {},
            Err(e) => {
                log::error!("Listen connection to can interface failed: {}", e);
                log::debug!("Details: {:?}", e);
            }
        };
    });

    Ok((stream, sink, task))
}

async fn read_can_frames(mut read: CANSocket, mut sender: mpsc::UnboundedSender<cand::Message>) -> anyhow::Result<()> {
    while let Some(frame) = read.next().await {
        let frame = frame?;

        let (src, dest) = can::can_id_to_tuple(frame.id())
            .map_err(Fail::compat)?;

        sender.send(cand::Message::Frame(CanPacket::new(
            src,
            dest,
            Vec::from(frame.data())
        ))).await?;
    }
    Ok(())
}

async fn write_can_frames(
    mut write: CANSocket,
    mut receiver: mpsc::UnboundedReceiver<cand::Message>,
    mut sender: mpsc::UnboundedSender<cand::Message>
) -> anyhow::Result<()> {
    while let Some(msg) = receiver.next().await {
        match msg {
            Message::Frame(frame) => {
                write.write_frame(CANFrame::new(
                    can::can_id_from_tuple(frame.src, frame.dest),
                    &frame.payload,
                    false,
                    false
                )?)?;
            }
            Message::Reset { .. } => {
                // GW -> Cand
            }
            Message::Ping => {
                sender.send(Message::Ping).await?;
            }
            Message::Resync => {
                // We don't need this crap, so this is a NO-OP
            }
            Message::VersionRequest => {
                let version: Version = env!("CARGO_PKG_VERSION").parse().unwrap();
                sender.send(Message::VersionReply {
                    major: version.major as u8,
                    minor: version.minor as u8
                }).await?;
            }
            Message::VersionReply { .. } => {
                // GW -> Cand
            }
            Message::FirmwareIdRequest => {
                sender.send(Message::FirmwareIdResponse(env!("CARGO_PKG_NAME").to_owned())).await?;
            }
            Message::FirmwareIdResponse(_) => {
                // GW -> Cand
            }
            Message::Unknown { .. } => {}
        }
    }
    Ok(())
}