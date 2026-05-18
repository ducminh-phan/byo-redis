use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot},
};

use crate::{
    command::Command,
    connection::Connection,
    db::{DbManager, DbRequest},
};

struct Server {
    listener: TcpListener,
}

impl Server {
    fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    pub async fn run(self) {
        let (tx, rx) = mpsc::channel::<DbRequest>(256);

        tokio::spawn(async move {
            let mut manager = DbManager::new(rx);
            manager.run().await;
        });

        loop {
            let (stream, addr) = self.listener.accept().await.unwrap();
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut handler = Handler {
                    connection: Connection::new(stream),
                    tx,
                };
                if let Err(e) = handler.handle().await {
                    eprintln!("Error handling connection from {addr}: {e}");
                }
            });
        }
    }
}

pub async fn run(listener: TcpListener) {
    Server::new(listener).run().await;
}

struct Handler {
    connection: Connection,
    tx: mpsc::Sender<DbRequest>,
}

impl Handler {
    async fn handle(&mut self) -> crate::Result<()> {
        loop {
            let maybe_frame = self.connection.read_frame().await;
            let frame = match maybe_frame {
                Ok(Some(frame)) => frame,
                Ok(None) => return Ok(()),
                Err(e) => return Err(e.into()),
            };

            let command = Command::from_frame(frame)?;
            let (tx, rx) = oneshot::channel();
            self.tx
                .send(DbRequest {
                    command,
                    response: tx,
                })
                .await
                .expect("Failed to send command to db manager");

            self.connection.write_frame(&rx.await??).await?;
        }
    }
}
