use bytes::Bytes;

use super::{Apply, CommandError};
use crate::{
    db::Db,
    frame::Frame,
    parse::{ParseError, Parser},
};

pub struct Echo {
    message: Bytes,
}

impl Apply for Echo {
    fn apply(self, _db: &mut Db) -> Result<Frame, CommandError> {
        Ok(Frame::Bulk(self.message))
    }
}

impl Echo {
    pub fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let message = parser.next_bytes()?;

        Ok(Self { message })
    }
}
