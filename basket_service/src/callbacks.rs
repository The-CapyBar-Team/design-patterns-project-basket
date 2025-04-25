use crate::basket::{Basket, BasketError};
use basket_communication::basket_service::basket_service_server;
use basket_communication::basket_service::{hp_request, ups_request};
use std::sync::Mutex;
use tonic::{Request, Response, Status};

pub(crate) struct BasketContext {
    basket: Mutex<Basket>,
}

#[tonic::async_trait]
impl basket_service_server::BasketService for BasketContext {
    #[inline(always)]
    async fn perform_ups(
        &self,
        request: Request<ups_request::Request>,
    ) -> Result<Response<ups_request::Response>, Status> {
        // let args = request.into_inner();
        // println!(
        //     "basket_service | stock updated for product: product_id={} new_stock={}",
        //     args.product_id, args.product_stock
        // );
        // let mut basket = self.basket.lock().unwrap();
        // basket.update_product_stock(args.product_id, args.product_stock);

        // Ok(Response::new(ups_request::Response { ok: 1 }))
        todo!()
    }

    // TODO: What if user_id is not identical and one user can do hp twice? he would be able to
    #[inline(always)]
    async fn perform_hp(
        &self,
        request: Request<hp_request::Request>,
    ) -> Result<Response<hp_request::Response>, Status> {
        use hp_request::response::Response::Failure;
        use hp_request::response::Response::Success;

        let hp_request::Request {
            user_id,
            product_id,
        } = request.into_inner();

        println!("HP | user_id: {}, product_id: {}", user_id, product_id);

        let response = hp_request::Response {
            response: Some(Success(hp_request::Success {
                user_id: 121,
                product_id: 122,
                queue_position: 123,
                status: hp_request::SuccessStatus::ProductHeldByUser as i32,
            })),
        };

        Ok(Response::new(response))

        // let failure_response = hp_request::Response {
        //     response: Some(Failure(hp_request::Failure {
        //         error_message: todo!(),
        //         status: todo!(),
        //     })),
        // };
    }
}

impl BasketContext {
    #[inline(always)]
    pub(crate) fn new(basket: Basket) -> Self {
        Self {
            basket: Mutex::new(basket),
        }
    }

    // #[inline(always)]
    // pub(crate) fn update_product_stock_requeest_received(&mut self, args: ups_request::Request) {
    //     println!(
    //         "basket_service | stock updated for product: product_id={} new_stock={}",
    //         args.product_id, args.product_stock
    //     );
    //     self.basket
    //         .update_product_stock(args.product_id, args.product_stock);
    // }

    // #[inline(always)]
    // pub(crate) fn hold_product_request_received(
    //     &mut self,
    //     args: hp_request::Request,
    // ) -> hp_request::Response {
    //     match self
    //         .basket
    //         .add_product_holder(args.product_id, args.user_id, args.queue_position)
    //     {
    //         Ok(()) => {
    //             println!(
    //                 "basket_service | hold_product_request_received | successfully held product: product_id={} user_id={}",
    //                 args.product_id, args.user_id
    //             );
    //             hp_request::Response {
    //                 user_id: args.user_id,
    //                 product_id: args.product_id,
    //                 queue_position: args.queue_position,
    //                 status: hp_request::ProductStatus::ProductHeldByUser.into(),
    //             }
    //         }
    //         Err(error) => {
    //             eprintln!(
    //                 "basket_service | hold_product_request_received | Error: {}",
    //                 error
    //             );
    //             hp_request::Response {
    //                 user_id: args.user_id,
    //                 product_id: args.product_id,
    //                 queue_position: args.queue_position,
    //                 status: add_product_holder_error_to_status(error).into(),
    //             }
    //         }
    //     }
    // }

    // #[inline(always)]
    // pub(crate) fn await_product_request_received(
    //     &mut self,
    //     args: hp_request::Request,
    // ) -> hp_request::Response {
    //     match self
    //         .basket
    //         .add_product_awaiter(args.product_id, args.user_id, args.queue_position)
    //     {
    //         Ok(()) => {
    //             println!(
    //                 "basket_service | await_product_request_received | successfully held product: product_id={} user_id={}",
    //                 args.product_id, args.user_id
    //             );
    //             hp_request::Response {
    //                 user_id: args.user_id,
    //                 product_id: args.product_id,
    //                 queue_position: args.queue_position,
    //                 status: hp_request::ProductStatus::ProductHeldByUser.into(),
    //             }
    //         }
    //         Err(error) => {
    //             eprintln!(
    //                 "basket_service | await_product_request_received | Error: {}",
    //                 error
    //             );
    //             hp_request::Response {
    //                 user_id: args.user_id,
    //                 product_id: args.product_id,
    //                 queue_position: args.queue_position,
    //                 status: add_product_holder_error_to_status(error).into(),
    //             }
    //         }
    //     }
    // }
}

// #[inline(always)]
// fn add_product_holder_error_to_status(error: BasketError) -> hp_request::ProductStatus {
//     use hp_request::ProductStatus as ProtoStatus;

//     match error {
//         BasketError::ProductNotFound(_, _) => ProtoStatus::ProductNotFound,
//         BasketError::HoldersQueueAlreadyFull(_, _) => ProtoStatus::HoldersQueueAlreadyFull,
//         BasketError::PrematureAwait(_, _) => ProtoStatus::PrematureAwait,
//         BasketError::UserAlreadyAdded(_, _) => ProtoStatus::UserAlreadyAdded,
//         BasketError::QueuePositionIsIncorrect(_, _, _) => ProtoStatus::QueuePositionIsIncorrect,
//     }
// }
