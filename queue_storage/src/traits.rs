use std::fmt::{Display, Formatter};
use std::future::Future;

pub type UserId = String;

pub trait QueueStorage {
    fn create_queue_of_max_size(
        &mut self,
        queue_id: &str,
        max_size: usize,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    fn push_user_id(
        &mut self,
        queue_id: &str,
        user_id: &UserId,
    ) -> impl Future<Output = Result<usize, Error>> + Send;

    fn pop_user_id(&mut self, queue_id: &str) -> impl Future<Output = Result<(), Error>> + Send;

    fn queue_length(&mut self, queue_id: &str) -> impl Future<Output = Result<usize, Error>> + Send;
}

#[derive(Debug)]
pub struct Error {
    pub(crate) message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for Error {}
