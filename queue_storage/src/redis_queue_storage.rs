use crate::traits::{Error, QueueStorage, UserId};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, RedisError};

pub struct RedisQueueStorage {
    connection: MultiplexedConnection,
}

impl RedisQueueStorage {
    #[inline(always)]
    pub async fn new(connection_string: &str) -> Result<Self, Error> {
        let client = Client::open(connection_string).map_err(redis_error_to_native)?;
        let connection = client
            .get_multiplexed_async_connection()
            .await
            .map_err(redis_error_to_native)?;

        Ok(Self { connection })
    }
}

impl QueueStorage for RedisQueueStorage {
    #[inline(always)]
    async fn push_user_id(&mut self, queue_id: &str, user_id: &UserId) -> Result<(), Error> {
        self.connection
            .rpush(queue_id, user_id)
            .await
            .map_err(redis_error_to_native)
    }

    #[inline(always)]
    async fn pop_user_id(&mut self, queue_id: &str) -> Result<(), Error> {
        self.connection
            .rpop(queue_id, None)
            .await
            .map_err(redis_error_to_native)
    }
}

#[inline(always)]
fn redis_error_to_native(redis_error: RedisError) -> Error {
    Error {
        message: redis_error.to_string()
    }
}
