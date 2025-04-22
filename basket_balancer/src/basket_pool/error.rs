use crate::types::BasketId;
use basket_communication::basket_balancer::cb_request;
use basket_communication::types::{ProductId, UserId};
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum BasketPoolError {
    #[error("Unable to establish channel (connection) with id {0}: connection pool is full")]
    ConnectionPoolIsFull(BasketId),

    #[error("Channel (connection) with id '`{0}`' is already established")]
    AlreadyConnected(BasketId),

    #[error("Unable to establish channel (connection) with id {0}: {1}")]
    InternalError(BasketId, Box<dyn Error>),
}

#[inline(always)]
pub(crate) fn add_product_holder_error_to_status(
    error: BasketPoolError,
) -> cb_request::ConnectionStatus {
    use cb_request::ConnectionStatus as ProtoStatus;

    match error {
        BasketPoolError::ConnectionPoolIsFull(_) => ProtoStatus::ConnectionPoolIsFull,
        BasketPoolError::AlreadyConnected(_) => ProtoStatus::AlreadyConnected,
        BasketPoolError::InternalError(_, _) => ProtoStatus::InternalError,
    }
}
