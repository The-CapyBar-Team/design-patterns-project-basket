use basket_communication::basket_service::{ap_request, hp_request};
use basket_communication::types::{ProductId, QueuePosition, UserId};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub(crate) enum HpRequestError {
    #[error("[UserId='`{1}`'] Product with id '`{0}`' is not available")]
    ProductNotFound(ProductId, UserId),

    #[error(
        "[UserId='`{1}`'] Cannot hold product with id `'{0}'`: holders queue reached its max size"
    )]
    HoldersQueueAlreadyFull(ProductId, UserId),

    #[error(
        "[UserId='`{1}`'] The user is already added to the context of the product with id {0}"
    )]
    UserAlreadyAdded(ProductId, UserId),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub(crate) enum ApRequestError {
    #[error("[UserId='`{1}`'] Product with id '`{0}`' is not available")]
    ProductNotFound(ProductId, UserId),

    #[error(
        "[UserId='`{1}`'] Cannot hold product with id `'{0}'`: holders queue reached its max size"
    )]
    HoldersQueueNotFullYet(ProductId, UserId),

    #[error(
        "[UserId='`{1}`'] The user is already added to the context of the product with id {0}"
    )]
    UserAlreadyAdded(ProductId, UserId),

    #[error("[UserId='`{1}`'] Trying to await product with id '`{0}`' while its holders queue has not reached its max size")]
    PrematureAwait(ProductId, UserId),

    #[cfg(feature = "extra_protection")]
    #[error("[UserId='`{1}`'] The queue position {2} is already occupied for product with id {0}")]
    QueuePositionIsIncorrect(ProductId, UserId, QueuePosition),
}

#[inline(always)]
pub(crate) fn hp_request_error_to_status(error: HpRequestError) -> hp_request::FailureStatus {
    use hp_request::FailureStatus as ProtoStatus;

    match error {
        HpRequestError::ProductNotFound(_, _) => ProtoStatus::ProductNotFound,
        HpRequestError::HoldersQueueAlreadyFull(_, _) => ProtoStatus::HoldersQueueAlreadyFull,
        HpRequestError::UserAlreadyAdded(_, _) => ProtoStatus::UserAlreadyAdded,
    }
}

#[inline(always)]
pub(crate) fn ap_request_error_to_status(error: ApRequestError) -> ap_request::FailureStatus {
    use ap_request::FailureStatus as ProtoStatus;

    match error {
        ApRequestError::ProductNotFound(_, _) => ProtoStatus::ProductNotFound,
        ApRequestError::HoldersQueueNotFullYet(_, _) => ProtoStatus::HoldersQueueNotFullYet,
        ApRequestError::UserAlreadyAdded(_, _) => ProtoStatus::UserAlreadyAdded,
        ApRequestError::PrematureAwait(_, _) => ProtoStatus::PrematureAwait,
        ApRequestError::QueuePositionIsIncorrect(_, _, _) => ProtoStatus::QueuePositionIsIncorrect,
    }
}
