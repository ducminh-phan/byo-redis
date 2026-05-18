use redis::aio::MultiplexedConnection;
use tokio::net::TcpListener;

pub async fn connect() -> MultiplexedConnection {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(byo_redis::server::run(listener));
    redis::Client::open(format!("redis://{addr}"))
        .unwrap()
        .get_multiplexed_async_connection()
        .await
        .unwrap()
}
