use redis::{AsyncTypedCommands, aio::MultiplexedConnection};

mod common;

#[tokio::test]
async fn get_set() {
    let mut conn = common::connect().await;
    let value: Option<String> = conn.get("key").await.unwrap();
    assert!(value.is_none());

    assert_eq!(get_db_size(&mut conn).await, 0);

    conn.set("key", "value").await.unwrap();
    assert_eq!(get_db_size(&mut conn).await, 1);

    let value: Option<String> = conn.get("key").await.unwrap();
    assert!(value.is_some());
    assert_eq!(value.unwrap(), "value");
}

async fn get_db_size(conn: &mut MultiplexedConnection) -> usize {
    redis::cmd("DBSIZE").query_async(conn).await.unwrap()
}
