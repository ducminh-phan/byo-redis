use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

struct Server {
    listener: TcpListener,
    db: crate::Db,
}

impl Server {
    fn new(listener: TcpListener) -> Self {
        Self {
            listener,
            db: crate::Db::default(),
        }
    }

    pub async fn run(self) {
        loop {
            let (stream, addr) = self.listener.accept().await.unwrap();
            let db = self.db.clone();
            tokio::spawn(async move {
                process(stream, addr, db).await;
            });
        }
    }
}

pub async fn run(listener: TcpListener) {
    Server::new(listener).run().await;
}

async fn process(mut stream: TcpStream, addr: SocketAddr, _db: crate::Db) {
    println!("Connection from {}", addr);
    stream
        .write_all(format!("Hello {:?}!\r\n", addr).as_bytes())
        .await
        .expect("Failed to write to socket")
}
