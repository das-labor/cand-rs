use std::env;

extern crate tokio;

use tokio::net::TcpListener;
use tokio::prelude::*;

use tokio::codec::{Decoder, Encoder};
use tokio::codec::Framed;
use bytes::BytesMut;
use bytes::BufMut;
use tokio::io;
use tokio::io::ErrorKind;
use std::process::exit;

use cand::can::encap::Rs232CanCmd;

use num;

const HEADER_LENGTH: usize = 2;
const MAX_PAYLOAD_LENGTH: usize = 18;

struct CanTCPCodec;

#[derive(Debug)]
struct CanTCPPacket {
    cmd: Rs232CanCmd,
    data: Vec<u8>
}

impl CanTCPPacket {
    fn data_len(self) -> usize {
        self.data.len()
    }
}

impl Decoder for CanTCPCodec {
    type Item = CanTCPPacket;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() == 0 {
            // why would we ever return any data for an empty input???
            return Ok(None);
        }

        let payload_length = src[0] as usize;

        if src.len() < payload_length + HEADER_LENGTH {
            // more bytes are needed
            return Ok(None);
        }

        if payload_length <= MAX_PAYLOAD_LENGTH {
            // we have enough bytes, split off a packet
            let packet_data = src.split_to(HEADER_LENGTH + payload_length);

            let slice = &packet_data[HEADER_LENGTH..HEADER_LENGTH+payload_length];

            let cmd = if let Some(cmd) = num::FromPrimitive::from_u8(packet_data[1]) {
                cmd
            } else {
                return Err(io::Error::new(ErrorKind::InvalidData, "invalid command"))
            };

            let packet = CanTCPPacket {
                cmd,
                data: slice.to_vec()
            };

            Ok(Some(packet))
        } else {
            // invalid length specified
            Err(io::Error::new(ErrorKind::InvalidData, "invalid length field"))
        }
    }
}

impl Encoder for CanTCPCodec {
    type Item = CanTCPPacket;
    type Error = io::Error;

    /// CanTCP packet format:
    /// +--------+---------+--------------+
    /// | u8 len | u8 type | [u8] payload |
    /// +--------+---------+--------------+
    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(HEADER_LENGTH + item.data.len());

        dst.put_u8(item.data.len() as u8);
        dst.put_u8(item.cmd as u8);
        dst.put(item.data);

        Ok(())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let port = 2342;

    let addr = "127.0.0.1:2342".parse().unwrap();
    let mut listener = TcpListener::bind(&addr).unwrap_or_else(|err| {
        println!("{}", err);
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
