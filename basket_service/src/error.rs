use basket_communication::basket_service::{ap_request, hp_request, ru_request};
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
        "[UserId='`{1}`'] The user is already added to the context of the product with id {0}"
    )]
    UserAlreadyAdded(ProductId, UserId),

    #[error("[UserId='`{1}`'] Trying to await product with id '`{0}`' while its holders queue has not reached its max size")]
    PrematureAwait(ProductId, UserId),

    #[cfg(feature = "extra_protection")]
    #[error("[UserId='`{1}`'] The queue position {2} is already occupied for product with id {0}")]
    QueuePositionIsIncorrect(ProductId, UserId, QueuePosition),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub(crate) enum RuRequestError {
    #[error("[UserId='`{1}`'] Product with id '`{0}`' is not available")]
    ProductNotFound(ProductId, UserId),

    #[error(
        "Unable to remove user with id '{1}', as it is not present in the basket at product '{0}'"
    )]
    UserNotFound(ProductId, UserId),

    #[error("Removal DebugError: {0}")]
    DebugError(String),
}

pub(crate) struct LocallyLoggedError<Error> {
    pub(crate) error: Error,
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
pub(crate) fn ap_request_error_to_status(
    error: ApRequestError,
) -> Result<ap_request::FailureStatus, LocallyLoggedError<ApRequestError>> {
    use ap_request::FailureStatus as ProtoStatus;

    match error {
        ApRequestError::ProductNotFound(_, _) => Ok(ProtoStatus::ProductNotFound),
        ApRequestError::UserAlreadyAdded(_, _) => Ok(ProtoStatus::UserAlreadyAdded),
        ApRequestError::PrematureAwait(_, _) => Err(LocallyLoggedError { error }),

        #[cfg(feature = "extra_protection")]
        ApRequestError::QueuePositionIsIncorrect(_, _, _) => Err(LocallyLoggedError { error }),
    }
}

#[inline(always)]
pub(crate) fn ru_request_error_to_status(
    error: RuRequestError,
) -> Result<ru_request::FailureStatus, LocallyLoggedError<RuRequestError>> {
    use ru_request::FailureStatus as ProtoStatus;

    match error {
        RuRequestError::ProductNotFound(_, _) => Ok(ProtoStatus::ProductNotFound),
        RuRequestError::UserNotFound(_, _) => Ok(ProtoStatus::UserNotFound),
        RuRequestError::DebugError(_) => Err(LocallyLoggedError { error }),
    }
}
