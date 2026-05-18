use bytes::Bytes;

use super::{Apply, CommandError};
use crate::{
    db::{Db, Value},
    frame::Frame,
    parse::{ParseError, Parser},
};

pub struct Get {
    key: Bytes,
}

impl Apply for Get {
    fn apply(self, db: &mut Db) -> Result<Frame, CommandError> {
        match db.data.get(&self.key) {
            None => Ok(Frame::Null),
            Some(Value::String(d)) => Ok(Frame::Bulk(d.clone())),
            Some(_) => Err(CommandError::InvalidArgument),
        }
    }
}

impl Get {
    pub fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // The `GET` string has already been consumed. The next value is the
        // name of the key to get. If the next value is not a string or the
        // input is fully consumed, then an error is returned.
        let key = parser.next_bytes()?;

        Ok(Self { key })
    }
}
