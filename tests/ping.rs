use redis::AsyncTypedCommands;

mod common;

#[tokio::test]
async fn ping_no_arg() {
    let mut conn = common::connect().await;
    let reply: String = conn.ping().await.unwrap();
    assert_eq!(reply, "PONG");
}

#[tokio::test]
async fn ping_with_message() {
    let mut conn = common::connect().await;
    let reply: String = conn.ping_message("hello").await.unwrap();
    assert_eq!(reply, "hello");
}
