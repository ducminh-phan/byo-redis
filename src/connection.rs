use std::io::Cursor;

use bytes::{Buf, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::frame::{Frame, FrameError};

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

#[derive(Debug)]
pub enum ConnectionError {
    Parse(FrameError),
    Io(std::io::Error),
}

impl From<std::io::Error> for ConnectionError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<FrameError> for ConnectionError {
    fn from(value: FrameError) -> Self {
        Self::Parse(value)
    }
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionError::Parse(e) => e.fmt(f),
            ConnectionError::Io(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for ConnectionError {}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4096),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>, ConnectionError> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }
            if self.stream.read_buf(&mut self.buffer).await? == 0 {
                return Ok(None); // clean close
            }
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>, FrameError> {
        let mut cursor = Cursor::new(&self.buffer[..]);
        match Frame::check(&mut cursor) {
            Ok(()) => {
                let consumed = cursor.position() as usize;
                cursor.set_position(0);
                let frame = Frame::parse(&mut cursor)?;
                self.buffer.advance(consumed);
                Ok(Some(frame))
            }
            Err(FrameError::Incomplete) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> std::io::Result<()> {
        // Arrays are encoded by encoding each entry. All other frame types are
        // considered literals. For now, mini-redis is not able to encode
        // recursive frame structures. See below for more details.
        match frame {
            Frame::Array(val) => {
                // Encode the frame type prefix. For an array, it is `*`.
                self.stream.write_u8(b'*').await?;

                // Encode the length of the array.
                self.write_integer(val.len() as i64).await?;

                // Iterate and encode each entry in the array.
                for entry in val {
                    self.write_value(entry).await?;
                }
            }
            // The frame type is a literal. Encode the value directly.
            _ => self.write_value(frame).await?,
        }

        // Ensure the encoded frame is written to the socket. The calls above
        // are to the buffered stream and writes. Calling `flush` writes the
        // remaining contents of the buffer to the socket.
        self.stream.flush().await
    }

    /// Write a frame literal to the stream
    async fn write_value(&mut self, frame: &Frame) -> std::io::Result<()> {
        match frame {
            Frame::Simple(val) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(val) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                self.write_integer(*val).await?;
            }
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Bulk(val) => {
                let len = val.len();

                self.stream.write_u8(b'$').await?;
                self.write_integer(len as i64).await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            // Encoding an `Array` from within a value cannot be done using a
            // recursive strategy. In general, async fns do not support
            // recursion. Mini-redis has not needed to encode nested arrays yet,
            // so for now it is skipped.
            Frame::Array(_val) => unreachable!(),
        }

        Ok(())
    }

    /// Write an integer frame to the stream
    async fn write_integer(&mut self, val: i64) -> std::io::Result<()> {
        use std::io::Write;

        // Convert the value to a string
        let mut buf = [0u8; 20];
        let mut buf = Cursor::new(&mut buf[..]);
        write!(&mut buf, "{val}")?;

        let pos = buf.position() as usize;
        self.stream.write_all(&buf.get_ref()[..pos]).await?;
        self.stream.write_all(b"\r\n").await?;

        Ok(())
    }
}
