use bytes::Bytes;

use crate::{
    command::{Apply, CommandError},
    db::Db,
    frame::Frame,
    parse::{ParseError, Parser},
};

#[derive(Debug, Default)]
pub struct Ping {
    msg: Option<Bytes>,
}

impl Apply for Ping {
    fn apply(self, _db: &mut Db) -> Result<Frame, CommandError> {
        match self.msg {
            Some(msg) => Ok(Frame::Bulk(msg)),
            None => Ok(Frame::Simple("PONG".into())),
        }
    }
}

impl Ping {
    fn new(message: Option<Bytes>) -> Self {
        Self { msg: message }
    }

    pub fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.next_bytes() {
            Ok(msg) => Ok(Ping::new(Some(msg))),
            Err(ParseError::EndOfStream) => Ok(Ping::default()),
            Err(e) => Err(e.into()),
        }
    }
}
