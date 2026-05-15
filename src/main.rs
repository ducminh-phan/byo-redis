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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let port = args.port.unwrap_or(6379);
    let bind_addr = args.bind.unwrap_or("127.0.0.1".to_string());

    println!("Listening on {}:{}", bind_addr, port);
    let listener = TcpListener::bind((bind_addr, port)).await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        tokio::spawn(async move {
            lib::server::process(stream, addr).await;
        });
    }
}
