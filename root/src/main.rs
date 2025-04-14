use queue_storage::redis_queue_storage::RedisQueueStorage;
use queue_storage::traits::QueueStorage;
use std::{thread, time::Duration};

const REDIS_CONNECTION_STRING: &str = "redis://redis:6379";
const NUMBER_OF_QUEUES: usize = 10;
const MAX_SIZE_OF_EACH_QUEUE: usize = 5;

async fn handle_iteration(iteration: usize, queue_storage: &mut RedisQueueStorage) -> anyhow::Result<()> {
    let key = (iteration % NUMBER_OF_QUEUES).to_string();
    let value = iteration.to_string();

    println!("Basket Service is pushing user_id='{}' to '{}' queue)", value, key);

    let index = queue_storage.push_user_id(&key, &value).await?;
    println!("Basket Service pushed user_id='{}' at {} index to '{}' queue)", value, index, key);

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut queue_storage = RedisQueueStorage::new(REDIS_CONNECTION_STRING).await?;

    for queue_index in 0..NUMBER_OF_QUEUES {
        queue_storage.create_queue_of_max_size(&queue_index.to_string(), MAX_SIZE_OF_EACH_QUEUE).await?;
    }

    for iteration in 0..usize::MAX {
        match handle_iteration(iteration, &mut queue_storage).await {
            Err(error) => println!("Error on iteration #{}: {}", iteration, error),
            _ => {},
        };
        thread::sleep(Duration::from_secs(3));
    }

    Ok(())
}
