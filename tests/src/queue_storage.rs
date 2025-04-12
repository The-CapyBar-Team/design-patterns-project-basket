#[cfg(test)]
mod tests {
    use queue_storage::redis_queue_storage::RedisQueueStorage;
    use queue_storage::traits::QueueStorage;
    // use std::sync::atomic::{AtomicUsize, Ordering};

    const REDIS_CONNECTION_STRING: &str = "redis://redis:6379";

    // static mut CURRENT_QUEUE_ID: AtomicUsize = AtomicUsize::new(0usize);

    // #[inline(always)]
    // fn next_queue_id() -> String {
    //     unsafe { CURRENT_QUEUE_ID.fetch_add(1, Ordering::Relaxed) };
    //     format!("product_{}", unsafe { CURRENT_QUEUE_ID.load(Ordering::Relaxed) })
    // }

    #[tokio::test]
    async fn redis_connection() {
        let storage = RedisQueueStorage::new(REDIS_CONNECTION_STRING).await;
        storage.unwrap();
    }

    #[tokio::test]
    async fn queue_basic_operations() {
        const USERS_INSERTED: usize = 1000;
        let mut storage = RedisQueueStorage::new(REDIS_CONNECTION_STRING)
            .await
            .unwrap();
        let queue_id = "product_1";
        let max_size = 10usize;

        storage
            .create_queue_of_max_size(&queue_id, max_size)
            .await
            .unwrap();

        for user_id in 0..USERS_INSERTED {
            let index = storage.push_user_id(&queue_id, &user_id.to_string()).await.unwrap();
            assert_eq!(index, user_id);
        }

        let queue_length = storage.queue_length(queue_id).await.unwrap();
        assert_eq!(queue_length, USERS_INSERTED);

        for _ in 0..USERS_INSERTED {
            storage.pop_user_id(&queue_id).await.unwrap();
        }

        let queue_length = storage.queue_length(queue_id).await.unwrap();
        assert_eq!(queue_length, 0);
    }

    #[tokio::test]
    async fn interaction_with_non_existent_queue() {
        let mut storage = RedisQueueStorage::new(REDIS_CONNECTION_STRING)
            .await
            .unwrap();
        let queue_id = "non_existent_product";
        let pushing_result = storage.push_user_id(&queue_id, &"some_user".to_string()).await;
        let popping_result = storage.pop_user_id(&queue_id).await;

        assert!(pushing_result.is_err());
        assert!(popping_result.is_err())
    }
}
