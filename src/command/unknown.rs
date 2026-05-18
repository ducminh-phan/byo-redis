use bytes::Bytes;

use super::{Apply, CommandError};
use crate::{db::Db, frame::Frame};

/// Represents an "unknown" command. This is not a real `Redis` command.
#[derive(Debug)]
pub struct Unknown {
    command_name: Bytes,
}

impl Apply for Unknown {
    fn apply(self, db: &mut Db) -> Result<Frame, CommandError> {
        Ok(Frame::Error(
            format!(
                "ERR unknown command '{}'",
                String::from_utf8_lossy(&self.command_name)
            )
            .into_bytes()
            .into(),
        ))
    }
}

impl Unknown {
    pub fn new(command_name: Bytes) -> Self {
        Self { command_name }
    }
}
