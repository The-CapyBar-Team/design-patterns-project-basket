use queue_storage::redis_queue_storage::RedisQueueStorage;
use queue_storage::traits::QueueStorage;
use std::{thread, time::Duration};

const REDIS_CONNECTION_STRING: &str = "redis://redis:6379";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut queue_storage = RedisQueueStorage::new(REDIS_CONNECTION_STRING).await?;

    for i in 0..usize::MAX {
        let key = (i % 10).to_string();
        let value = i.to_string();

        println!("Basket Service is writting ({}, {})", key, value);

        let push_result = queue_storage.push_user_id(&key, &value).await;

        if let Err(err) = push_result {
            println!("Pushing error: {}", err);
        }

        // let get_result: Result<Option<String>, _> = con.get(&key_value).await;

        // match get_result {
        //     Ok(Some(resolved_value)) => {
        //         println!("Resolved value for key {}: {}", key_value, resolved_value)
        //     }
        //     Ok(None) => {
        //         println!("Value for key {} does not exist.", key_value)
        //     }
        //     Err(err) => {
        //         println!("Could not resolve value for key {}: {}", key_value, err)
        //     }
        // }

        thread::sleep(Duration::from_secs(3));
    }

    Ok(())
}
