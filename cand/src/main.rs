#[warn(rust_2018_idioms)]

use std::env;

extern crate tokio;

use tokio::net::TcpListener;
use tokio::prelude::*;

use tokio::codec::Framed;
use std::process::exit;

use lab_can_tcp_proto::{Rs232CanCmd, CanTCPCodec};

fn main() {
    //let args: Vec<String> = env::args().collect();

    //let port = 2342;

    let addr = "127.0.0.1:2342".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });

    let server = listener.incoming().for_each(|sock| {
        println!("connection accepted");

        let framed_sock = Framed::new(sock, CanTCPCodec {});
        framed_sock.for_each(move |frame| {
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
                Rs232CanCmd::Reset => {}, // Reset isn't handled in cand-c ??? only default case
            }

            println!("...processing done.");
            Ok(())
        })
    })
    .map_err(|err| {
        println!("accept error = {:?}", err);
    });

    println!("starting event loop...");
    tokio::run(server);
}
