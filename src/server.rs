use std::net::SocketAddr;

use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

use crate::db::{DbManager, DbRequest};

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
                process(stream, addr, tx).await;
            });
        }
    }
}

pub async fn run(listener: TcpListener) {
    Server::new(listener).run().await;
}

async fn process(mut stream: TcpStream, addr: SocketAddr, _tx: mpsc::Sender<DbRequest>) {
    println!("Connection from {}", addr);
    stream
        .write_all(format!("Hello {:?}!\r\n", addr).as_bytes())
        .await
        .expect("Failed to write to socket")

    // TODO: Read commands from socket and process them
}
