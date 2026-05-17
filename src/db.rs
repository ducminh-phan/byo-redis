use std::collections::{HashMap, HashSet, VecDeque};

use bytes::Bytes;
use tokio::sync::{mpsc, oneshot};

use crate::{command::Command, frame::Frame};

#[derive(Debug)]
enum Value {
    String(Bytes),
    List(VecDeque<Bytes>),
    Set(HashSet<Bytes>),
    Hash(HashMap<Bytes, Bytes>),
}

pub struct DbManager {
    db: HashMap<Bytes, Value>,
    rx: mpsc::Receiver<DbRequest>,
}

pub struct DbRequest {
    command: Command,
    response: oneshot::Sender<Result<Frame, CommandError>>,
}

pub enum CommandError {
    UnknownCommand,
    WrongNumberOfArguments,
    InvalidArgument,
    InternalError,
}

impl DbManager {
    pub fn new(rx: mpsc::Receiver<DbRequest>) -> Self {
        Self {
            db: HashMap::new(),
            rx,
        }
    }

    pub async fn run(&mut self) {
        while let Some(DbRequest { command, response }) = self.rx.recv().await {
            let result = self.apply(command);
            let _ = response.send(result);
        }
    }

    pub fn apply(&mut self, command: Command) -> Result<Frame, CommandError> {
        match command {
            Command::Get => Ok(Frame::Simple("Hello World!".into())),
        }
    }
}
