use std::{fmt, io::Cursor, num::TryFromIntError, string::FromUtf8Error};

use atoi::atoi;
use bytes::{Buf, Bytes};

#[derive(Debug)]
pub enum Frame {
    Simple(Bytes),
    Error(Bytes),
    Integer(i64),
    Bulk(Bytes),
    Array(Vec<Frame>),
    Null,
}

#[derive(Debug)]
pub enum FrameError {
    Incomplete,      // need more bytes
    Invalid(String), // malformed RESP
}

impl Frame {
    /// Checks if an entire message can be decoded from `cursor`
    pub fn check(cursor: &mut Cursor<&[u8]>) -> Result<(), FrameError> {
        match get_u8(cursor)? {
            b'+' => {
                get_line(cursor)?;
                Ok(())
            }
            b'-' => {
                get_line(cursor)?;
                Ok(())
            }
            b':' => {
                let _ = get_integer(cursor)?;
                Ok(())
            }
            b'$' => {
                // Bulk strings in the RESP protocol.
                //
                // Format: $<length>\r\n<data>\r\n
                //
                // Special case: $-1\r\n represents a Null value.
                // Validates that the frame conforms to RESP protocol.
                if b'-' == peek_u8(cursor)? {
                    let line = get_line(cursor)?;
                    if line != b"-1" {
                        return Err("protocol error; invalid frame format".into());
                    }
                    Ok(())
                } else {
                    // Read the bulk string
                    let len: usize = get_integer(cursor)?.try_into()?;

                    // skip that number of bytes + 2 (\r\n).
                    skip(cursor, len + 2)
                }
            }
            b'*' => {
                let len = get_integer(cursor)?;

                for _ in 0..len {
                    Frame::check(cursor)?;
                }

                Ok(())
            }
            actual => Err(format!("protocol error; invalid frame type byte `{actual}`").into()),
        }
    }

    /// The message has already been validated with `check`.
    pub fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Frame, FrameError> {
        match get_u8(cursor)? {
            b'+' => Ok(Frame::Simple(Bytes::copy_from_slice(get_line(cursor)?))),
            b'-' => Ok(Frame::Error(Bytes::copy_from_slice(get_line(cursor)?))),
            b':' => Ok(Frame::Integer(get_integer(cursor)?)),
            b'$' => {
                if b'-' == peek_u8(cursor)? {
                    let line = get_line(cursor)?;

                    if line != b"-1" {
                        return Err("protocol error; invalid frame format".into());
                    }

                    Ok(Frame::Null)
                } else {
                    // Read the bulk string
                    let len = get_integer(cursor)?.try_into()?;
                    let n = len + 2;

                    if cursor.remaining() < n {
                        return Err(FrameError::Incomplete);
                    }

                    let data = Bytes::copy_from_slice(&cursor.chunk()[..len]);

                    // skip that number of bytes + 2 (\r\n).
                    skip(cursor, n)?;

                    Ok(Frame::Bulk(data))
                }
            }
            b'*' => {
                let len = get_integer(cursor)?.try_into()?;
                let mut out = Vec::with_capacity(len);

                for _ in 0..len {
                    out.push(Frame::parse(cursor)?);
                }

                Ok(Frame::Array(out))
            }
            _ => unimplemented!(),
        }
    }

    /// Converts the frame to an "unexpected frame" error
    pub fn to_error(&self) -> crate::Error {
        format!("unexpected frame: {self}").into()
    }
}

impl PartialEq<&str> for Frame {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Frame::Simple(s) => s.eq(other),
            Frame::Bulk(s) => s.eq(other),
            _ => false,
        }
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use std::str;

        match self {
            Frame::Simple(msg) => match str::from_utf8(msg) {
                Ok(string) => string.fmt(fmt),
                Err(_) => write!(fmt, "{msg:?}"),
            },
            Frame::Error(msg) => write!(fmt, "error: {msg:?}"),
            Frame::Integer(num) => num.fmt(fmt),
            Frame::Bulk(msg) => match str::from_utf8(msg) {
                Ok(string) => string.fmt(fmt),
                Err(_) => write!(fmt, "{msg:?}"),
            },
            Frame::Null => "(nil)".fmt(fmt),
            Frame::Array(parts) => {
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        // use space as the array element display separator
                        write!(fmt, " ")?;
                    }

                    part.fmt(fmt)?;
                }

                Ok(())
            }
        }
    }
}

fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    if !src.has_remaining() {
        return Err(FrameError::Incomplete);
    }

    Ok(src.chunk()[0])
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    if !src.has_remaining() {
        return Err(FrameError::Incomplete);
    }

    Ok(src.get_u8())
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), FrameError> {
    if src.remaining() < n {
        return Err(FrameError::Incomplete);
    }

    src.advance(n);
    Ok(())
}

/// Read a new-line terminated integer
fn get_integer(cursor: &mut Cursor<&[u8]>) -> Result<i64, FrameError> {
    let line = get_line(cursor)?;
    atoi::<i64>(&line).ok_or("invalid number".into())
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], FrameError> {
    // Scan the bytes directly
    let start = src.position() as usize;
    // Scan to the second-to-last byte
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            // We found a line, update the position to be *after* the \n
            src.set_position((i + 2) as u64);

            // Return the line
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(FrameError::Incomplete)
}

impl From<String> for FrameError {
    fn from(src: String) -> FrameError {
        FrameError::Invalid(src)
    }
}

impl From<&str> for FrameError {
    fn from(src: &str) -> FrameError {
        src.to_string().into()
    }
}

impl From<FromUtf8Error> for FrameError {
    fn from(_src: FromUtf8Error) -> FrameError {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for FrameError {
    fn from(_src: TryFromIntError) -> FrameError {
        "protocol error; invalid frame format".into()
    }
}

impl std::error::Error for FrameError {}

impl fmt::Display for FrameError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FrameError::Incomplete => "stream ended early".fmt(fmt),
            FrameError::Invalid(err) => err.fmt(fmt),
        }
    }
}
