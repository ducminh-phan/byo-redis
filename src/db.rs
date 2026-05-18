use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::Instant,
};

use bytes::Bytes;
use tokio::sync::{mpsc, oneshot};

use crate::{
    command::{Apply, Command, CommandError},
    frame::Frame,
};

#[derive(Debug)]
pub enum Value {
    String(Bytes),
    List(VecDeque<Bytes>),
    Set(HashSet<Bytes>),
    Hash(HashMap<Bytes, Bytes>),
}

#[derive(Default)]
pub struct Db {
    pub data: HashMap<Bytes, Value>,
    pub expires: HashMap<Bytes, Instant>,
}

pub struct DbManager {
    db: Db,
    rx: mpsc::Receiver<DbRequest>,
}

pub struct DbRequest {
    pub command: Command,
    pub response: oneshot::Sender<Result<Frame, CommandError>>,
}

impl DbManager {
    pub fn new(rx: mpsc::Receiver<DbRequest>) -> Self {
        Self {
            db: Db::default(),
            rx,
        }
    }

    pub async fn run(&mut self) {
        while let Some(DbRequest { command, response }) = self.rx.recv().await {
            let result = command.apply(&mut self.db);
            let _ = response.send(result);
        }
    }
}
