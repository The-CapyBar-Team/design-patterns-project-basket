use crate::basket::Basket;
use crate::error::{ap_request_error_to_status, hp_request_error_to_status};
use basket_communication::basket_service::basket_service_server;
use basket_communication::basket_service::{ap_request, hp_request, ups_request};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

pub(crate) struct BasketContext {
    basket: Mutex<Basket>,
}

impl BasketContext {
    #[inline(always)]
    pub(crate) fn new(basket: Basket) -> Self {
        Self {
            basket: Mutex::new(basket),
        }
    }
}

#[tonic::async_trait]
impl basket_service_server::BasketService for BasketContext {
    // TODO: when stock is increased, make some awaiters holders
    // TODO: when awaiters become holders the following
    //       invariant inconsistency may happen:
    //       basket: [holders, awaiters]
    //       [10/10, 0], [10/10, 5]
    //       after ups (+3 on each):
    //       [10/13, 0], [13/13, 2]
    // That's an issue, as there're 3 free holder-places, while
    // there still 2 awaiters!
    // We can resolve this by counting awaiters of each basket
    // on balancer (by sending the new awaiters count as responses
    // from modifying requests).
    #[inline(always)]
    async fn perform_ups(
        &self,
        request: Request<ups_request::Request>,
    ) -> Result<Response<ups_request::Response>, Status> {
        let ups_request::Request {
            product_id,
            product_stock_increase,
        } = request.into_inner();

        println!("debug | basket_service | received ups request: product_id = {}, product_stock_increase = {}", product_id, product_stock_increase);

        let mut basket = self.basket.lock().await;
        basket.update_product_stock(product_id, product_stock_increase);
        drop(basket);

        Ok(Response::new(ups_request::Response { queue_shift: 0 }))
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

        let mut basket = self.basket.lock().await;
        let holding_result = basket.add_product_holder(product_id, user_id.clone());
        drop(basket);

        let response = match holding_result {
            Ok(()) => hp_request::Response {
                response: Some(Success(hp_request::Success {
                    user_id: user_id.clone(),
                    product_id,
                })),
            },
            Err(err) => hp_request::Response {
                response: Some(Failure(hp_request::Failure {
                    error_message: err.to_string(),
                    status: hp_request_error_to_status(err).into(),
                })),
            },
        };

        Ok(Response::new(response))
    }

    #[inline(always)]
    async fn perform_ap(
        &self,
        request: Request<ap_request::Request>,
    ) -> Result<Response<ap_request::Response>, Status> {
        use ap_request::response::Response::Failure;
        use ap_request::response::Response::Success;

        let ap_request::Request {
            user_id,
            product_id,
            queue_position,
        } = request.into_inner();

        let mut basket = self.basket.lock().await;
        let holding_result =
            basket.add_product_awaiter(product_id, user_id.clone(), queue_position);
        drop(basket);

        let response = match holding_result {
            Ok(()) => ap_request::Response {
                response: Some(Success(ap_request::Success {
                    user_id: user_id.clone(),
                    product_id,
                    queue_position,
                })),
            },
            Err(err) => ap_request::Response {
                response: Some(Failure(ap_request::Failure {
                    error_message: err.to_string(),
                    status: ap_request_error_to_status(err).into(),
                })),
            },
        };

        Ok(Response::new(response))
    }
}
