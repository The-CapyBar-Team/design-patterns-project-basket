use crate::types::BasketId;
use basket_communication::types::{ProductId, UserId};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub(crate) enum BalancerError {
    #[error("[UserId='`{1}`'] Product with id '`{0}`' is not available")]
    ProductNotFound(ProductId, UserId),
}
