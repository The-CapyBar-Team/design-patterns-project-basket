use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum BasketSetError {
    #[error("")]
    Basket(BasketId),
}
