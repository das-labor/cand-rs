#[warn(rust_2018_idioms)]
use std::env;

use std::net::SocketAddr;

#[allow(warnings)]
use tokio::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Decoder;

use futures::stream::StreamExt;


use lab_can_tcp_proto::{Rs232CanCmd, CanTCPCodec};

async fn client_connection(conn: TcpStream) {
    println!("connection accepted");

    let mut framed_sock = CanTCPCodec.framed(conn);
    while let Some(frame) = framed_sock.next().await {
        let frame = frame.unwrap();
        // println!("accept error = {:?}", err); on error?

        println!("Activity on client");

        println!("Processing message from network...");
        dbg!(&frame);

        // customscripts(msg);

        match frame.cmd {
            Rs232CanCmd::SetFilter | Rs232CanCmd::SetMode => {
                //* XXX *//
            }
            Rs232CanCmd::Pkt => {
                // transmit to serial
                // send to all connected network clients
            }
            Rs232CanCmd::PingGateway |
            Rs232CanCmd::Version |
            Rs232CanCmd::IDString |
            Rs232CanCmd::Packetcounters |
            Rs232CanCmd::Errorcounters |
            Rs232CanCmd::Powerdraw |
            Rs232CanCmd::ReadCtrlReg |
            Rs232CanCmd::GetResetCause => {
                // msg len = 0
                // send to serial
            }
            Rs232CanCmd::WriteCtrlReg => {
                if frame.data_len() == 1 {
                    // send to serial
                }
            }
            Rs232CanCmd::Error |
            Rs232CanCmd::NotifyReset |
            Rs232CanCmd::Resync |
            Rs232CanCmd::NotifyTXOvf => {
                //don't react on these commands
            }
            _ => {} // Reset isn't handled in cand-c ??? only default case
        }

        println!("...processing done.");
    }
}

#[tokio::main]
async fn main() {
    let _args: Vec<String> = env::args().collect();

    let _port: u32 = 2342;

    let addr: SocketAddr = "127.0.0.1:2342".parse().unwrap();

    let mut listener = TcpListener::bind(&addr).await.unwrap();

    loop {
        let (sock, _) = listener.accept().await.expect("accept error");
        println!("incoming connection");

        tokio::spawn(async move {
            client_connection(sock).await
        });
    }
}
