use std::future::Future;
use std::fmt::{Display, Formatter};

pub type UserId = String;

pub trait QueueStorage {
    fn push_user_id(
        &mut self,
        queue_id: &str,
        user_id: &UserId,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    fn pop_user_id(&mut self, queue_id: &str) -> impl Future<Output = Result<(), Error>> + Send;
}

#[derive(Debug)]
pub struct Error {
    pub(crate) message: String
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for Error {}
