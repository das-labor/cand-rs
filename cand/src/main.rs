#![warn(rust_2018_idioms)]

use std::env;

use std::net::SocketAddr;

#[allow(unused_imports)]
use tokio::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Decoder;

use tokio::sync::broadcast;
use futures::stream::StreamExt;

use lab_can_tcp_proto::{Rs232CanCmd, CanTCPCodec, CanTCPPacket, Rs232CanPacket};
use std::convert::TryInto;
use futures::SinkExt;

async fn client_connection(
    conn: TcpStream,
    mut rx: tokio::sync::broadcast::Receiver<Rs232CanPacket>,
    tx: tokio::sync::broadcast::Sender<Rs232CanPacket>) {
    println!("connection accepted");

    let mut framed_sock = CanTCPCodec.framed(conn);
    loop {
        tokio::select! {
            tcp_packet = framed_sock.next() => {
                if let Some(frame) = tcp_packet {
                    let frame = frame.unwrap();
                    // println!("accept error = {:?}", err); on error?

                    println!("Processing message from network client...");
                    dbg!(&frame);

                    // customscripts(msg);

                    if frame.cmd == Rs232CanCmd::Pkt {
                        // todo transmit to serial
                        tx.send(frame.try_into().unwrap()).unwrap();
                    }

                    println!("...processing done.");
                }
            }
            rs232_packet = rx.recv() => { //
                if let Ok(packet) = rs232_packet {
                    if packet.cmd == Rs232CanCmd::Pkt {
                        framed_sock.send(CanTCPPacket {
                                cmd: Rs232CanCmd::Pkt,
                                data: packet.data.clone()
                            }).await.unwrap();
                    }
                }
            }
        };
    }
}

#[tokio::main]
async fn main() {
    let _args: Vec<String> = env::args().collect();

    let _port: u32 = 2342;

    let addr: SocketAddr = "127.0.0.1:2342".parse().unwrap();

    let mut listener = TcpListener::bind(&addr).await.unwrap();

    let (tx, mut _rx) = broadcast::channel::<Rs232CanPacket>(16);

    loop {
        let (sock, _) = listener.accept().await.expect("accept error");
        println!("incoming connection");

        let client_tx = tx.clone();

        let client_rx = client_tx.subscribe();

        tokio::spawn(async move {
            client_connection(sock, client_rx, client_tx).await
        });
    }
}
