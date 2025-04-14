#[cfg(test)]
mod tests {
    use queue_storage::redis_queue_storage::RedisQueueStorage;
    use queue_storage::traits::QueueStorage;

    const REDIS_CONNECTION_STRING: &str = "redis://redis:6379";

    #[tokio::test]
    async fn redis_connection() {
        let storage = RedisQueueStorage::new(REDIS_CONNECTION_STRING).await;
        storage.unwrap();
    }

    #[tokio::test]
    async fn queue_basic_operations() {
        const USERS_INSERTED: usize = 10;
        let mut storage = RedisQueueStorage::new(REDIS_CONNECTION_STRING)
            .await
            .unwrap();
        let queue_id = "product_1";

        storage
            .create_queue_of_max_size(&queue_id, USERS_INSERTED)
            .await
            .unwrap();

        for user_id in 0..USERS_INSERTED {
            let index = storage
                .push_user_id(&queue_id, &user_id.to_string())
                .await
                .unwrap();
            assert_eq!(index, user_id);
        }

        let error_push_result = storage
            .push_user_id(&queue_id, &"some_random_user_is".to_string())
            .await;
        assert!(error_push_result.is_err());

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
        let pushing_result = storage
            .push_user_id(&queue_id, &"some_user".to_string())
            .await;
        let popping_result = storage.pop_user_id(&queue_id).await;

        assert!(pushing_result.is_err());
        assert!(popping_result.is_err());
    }
}
