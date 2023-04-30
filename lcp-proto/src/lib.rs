mod error;
mod helper;
mod net;

use std::io::Cursor;

use crate::helper::{ReadExt, WriteExt};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ciborium::value::Value;
pub use error::*;
use net::{Deserialize, DeserializeId, Serialize, SerializeId};

#[cfg(feature = "async")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub type ID = Vec<u8>;

#[derive(Debug)]
pub struct Message<T> {
    pub request_id: u64,
    pub payload: T,
}

#[cfg(feature = "async")]
impl<T: SerializeId + DeserializeId> Message<T> {
    pub async fn serialize_async<W: tokio::io::AsyncWrite + Unpin>(
        &self,
        write: &mut W,
    ) -> crate::Result<()> {
        write.write_u8(self.payload.id()).await?;
        write.write_u64(self.request_id).await?;

        let mut buf = Cursor::new(Vec::new());
        self.payload.serialize(&mut buf)?;
        let buf = buf.into_inner();

        let mut len_buf = Cursor::new(Vec::new());
        len_buf.write_varlen(buf.len())?;
        let len_buf = len_buf.into_inner();

        write.write_all(&len_buf).await?;
        write.write_all(&buf).await?;
        Ok(())
    }

    pub async fn deserialize_async<R: tokio::io::AsyncRead + Unpin>(
        read: &mut R,
    ) -> crate::Result<Self> {
        let opcode = read.read_u8().await?;
        let request_id = read.read_u64().await?;

        let buflen = read.read_u8().await?;
        if buflen & 0x80 != 0 {
            todo!("I am lazy");
        }
        let buflen = buflen as usize;

        let mut buf = vec![0; buflen];
        read.read_exact(&mut buf).await?;

        let payload = T::deserialize(opcode, &mut Cursor::new(buf))?;

        Ok(Message {
            request_id,
            payload,
        })
    }
}

impl<T: SerializeId> Serialize for Message<T> {
    fn serialize<W: std::io::Write>(&self, write: &mut W) -> crate::Result<()> {
        let opcode = self.payload.id();
        write.write_u8(opcode)?;
        write.write_u64::<BigEndian>(self.request_id)?;

        let mut window = write.write_window();
        self.payload.serialize(&mut window)?;
        window.finish()?;

        Ok(())
    }
}

impl<T: DeserializeId> Deserialize for Message<T> {
    fn deserialize<R: std::io::Read>(read: &mut R) -> crate::Result<Self> {
        let opcode = read.read_u8()?;
        let request_id = read.read_u64::<BigEndian>()?;
        Ok({
            let mut window = read.read_window()?;
            let payload = T::deserialize(opcode, &mut window)?;
            window.skip_to_end()?;
            Message {
                request_id,
                payload,
            }
        })
    }
}

impl Message<ToServerPayload> {
    pub fn new_response(&self, payload: ToClientPayload) -> Message<ToClientPayload> {
        Message {
            request_id: self.request_id,
            payload,
        }
    }
}

#[derive(Debug)]
pub enum ToServerPayload {
    Hello,
    GetDevices,
    SetChannel {
        device: ID,
        room: ID,
        channel: ID,
        value: Value,
    },
    GetChannel {
        device: ID,
        room: ID,
        channel: ID,
    },
    SubscribeChannel {
        device: ID,
        room: ID,
        channel: ID,
    },
}

impl Serialize for ToServerPayload {
    fn serialize<W: std::io::Write>(&self, write: &mut W) -> crate::Result<()> {
        match &self {
            ToServerPayload::Hello => {}
            ToServerPayload::GetDevices => {}
            ToServerPayload::SetChannel {
                device,
                room,
                channel,
                value,
            } => {
                write.write_id(&device)?;
                write.write_id(&room)?;
                write.write_id(&channel)?;
                value.serialize(write)?;
            }
            ToServerPayload::GetChannel {
                device,
                room,
                channel,
            } => {
                write.write_id(&device)?;
                write.write_id(&room)?;
                write.write_id(&channel)?;
            }
            ToServerPayload::SubscribeChannel {
                device,
                room,
                channel,
            } => {
                write.write_id(&device)?;
                write.write_id(&room)?;
                write.write_id(&channel)?;
            }
        }
        Ok(())
    }
}

impl SerializeId for ToServerPayload {
    fn id(&self) -> u8 {
        match &self {
            ToServerPayload::Hello => 0,
            ToServerPayload::GetDevices => 1,
            ToServerPayload::SetChannel { .. } => 2,
            ToServerPayload::GetChannel { .. } => 3,
            ToServerPayload::SubscribeChannel { .. } => 4,
        }
    }
}

impl DeserializeId for ToServerPayload {
    fn deserialize<R: std::io::Read>(id: u8, read: &mut R) -> crate::Result<Self> {
        Ok(match id {
            0 => ToServerPayload::Hello,
            1 => ToServerPayload::GetDevices,
            2 => ToServerPayload::SetChannel {
                device: read.read_id()?,
                room: read.read_id()?,
                channel: read.read_id()?,
                value: Value::deserialize(read)?,
            },
            3 => ToServerPayload::GetChannel {
                device: read.read_id()?,
                room: read.read_id()?,
                channel: read.read_id()?,
            },
            4 => ToServerPayload::SubscribeChannel {
                device: read.read_id()?,
                room: read.read_id()?,
                channel: read.read_id()?,
            },
            _ => return Err(crate::Error::InvalidId),
        })
    }
}

#[derive(Debug)]
pub enum ToClientPayload {
    Welcome,
    Devices {
        rooms: Vec<RoomDescriptor>,
        devices: Vec<DeviceDescriptor>,
    },
    ChannelValue {
        flags: u8,
        value: Value,
    },
    Ok,
    Err {
        code: ErrorCode,
        message: String,
    },
}

impl Serialize for ToClientPayload {
    fn serialize<W: std::io::Write>(&self, write: &mut W) -> crate::Result<()> {
        Ok(match &self {
            ToClientPayload::Welcome => {}
            ToClientPayload::Devices { rooms, devices } => {
                write.write_varlen(rooms.len())?;
                for room in rooms {
                    room.serialize(write)?;
                }
                write.write_varlen(devices.len())?;
                for device in devices {
                    device.serialize(write)?;
                }
            }
            ToClientPayload::ChannelValue { flags, value } => {
                write.write_u8(*flags)?;
                value.serialize(write)?;
            }
            ToClientPayload::Ok => {}
            ToClientPayload::Err { code, message } => {
                code.serialize(write)?;
                write.write_string(&message)?;
            }
        })
    }
}

impl SerializeId for ToClientPayload {
    fn id(&self) -> u8 {
        match &self {
            ToClientPayload::Welcome => 0,
            ToClientPayload::Devices { .. } => 1,
            ToClientPayload::ChannelValue { .. } => 2,
            ToClientPayload::Ok => 3,
            ToClientPayload::Err { .. } => 4,
        }
    }
}

impl DeserializeId for ToClientPayload {
    fn deserialize<R: std::io::Read>(id: u8, read: &mut R) -> crate::Result<Self> {
        Ok(match id {
            0 => ToClientPayload::Welcome,
            1 => {
                let room_count = read.read_varlen()?;
                let mut rooms = Vec::with_capacity(room_count);
                for _ in 0..room_count {
                    rooms.push(RoomDescriptor::deserialize(read)?);
                }
                let device_count = read.read_varlen()?;
                let mut devices = Vec::with_capacity(device_count);
                for _ in 0..device_count {
                    devices.push(DeviceDescriptor::deserialize(read)?);
                }
                ToClientPayload::Devices { rooms, devices }
            }
            2 => ToClientPayload::ChannelValue {
                flags: read.read_u8()?,
                value: Value::deserialize(read)?,
            },
            3 => ToClientPayload::Ok,
            4 => ToClientPayload::Err {
                code: ErrorCode::deserialize(read)?,
                message: read.read_string()?,
            },
            _ => return Err(crate::Error::InvalidId),
        })
    }
}

#[derive(Debug, Clone)]
pub struct RoomDescriptor {
    pub id: ID,
    pub display_name: String,
}

impl Serialize for RoomDescriptor {
    fn serialize<W: std::io::Write>(&self, write: &mut W) -> crate::Result<()> {
        let mut window = write.write_window();
        window.write_id(&self.id)?;
        window.write_string(&self.display_name)?;
        window.finish()?;
        Ok(())
    }
}

impl Deserialize for RoomDescriptor {
    fn deserialize<R: std::io::Read>(read: &mut R) -> crate::Result<Self> {
        let mut window = read.read_window()?;
        let id = window.read_id()?;
        let display_name = window.read_string()?;
        window.skip_to_end()?;
        Ok(RoomDescriptor { id, display_name })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    NoSuchDevice,
    NoSuchRoom,
    NoSuchChannel,
    InvalidRequestForChannel,
    UnknownError(u8),
}

impl Serialize for ErrorCode {
    fn serialize<W: std::io::Write>(&self, write: &mut W) -> crate::Result<()> {
        let id = match self {
            ErrorCode::NoSuchDevice => 0,
            ErrorCode::NoSuchRoom => 1,
            ErrorCode::NoSuchChannel => 2,
            ErrorCode::InvalidRequestForChannel => 3,
            ErrorCode::UnknownError(id) => *id,
        };

        write.write_u8(id)?;
        Ok(())
    }
}

impl Deserialize for ErrorCode {
    fn deserialize<R: std::io::Read>(read: &mut R) -> crate::Result<Self> {
        let id = read.read_u8()?;
        Ok(match id {
            0 => ErrorCode::NoSuchDevice,
            1 => ErrorCode::NoSuchRoom,
            2 => ErrorCode::NoSuchChannel,
            3 => ErrorCode::InvalidRequestForChannel,
            id => ErrorCode::UnknownError(id),
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct EnumValue {
    pub id: ID,
    pub display_name: String,
}

impl Serialize for EnumValue {
    fn serialize<W: std::io::Write>(&self, write: &mut W) -> crate::Result<()> {
        write.write_id(&self.id)?;
        write.write_string(&self.display_name)?;
        Ok(())
    }
}

impl Deserialize for EnumValue {
    fn deserialize<R: std::io::Read>(read: &mut R) -> crate::Result<Self> {
        let id = read.read_id()?;
        let display_name = read.read_string()?;
        Ok(EnumValue { id, display_name })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum ValueType {
    #[serde(rename = "bool")]
    Boolean,
    #[serde(rename = "u8")]
    U8,
    #[serde(rename = "u32")]
    U32,
    #[serde(rename = "f32")]
    F32,
    #[serde(rename = "rgb")]
    RGB,
    #[serde(rename = "event")]
    Event,
    #[serde(rename = "enum")]
    Enum(Vec<EnumValue>),
    #[serde(rename = "string")]
    String,
    #[serde(rename = "binary")]
    Binary,
    #[serde(rename = "object")]
    Object,
}

impl Serialize for ValueType {
    fn serialize<W: std::io::Write>(&self, write: &mut W) -> crate::Result<()> {
        let id = match self {
            ValueType::Boolean => 0,
            ValueType::U8 => 1,
            ValueType::U32 => 2,
            ValueType::F32 => 3,
            ValueType::RGB => 4,
            ValueType::Event => 5,
            ValueType::Enum(_) => 6,
            ValueType::String => 7,
            ValueType::Binary => 8,
            ValueType::Object => 9,
        };

        write.write_u8(id)?;

        match self {
            ValueType::Enum(values) => {
                write.write_varlen(values.len())?;
                for value in values {
                    write.write_id(&value.id)?;
                    write.write_string(&value.display_name)?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl Deserialize for ValueType {
    fn deserialize<R: std::io::Read>(read: &mut R) -> crate::Result<Self> {
        let id = read.read_varlen()?;
        Ok(match id {
            0 => ValueType::Boolean,
            1 => ValueType::U8,
            2 => ValueType::U32,
            3 => ValueType::F32,
            4 => ValueType::RGB,
            5 => ValueType::Event,
            6 => {
                let count = read.read_varlen()?;
                let mut res = Vec::with_capacity(count);
                for _ in 0..count {
                    let id = read.read_id()?;
                    let display_name = read.read_string()?;
                    res.push(EnumValue { id, display_name });
                }
                ValueType::Enum(res)
            }
            7 => ValueType::String,
            8 => ValueType::Binary,
            9 => ValueType::Object,
            _ => return Err(crate::Error::InvalidId),
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum ChannelKind {
    #[serde(rename = "other")]
    Other,
    #[serde(rename = "actor:lamp")]
    ActorLamp,
    #[serde(rename = "actor:wall_socket")]
    ActorWallSocket,
    #[serde(rename = "actor:relay")]
    ActorRelay,
    #[serde(rename = "sensor:temperature")]
    SensorTemperature,
    #[serde(rename = "sensor:button")]
    SensorButton,
    #[serde(rename = "generic:volume")]
    Volume,
    #[serde(rename = "device:borg")]
    DeviceBorg,
    #[serde(rename = "unknown")]
    Unknown(u8),
}

impl Serialize for ChannelKind {
    fn serialize<W: std::io::Write>(&self, write: &mut W) -> crate::Result<()> {
        let id = match self {
            ChannelKind::Other => 0,
            ChannelKind::ActorLamp => 1,
            ChannelKind::ActorWallSocket => 2,
            ChannelKind::ActorRelay => 3,
            ChannelKind::SensorTemperature => 4,
            ChannelKind::SensorButton => 5,
            ChannelKind::Volume => 6,
            ChannelKind::DeviceBorg => 7,
            ChannelKind::Unknown(v) => *v,
        };
        write.write_u8(id)?;
        Ok(())
    }
}

impl Deserialize for ChannelKind {
    fn deserialize<R: std::io::Read>(read: &mut R) -> crate::Result<Self> {
        Ok(match read.read_u8()? {
            0 => ChannelKind::Other,
            1 => ChannelKind::ActorLamp,
            2 => ChannelKind::ActorWallSocket,
            3 => ChannelKind::ActorRelay,
            4 => ChannelKind::SensorTemperature,
            5 => ChannelKind::SensorButton,
            6 => ChannelKind::Volume,
            7 => ChannelKind::DeviceBorg,
            v => ChannelKind::Unknown(v),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChannelFlags(pub u8);
impl Serialize for ChannelFlags {
    fn serialize<W: std::io::Write>(&self, write: &mut W) -> crate::Result<()> {
        write.write_u8(self.0)?;
        Ok(())
    }
}

impl Deserialize for ChannelFlags {
    fn deserialize<R: std::io::Read>(read: &mut R) -> crate::Result<Self> {
        Ok(ChannelFlags(read.read_u8()?))
    }
}

#[derive(Debug, Clone)]
pub struct ChannelDescriptor {
    pub flags: ChannelFlags,
    pub room: ID,
    pub display_name: String,
    pub value_type: ValueType,
    pub channel_kind: ChannelKind,
}

impl Serialize for ChannelDescriptor {
    fn serialize<W: std::io::Write>(&self, write: &mut W) -> crate::Result<()> {
        let mut window = write.write_window();
        self.flags.serialize(&mut window)?;
        window.write_id(&self.room)?;
        window.write_string(&self.display_name)?;
        self.value_type.serialize(&mut window)?;
        self.channel_kind.serialize(&mut window)?;
        window.finish()?;
        Ok(())
    }
}

impl Deserialize for ChannelDescriptor {
    fn deserialize<R: std::io::Read>(read: &mut R) -> crate::Result<Self> {
        let mut window = read.read_window()?;
        let flags = ChannelFlags::deserialize(&mut window)?;
        let room = window.read_id()?;
        let display_name = window.read_string()?;
        let value_type = ValueType::deserialize(&mut window)?;
        let channel_kind = ChannelKind::deserialize(&mut window)?;
        window.skip_to_end()?;
        Ok(ChannelDescriptor {
            flags,
            room,
            display_name,
            value_type,
            channel_kind,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeviceDescriptor {
    pub id: ID,
    pub display_name: String,
    pub wiki_url: String,
    pub channels: Vec<ChannelDescriptor>,
}

impl Serialize for DeviceDescriptor {
    fn serialize<W: std::io::Write>(&self, write: &mut W) -> crate::Result<()> {
        let mut window = write.write_window();
        window.write_id(&self.id)?;
        window.write_string(&self.display_name)?;
        window.write_string(&self.wiki_url)?;
        window.write_varlen(self.channels.len())?;
        for channel in &self.channels {
            channel.serialize(&mut window)?;
        }
        window.finish()?;
        Ok(())
    }
}

impl Deserialize for DeviceDescriptor {
    fn deserialize<R: std::io::Read>(read: &mut R) -> crate::Result<Self> {
        let mut window = read.read_window()?;
        let id = window.read_id()?;
        let display_name = window.read_string()?;
        let wiki_url = window.read_string()?;
        let channel_count = window.read_varlen()?;
        let mut channels = Vec::with_capacity(channel_count);
        for _ in 0..channel_count {
            channels.push(ChannelDescriptor::deserialize(&mut window)?);
        }
        Ok(DeviceDescriptor {
            id,
            display_name,
            wiki_url,
            channels,
        })
    }
}
