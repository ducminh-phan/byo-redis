use bytes::Bytes;

use super::{Apply, CommandError};
use crate::{
    db::Db,
    frame::Frame,
    parse::{ParseError, Parser},
};

pub struct Set {
    key: Bytes,
    value: Bytes,
}

impl Apply for Set {
    fn apply(self, db: &mut Db) -> Result<Frame, CommandError> {
        todo!()
    }
}

impl Set {
    pub fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Read the key to set. This is a required field
        let key = parser.next_bytes()?;

        // Read the value to set. This is a required field.
        let value = parser.next_bytes()?;

        Ok(Self { key, value })
    }
}
