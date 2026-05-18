pub mod echo;
pub mod get;
pub mod ping;
pub mod set;
pub mod unknown;

use enum_dispatch::enum_dispatch;

use crate::{
    command::{echo::Echo, get::Get, ping::Ping, set::Set, unknown::Unknown},
    db::Db,
    frame::Frame,
    parse::Parser,
};

#[derive(Debug)]
pub enum CommandError {
    UnknownCommand,
    WrongNumberOfArguments,
    InvalidArgument,
    InternalError,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::UnknownCommand => write!(f, "unknown command"),
            CommandError::WrongNumberOfArguments => write!(f, "wrong number of arguments"),
            CommandError::InvalidArgument => write!(f, "invalid argument"),
            CommandError::InternalError => write!(f, "internal error"),
        }
    }
}

impl std::error::Error for CommandError {}

#[enum_dispatch]
pub trait Apply {
    fn apply(self, db: &mut Db) -> Result<Frame, CommandError>;
}

#[enum_dispatch(Apply)]
pub enum Command {
    Echo,
    Get,
    Ping,
    Set,
    Unknown,
}

impl Command {
    /// Parse a command from a received frame.
    ///
    /// The `Frame` must represent a Redis command supported by `mini-redis` and
    /// be the array variant.
    ///
    /// # Returns
    ///
    /// On success, the command value is returned; otherwise, `Err` is returned.
    pub fn from_frame(frame: Frame) -> crate::Result<Command> {
        // The frame value is decorated with `Parse`. `Parse` provides a
        // "cursor" like API which makes parsing the command easier.
        //
        // The frame value must be an array variant. Any other frame variants
        // result in an error being returned.
        let mut parse = Parser::new(frame)?;

        // All redis commands begin with the command name as a string. The name
        // is read and converted to lower cases to do case-sensitive
        // matching.
        let command_name = parse.next_bytes()?;
        let command_name_lower = command_name.to_ascii_lowercase();

        // Match the command name, delegating the rest of the parsing to the
        // specific command.
        let command = match &command_name_lower[..] {
            b"get" => Command::Get(Get::parse(&mut parse)?),
            b"set" => Command::Set(Set::parse(&mut parse)?),
            b"echo" => Command::Echo(Echo::parse(&mut parse)?),
            b"ping" => Command::Ping(Ping::parse(&mut parse)?),

            // The command is not recognized and an Unknown command is
            // returned.
            //
            // `return` is called here to skip the `finish()` call below. As
            // the command is not recognized, there are most likely
            // unconsumed fields remaining in the `Parse` instance.
            _ => return Ok(Command::Unknown(Unknown::new(command_name))),
        };

        // Check if there are any remaining unconsumed fields in the `Parse`
        // value. If fields remain, this indicates an unexpected frame format
        // and an error is returned.
        parse.finish()?;

        // The command has been successfully parsed
        Ok(command)
    }
}
