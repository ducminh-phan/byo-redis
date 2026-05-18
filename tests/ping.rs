mod common;

#[tokio::test]
async fn ping_no_arg() {
    let mut conn = common::connect().await;
    let reply: String = redis::cmd("PING").query_async(&mut conn).await.unwrap();
    assert_eq!(reply, "PONG");
}

#[tokio::test]
async fn ping_with_message() {
    let mut conn = common::connect().await;
    let reply: String = redis::cmd("PING")
        .arg("hello")
        .query_async(&mut conn)
        .await
        .unwrap();
    assert_eq!(reply, "hello");
}
