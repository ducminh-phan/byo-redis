use crate::{
    command::{Apply, CommandError},
    db::Db,
    frame::Frame,
    parse::{ParseError, Parser},
};

#[derive(Debug, Default)]
pub struct DbSize {}

impl Apply for DbSize {
    fn apply(self, db: &mut Db) -> Result<Frame, CommandError> {
        Ok(Frame::Integer(db.data.len() as i64))
    }
}

impl DbSize {
    pub fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        Ok(Self::default())
    }
}
