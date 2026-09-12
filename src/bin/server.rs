use std::{net::SocketAddr, str::FromStr};

use clap::Parser;
use tokio::net::TcpListener;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, help = "Listen on specified port (default: 6379)")]
    port: Option<u16>,

    #[arg(
        short,
        long,
        help = "Bind to specified IP address(es) (default: 127.0.0.1)"
    )]
    bind: Option<String>,

    #[arg(long, help = "Port for metrics server (default: 9000)")]
    metrics_port: Option<u16>,

    #[arg(long, help = "Bind address for metrics server (default: 127.0.0.1)")]
    metrics_bind: Option<String>,
}

impl Args {
    fn get_addr(&self) -> String {
        let port = self.port.unwrap_or(6379);
        let addr = self.bind.clone().unwrap_or("127.0.0.1".to_string());
        format!("{}:{}", addr, port)
    }

    fn get_metrics_addr(&self) -> SocketAddr {
        let port = self.metrics_port.unwrap_or(9000);
        let addr = self.metrics_bind.clone().unwrap_or("127.0.0.1".to_string());
        SocketAddr::from_str(&format!("{}:{}", addr, port)).unwrap()
    }
}

#[tokio::main]
async fn main() -> byo_redis::Result<()> {
    let args = Args::parse();

    let addr = args.get_addr();
    println!("Listening on {}", addr);

    let metrics_addr = args.get_metrics_addr();
    println!("Metrics server listening on {}", metrics_addr);

    let listener = TcpListener::bind(addr).await?;
    byo_redis::server::run(listener, metrics_addr).await?;

    Ok(())
}
