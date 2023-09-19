use std::io::{Read, Write};

use crate::helper::{ReadExt, WriteExt};

pub trait Serialize {
    fn serialize<W: Write>(&self, write: &mut W) -> crate::Result<()>;
}

pub trait Deserialize: Sized {
    fn deserialize<R: Read>(read: &mut R) -> crate::Result<Self>;
}

pub trait SerializeId: Serialize {
    fn id(&self) -> u8;
}

pub trait DeserializeId: Sized {
    fn deserialize<R: Read>(id: u8, read: &mut R) -> crate::Result<Self>;
}

impl Serialize for ciborium::value::Value {
    fn serialize<W: Write>(&self, write: &mut W) -> crate::Result<()> {
        let mut window = write.write_window();

        ciborium::ser::into_writer(&self, &mut window)?;
        window.finish()?;
        Ok(())
    }
}

impl Deserialize for ciborium::value::Value {
    fn deserialize<R: Read>(read: &mut R) -> crate::Result<Self> {
        let mut window = read.read_window()?;
        let value = ciborium::de::from_reader(&mut window)?;
        window.skip_to_end()?;
        Ok(value)
    }
}
