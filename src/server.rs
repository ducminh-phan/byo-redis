use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn run(self) {
        loop {
            let (stream, addr) = self.listener.accept().await.unwrap();
            tokio::spawn(async move {
                process(stream, addr).await;
            });
        }
    }
}

pub async fn run(listener: TcpListener) {
    Server { listener }.run().await;
}

async fn process(mut stream: TcpStream, addr: SocketAddr) {
    println!("Connection from {}", addr);
    stream
        .write_all(format!("Hello {:?}!\r\n", addr).as_bytes())
        .await
        .expect("Failed to write to socket")
}
