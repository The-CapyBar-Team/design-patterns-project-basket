use crate::basket::{Basket, BasketError};
use basket_communication::{hold_or_await_product_request, update_product_stock_request};

pub(crate) struct BasketContext {
    pub basket: Basket,
}

impl BasketContext {
    #[inline(always)]
    pub(crate) fn update_product_stock_requeest_received(
        &mut self,
        args: update_product_stock_request::Request,
    ) {
        println!(
            "basket_service | stock updated for product: product_id={} new_stock={}",
            args.product_id, args.product_stock
        );
        self.basket
            .update_product_stock(args.product_id, args.product_stock);
    }

    #[inline(always)]
    pub(crate) fn hold_product_request_received(
        &mut self,
        args: hold_or_await_product_request::Request,
    ) -> hold_or_await_product_request::Response {
        match self
            .basket
            .add_product_holder(args.product_id, args.user_id, args.queue_position)
        {
            Ok(()) => {
                println!(
                    "basket_service | hold_product_request_received | successfully held product: product_id={} user_id={}",
                    args.product_id, args.user_id
                );
                hold_or_await_product_request::Response {
                    user_id: args.user_id,
                    product_id: args.product_id,
                    queue_position: args.queue_position,
                    status: hold_or_await_product_request::ProductStatus::ProductHeldByUser.into(),
                }
            }
            Err(error) => {
                eprintln!(
                    "basket_service | hold_product_request_received | Error: {}",
                    error
                );
                hold_or_await_product_request::Response {
                    user_id: args.user_id,
                    product_id: args.product_id,
                    queue_position: args.queue_position,
                    status: add_product_holder_error_to_status(error).into(),
                }
            }
        }
    }

    #[inline(always)]
    pub(crate) fn await_product_request_received(
        &mut self,
        args: hold_or_await_product_request::Request,
    ) -> hold_or_await_product_request::Response {
        match self
            .basket
            .add_product_awaiter(args.product_id, args.user_id, args.queue_position)
        {
            Ok(()) => {
                println!(
                    "basket_service | await_product_request_received | successfully held product: product_id={} user_id={}",
                    args.product_id, args.user_id
                );
                hold_or_await_product_request::Response {
                    user_id: args.user_id,
                    product_id: args.product_id,
                    queue_position: args.queue_position,
                    status: hold_or_await_product_request::ProductStatus::ProductHeldByUser.into(),
                }
            }
            Err(error) => {
                eprintln!(
                    "basket_service | await_product_request_received | Error: {}",
                    error
                );
                hold_or_await_product_request::Response {
                    user_id: args.user_id,
                    product_id: args.product_id,
                    queue_position: args.queue_position,
                    status: add_product_holder_error_to_status(error).into(),
                }
            }
        }
    }
}

#[inline(always)]
fn add_product_holder_error_to_status(
    error: BasketError,
) -> hold_or_await_product_request::ProductStatus {
    use hold_or_await_product_request::ProductStatus as ProtoStatus;

    match error {
        BasketError::ProductNotFound(_, _) => ProtoStatus::ProductNotFound,
        BasketError::HoldersQueueAlreadyFull(_, _) => ProtoStatus::HoldersQueueAlreadyFull,
        BasketError::PrematureAwait(_, _) => ProtoStatus::PrematureAwait,
        BasketError::UserAlreadyAdded(_, _) => ProtoStatus::UserAlreadyAdded,
        BasketError::QueuePositionIsIncorrect(_, _, _) => ProtoStatus::QueuePositionIsIncorrect,
    }
}
