use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn process(mut stream: TcpStream, addr: SocketAddr) {
    println!("Connection from {}", addr);
    stream
        .write_all(format!("Hello {:?}!\r\n", addr).as_bytes())
        .await
        .expect("Failed to write to socket")
}
