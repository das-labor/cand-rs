use std::io::{Cursor, Read, Write};

use byteorder::ReadBytesExt;

const MAX_VARLEN: usize = 0x3fff;

fn check_varlen(len: usize) {
    if len > MAX_VARLEN {
        panic!("Item too long for protocol")
    }
}

pub trait WriteExt: Write + Sized {
    fn write_varlen(&mut self, len: usize) -> crate::Result<usize>;
    fn write_string(&mut self, string: &str) -> crate::Result<usize>;
    fn write_bytes(&mut self, bytes: &[u8]) -> crate::Result<usize>;
    fn write_id(&mut self, id: &[u8]) -> crate::Result<usize>;
    fn write_window<'a>(&'a mut self) -> WriteWindow<'a, Self>;
}

impl<T> WriteExt for T
where
    T: Write + Sized,
{
    fn write_varlen(&mut self, len: usize) -> crate::Result<usize> {
        check_varlen(len);

        if len > 0x7f {
            let first_byte = (len >> 7) as u8;
            let second_byte = (len ^ 0x80) as u8;
            self.write_all(&[first_byte, second_byte])?;
            Ok(2)
        } else {
            self.write_all(&[len as u8])?;
            Ok(1)
        }
    }

    fn write_string(&mut self, string: &str) -> crate::Result<usize> {
        let varlen_len = self.write_varlen(string.len())?;
        self.write_all(string.as_bytes())?;

        Ok(varlen_len + string.len())
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> crate::Result<usize> {
        let len_len = self.write_varlen(bytes.len())?;
        self.write_all(&bytes)?;
        Ok(len_len + bytes.len())
    }

    fn write_id(&mut self, id: &[u8]) -> crate::Result<usize> {
        self.write_bytes(id)
    }

    fn write_window<'a>(&'a mut self) -> WriteWindow<'a, Self> {
        WriteWindow {
            inner: self,
            buffer: Some(Cursor::new(Vec::new())),
        }
    }
}

pub trait ReadExt: Read + Sized {
    fn read_varlen(&mut self) -> crate::Result<usize>;
    fn read_string(&mut self) -> crate::Result<String>;
    fn read_bytes(&mut self) -> crate::Result<Vec<u8>>;
    fn read_id(&mut self) -> crate::Result<Vec<u8>>;
    fn read_window_with_length(&mut self, len: usize) -> ReadWindow<Self>;

    fn read_window(&mut self) -> crate::Result<ReadWindow<Self>> {
        let len = self.read_varlen()?;
        Ok(self.read_window_with_length(len))
    }
}

impl<T> ReadExt for T
where
    T: Read,
{
    fn read_varlen(&mut self) -> crate::Result<usize> {
        let upper_byte = self.read_u8()? as usize;
        if upper_byte & 0x80 != 0 {
            let lower_byte = self.read_u8()? as usize;
            Ok((upper_byte << 7) ^ lower_byte)
        } else {
            Ok(upper_byte)
        }
    }

    fn read_string(&mut self) -> crate::Result<String> {
        let bytes = self.read_bytes()?;
        Ok(String::from_utf8(bytes).map_err(|_| crate::Error::UTF8Error)?)
    }

    fn read_window_with_length(&mut self, len: usize) -> ReadWindow<Self> {
        ReadWindow {
            inner: self,
            remaining_bytes: len,
        }
    }

    fn read_bytes(&mut self) -> crate::Result<Vec<u8>> {
        let len = self.read_varlen()?;
        let mut buf = vec![0; len];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_id(&mut self) -> crate::Result<Vec<u8>> {
        self.read_bytes()
    }
}

pub struct ReadWindow<'a, T: Read> {
    inner: &'a mut T,
    remaining_bytes: usize,
}

impl<'a, T: Read> ReadWindow<'a, T> {
    pub fn skip_to_end(&mut self) -> std::io::Result<()> {
        if self.remaining_bytes != 0 {
            self.inner.read_exact(&mut vec![0; self.remaining_bytes])?;
            self.remaining_bytes = 0;
        }
        Ok(())
    }
}

impl<'a, T: Read> Read for ReadWindow<'a, T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.remaining_bytes == 0 {
            Ok(0)
        } else if buf.len() < self.remaining_bytes {
            let len = self.inner.read(buf)?;
            self.remaining_bytes -= len;
            Ok(len)
        } else {
            let len = self.inner.read(&mut buf[0..self.remaining_bytes])?;
            self.remaining_bytes -= len;
            Ok(len)
        }
    }
}

impl<'a, T: Read> Drop for ReadWindow<'a, T> {
    fn drop(&mut self) {
        self.skip_to_end().unwrap();
    }
}

pub struct WriteWindow<'a, T: Write> {
    inner: &'a mut T,
    buffer: Option<Cursor<Vec<u8>>>,
}

impl<'a, T: Write> WriteWindow<'a, T> {
    pub fn finish(mut self) -> crate::Result<()> {
        let buf = self.buffer.take().unwrap().into_inner();

        self.inner.write_varlen(buf.len())?;
        self.inner.write_all(&buf)?;
        Ok(())
    }
}

impl<'a, T: Write> Write for WriteWindow<'a, T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(ref mut w) = self.buffer {
            w.write(buf)
        } else {
            panic!("Invalid inner state");
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
