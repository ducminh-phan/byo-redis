mod common;

#[tokio::test]
async fn echo_message() {
    let mut conn = common::connect().await;
    let reply: String = redis::cmd("ECHO")
        .arg("Hello World!")
        .query_async(&mut conn)
        .await
        .unwrap();
    assert_eq!(reply, "Hello World!");
}
