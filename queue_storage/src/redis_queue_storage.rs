use crate::traits::{Error, QueueStorage, UserId};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, RedisError};
use std::num::NonZero;

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
    async fn create_queue_of_max_size(
        &mut self,
        queue_id: &str,
        max_size: usize,
    ) -> Result<(), Error> {
        self.connection
            .set(queue_id_to_queue_max_size_id(queue_id), max_size)
            .await
            .map_err(redis_error_to_native)
    }

    #[inline(always)]
    async fn push_user_id(&mut self, queue_id: &str, user_id: &UserId) -> Result<usize, Error> {
        let queue_max_size: Option<usize> = self
            .connection
            .get(queue_id_to_queue_max_size_id(queue_id))
            .await
            .map_err(redis_error_to_native)?;

        let queue_max_size = queue_max_size.ok_or(Error {
            message: format!(
                "Queue with id '{}' not found. First create it with create_queue_of_max_size.",
                queue_id
            ),
        })?;

        let queue_actual_size: usize = self.queue_length(queue_id).await?;

        if queue_actual_size >= queue_max_size {
            return Err(Error {
                message: format!("Queue with id '{}' is full.", queue_id),
            });
        }

        let _: () = self
            .connection
            .rpush(queue_id_to_queue_list_id(queue_id), user_id)
            .await
            .map_err(redis_error_to_native)?;

        Ok(queue_actual_size)
    }

    #[inline(always)]
    async fn pop_user_id(&mut self, queue_id: &str) -> Result<(), Error> {
        let popped_user_id: Option<[String; 1]> = self
            .connection
            .rpop(queue_id_to_queue_list_id(queue_id), NonZero::new(1))
            .await
            .map_err(redis_error_to_native)?;

        if popped_user_id.is_none() {
            Err(Error {
                message: format!("Queue '{}' is empty. Nothing to pop.", queue_id),
            })
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    async fn queue_length(&mut self, queue_id: &str) -> Result<usize, Error> {
        self.connection
            .llen(queue_id_to_queue_list_id(queue_id))
            .await
            .map_err(redis_error_to_native)
    }
}

#[inline(always)]
fn queue_id_to_queue_max_size_id(queue_id: &str) -> String {
    format!("queue:{}:max_size", queue_id)
}

#[inline(always)]
fn queue_id_to_queue_list_id(queue_id: &str) -> String {
    format!("queue:{}", queue_id)
}

#[inline(always)]
fn redis_error_to_native(redis_error: RedisError) -> Error {
    Error {
        message: redis_error.to_string(),
    }
}
