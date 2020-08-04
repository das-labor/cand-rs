use std::io;
use std::io::ErrorKind;
use bytes::{BufMut, BytesMut};

use tokio_util::codec::{Decoder, Encoder};
use std::convert::{TryFrom, TryInto};

pub struct CmdError;

#[derive(Debug)]
#[repr(u8)]
pub enum Rs232CanCmd {
    Reset = 0x00,
    SetFilter = 0x10,
    Pkt = 0x11,
    SetMode = 0x12,
    Error = 0x13,
    NotifyReset = 0x14,
    PingGateway = 0x15,
    Resync = 0x16,
    Version = 0x17,
    IDString = 0x18,
    Packetcounters = 0x19,
    Errorcounters = 0x1A,
    Powerdraw = 0x1B,
    ReadCtrlReg = 0x1C,
    WriteCtrlReg = 0x1D,
    GetResetCause = 0x1E,
    NotifyTXOvf = 0x1F
}

impl TryFrom<u8> for Rs232CanCmd {
    type Error = CmdError;

    fn try_from(value: u8) -> Result<Self, <Self as TryFrom<u8>>::Error> {
        let cmd = match value {
            0x00 => Self::Reset,
            0x10 => Self::SetFilter,
            0x11 => Self::Pkt,
            0x12 => Self::SetMode,
            0x13 => Self::Error,
            0x14 => Self::NotifyReset,
            0x15 => Self::PingGateway,
            0x16 => Self::Resync,
            0x17 => Self::Version,
            0x18 => Self::IDString,
            0x19 => Self::Packetcounters,
            0x1A => Self::Errorcounters,
            0x1B => Self::Powerdraw,
            0x1C => Self::ReadCtrlReg,
            0x1D => Self::WriteCtrlReg,
            0x1E => Self::GetResetCause,
            0x1F => Self::NotifyTXOvf,
            _ => {
                return Err(CmdError)
            }
        };

        Ok(cmd)
    }
}

const HEADER_LENGTH: usize = 2;
const MAX_PAYLOAD_LENGTH: usize = 18;

pub struct CanTCPCodec;

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

            let cmd = if let Ok(cmd) = packet_data[1].try_into() {
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

impl Encoder<CanTCPPacket> for CanTCPCodec {
    type Error = io::Error;

    /// CanTCP packet format:
    /// +--------+---------+--------------+
    /// | u8 len | u8 type | [u8] payload |
    /// +--------+---------+--------------+
    fn encode(&mut self, item: CanTCPPacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(HEADER_LENGTH + item.data.len());

        dst.put_u8(item.data.len() as u8);
        dst.put_u8(item.cmd as u8);
        dst.put_slice(&item.data);

        Ok(())
    }
}


#[derive(Debug)]
pub struct CanTCPPacket {
    pub cmd: Rs232CanCmd,
    pub data: Vec<u8>
}

impl CanTCPPacket {
    pub fn data_len(self) -> usize {
        self.data.len()
    }
}

