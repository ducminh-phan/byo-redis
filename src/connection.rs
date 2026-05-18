use std::io::Cursor;

use bytes::{Buf, BytesMut};
use tokio::{
    io::{AsyncReadExt, BufWriter},
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

    pub async fn write_frame(&mut self, _frame: &Frame) -> std::io::Result<()> {
        todo!("serialize frame to RESP bytes")
    }
}
