#[warn(rust_2018_idioms)]

use std::env;

extern crate tokio;

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::future;
use tokio::codec::Framed;

use lab_can_tcp_proto::{Rs232CanCmd, CanTCPCodec};

async fn tcp_listener(addr: &mut SocketAddr) {
    let listener = TcpListener::bind(addr).await.unwrap();

    listener.incoming().for_each(|sock| {
        println!("incoming connection");
        let sock = sock.unwrap();
        // println!("accept error = {:?}", err); on error?

        tokio::spawn({
            println!("connection accepted");

            let framed_sock = Framed::new(sock, CanTCPCodec {});
            framed_sock.for_each(move |frame| {
                let frame = frame.unwrap();
                // println!("accept error = {:?}", err); on error?

                println!("Activity on client");

                println!("Processing message from network...");
                dbg!(&frame);

                // customscripts(msg);

                match frame.cmd {
                    Rs232CanCmd::SetFilter | Rs232CanCmd::SetMode => {
                        //* XXX *//
                    },
                    Rs232CanCmd::Pkt => {
                        // transmit to serial
                        // send to all connected network clients
                    },
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
                    },
                    Rs232CanCmd::WriteCtrlReg => {
                        if frame.data_len() == 1 {
                            // send to serial
                        }
                    },
                    Rs232CanCmd::Error |
                    Rs232CanCmd::NotifyReset |
                    Rs232CanCmd::Resync |
                    Rs232CanCmd::NotifyTXOvf => {
                        //don't react on these commands
                    },
                    _ => {}, // Reset isn't handled in cand-c ??? only default case
                }

                println!("...processing done.");
                future::ready(())
            });
            future::ready(())
        })
    }).await;
    future::ready(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _args: Vec<String> = env::args().collect();

    let _port: u32 = 2342;

    let addr: SocketAddr = "127.0.0.1:2342".parse().unwrap();
    let listener = TcpListener::bind(&addr).await?;

    listener.incoming().for_each(|sock| {
        println!("incoming connection");
        let sock = sock.unwrap();
        // println!("accept error = {:?}", err); on error?

        tokio::spawn({
            println!("connection accepted");

            let framed_sock = Framed::new(sock, CanTCPCodec {});
            framed_sock.for_each(move |frame| {
                let frame = frame.unwrap();
                // println!("accept error = {:?}", err); on error?

                println!("Activity on client");

                println!("Processing message from network...");
                dbg!(&frame);

                // customscripts(msg);

                match frame.cmd {
                    Rs232CanCmd::SetFilter | Rs232CanCmd::SetMode => {
                        //* XXX *//
                    },
                    Rs232CanCmd::Pkt => {
                        // transmit to serial
                        // send to all connected network clients
                    },
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
                    },
                    Rs232CanCmd::WriteCtrlReg => {
                        if frame.data_len() == 1 {
                            // send to serial
                        }
                    },
                    Rs232CanCmd::Error |
                    Rs232CanCmd::NotifyReset |
                    Rs232CanCmd::Resync |
                    Rs232CanCmd::NotifyTXOvf => {
                        //don't react on these commands
                    },
                    _ => {}, // Reset isn't handled in cand-c ??? only default case
                }

                println!("...processing done.");
                future::ready(()) // tokio 0.2
            })
        });

        future::ready(())
    }).await;
    /*.map_err(|err| {
        println!("accept error = {:?}", err);
    });*/

    println!("starting event loop...");

    Ok(())
}
