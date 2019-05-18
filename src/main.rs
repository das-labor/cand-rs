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

const HEADER_LENGTH: usize = 2;
const MAX_PAYLOAD_LENGTH: usize = 18;

struct CanTCPCodec;

#[derive(Debug)]
struct CanTCPPacket {
    cmd: u8,
    data: Vec<u8>
}

impl CanTCPPacket {

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

            let packet = CanTCPPacket {
                cmd: packet_data[1],
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

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(HEADER_LENGTH + item.data.len());

        dst.put_u8(item.data.len() as u8);
        dst.put_u8(item.cmd);
        dst.put(item.data);

        Ok(())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let port = 2342;

    let addr = "127.0.0.1:2342".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let server = listener.incoming().for_each(|sock| {
        println!("connection accepted");

        let framed_sock = Framed::new(sock, CanTCPCodec {});
        framed_sock.for_each(|frame| {
            dbg!(frame);
            Ok(())
        })
    })
    .map_err(|err| {
        println!("accept error = {:?}", err);
    });

    tokio::run(server);
}
