use crate::basket::{AddProductHolderError, Basket};
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
            .add_product_holder(args.product_id, args.user_id)
        {
            Ok(()) => {
                println!(
                    "basket_service | successfully held product: product_id={} user_id={}",
                    args.product_id, args.user_id
                );
                hold_or_await_product_request::Response {
                    user_id: args.user_id,
                    product_id: args.product_id,
                    status: hold_or_await_product_request::ProductStatus::ProductHeldByUser.into(),
                }
            }
            Err(err @ AddProductHolderError::ProductNotFound(product_id, user_id)) => {
                eprintln!("basket_service | Error: {}", err);
                hold_or_await_product_request::Response {
                    user_id: user_id,
                    product_id: product_id,
                    status: hold_or_await_product_request::ProductStatus::ProductNotFound.into(),
                }
            }
            Err(err @ AddProductHolderError::HoldersQueueAlreadyFull(product_id, user_id)) => {
                eprintln!("basket_service | Error: {}", err);
                hold_or_await_product_request::Response {
                    user_id: user_id,
                    product_id: product_id,
                    status: hold_or_await_product_request::ProductStatus::HoldersQueueAlreadyFull
                        .into(),
                }
            }
            Err(err @ AddProductHolderError::PrematureAwait(product_id, user_id)) => {
                eprintln!("basket_service | Error: {}", err);
                hold_or_await_product_request::Response {
                    user_id: user_id,
                    product_id: product_id,
                    status: hold_or_await_product_request::ProductStatus::PrematureAwait.into(),
                }
            }
            Err(err @ AddProductHolderError::UserAlreadyAdded(product_id, user_id)) => {
                eprintln!("basket_service | Error: {}", err);
                hold_or_await_product_request::Response {
                    user_id: user_id,
                    product_id: product_id,
                    status: hold_or_await_product_request::ProductStatus::PrematureAwait.into(),
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
            .add_product_awaiter(args.product_id, args.user_id)
        {
            Ok(()) => {
                println!(
                    "basket_service | successfully held product: product_id={} user_id={}",
                    args.product_id, args.user_id
                );
                hold_or_await_product_request::Response {
                    user_id: args.user_id,
                    product_id: args.product_id,
                    status: hold_or_await_product_request::ProductStatus::ProductHeldByUser.into(),
                }
            }
            Err(err @ AddProductHolderError::ProductNotFound(product_id, user_id)) => {
                eprintln!("basket_service | Error: {}", err);
                hold_or_await_product_request::Response {
                    user_id: user_id,
                    product_id: product_id,
                    status: hold_or_await_product_request::ProductStatus::ProductNotFound.into(),
                }
            }
            Err(err @ AddProductHolderError::HoldersQueueAlreadyFull(product_id, user_id)) => {
                eprintln!("basket_service | Error: {}", err);
                hold_or_await_product_request::Response {
                    user_id: user_id,
                    product_id: product_id,
                    status: hold_or_await_product_request::ProductStatus::HoldersQueueAlreadyFull
                        .into(),
                }
            }
            Err(err @ AddProductHolderError::PrematureAwait(product_id, user_id)) => {
                eprintln!("basket_service | Error: {}", err);
                hold_or_await_product_request::Response {
                    user_id: user_id,
                    product_id: product_id,
                    status: hold_or_await_product_request::ProductStatus::PrematureAwait.into(),
                }
            }
            Err(err @ AddProductHolderError::UserAlreadyAdded(product_id, user_id)) => {
                eprintln!("basket_service | Error: {}", err);
                hold_or_await_product_request::Response {
                    user_id: user_id,
                    product_id: product_id,
                    status: hold_or_await_product_request::ProductStatus::PrematureAwait.into(),
                }
            }
        }
    }
}
