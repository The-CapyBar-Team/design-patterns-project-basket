use crate::basket_pool::basket_pool::ProtectedBasketPool;
use crate::types::BasketId;
use basket_communication::basket_service::{ap_request, hp_request, ups_request};
use basket_communication::types::{ProductId, UserId};
use std::sync::Arc;

pub(crate) enum GrpcFailure<CustomError> {
    Custom(CustomError),
    Internal,
}

// UPS = "Update Product's Stock"
// HP = "Hold Product"
// AP = "Await Request"
pub(crate) trait RequestSender {
    async fn perform_ups_request(
        &mut self,
        basket_id: BasketId,
        request_args: ups_request::Request,
    ) -> Option<ups_request::Response>;

    async fn perform_hp_request(
        &mut self,
        basket_id: BasketId,
        request: hp_request::Request,
    ) -> Option<()>;

    async fn perform_ap_request(
        &mut self,
        basket_id: BasketId,
        request: ap_request::Request,
    ) -> Result<ap_request::Success, GrpcFailure<ap_request::Failure>>;
}

pub(crate) struct BasicRequestSender<BasketSet>
where
    BasketSet: Default + crate::basket_set::traits::BasketSet,
{
    basket_pool: Arc<ProtectedBasketPool<BasketSet>>,
}

impl<BasketSet> BasicRequestSender<BasketSet>
where
    BasketSet: Default + crate::basket_set::traits::BasketSet,
{
    pub(crate) fn new(basket_pool: Arc<ProtectedBasketPool<BasketSet>>) -> Self {
        Self { basket_pool }
    }
}

impl<BasketSet> RequestSender for BasicRequestSender<BasketSet>
where
    BasketSet: Default + crate::basket_set::traits::BasketSet,
{
    async fn perform_ups_request(
        &mut self,
        basket_id: BasketId,
        request: ups_request::Request,
    ) -> Option<ups_request::Response> {
        let mut basket_pool = self.basket_pool.basket_pool.lock().await;
        let channel = basket_pool.get_mut_basket_channel(basket_id)?;

        channel
            .perform_ups(request)
            .await
            .map(|response| response.into_inner())
            .ok()
    }

    async fn perform_hp_request(
        &mut self,
        basket_id: BasketId,
        request: hp_request::Request,
    ) -> Option<()> {
        use hp_request::response::Response::Failure;
        use hp_request::response::Response::Success;

        let response = {
            let mut basket_pool = self.basket_pool.basket_pool.lock().await;
            let channel = basket_pool.get_mut_basket_channel(basket_id)?;
            channel
                .perform_hp(request)
                .await
                .map(|response| response.into_inner().response)
        };

        match response {
            Ok(Some(Success(
                ref a @ hp_request::Success {
                    ref user_id,
                    product_id,
                },
            ))) => {
                println!("hp_request | received success: {:?}", a);
                Some(())
            }

            Ok(Some(Failure(hp_request::Failure {
                error_message,
                status,
            }))) => {
                println!(
                    "hp_request | received failure | status = {}, error_message = {}",
                    status, error_message
                );
                None
            }

            Err(err) => {
                println!("hp_request | received Err(err): {}", err);
                None
            }

            Ok(_) => {
                println!("hp_request | received Ok(None)");
                None
            }
        }
    }

    async fn perform_ap_request(
        &mut self,
        basket_id: BasketId,
        request: ap_request::Request,
    ) -> Result<ap_request::Success, GrpcFailure<ap_request::Failure>> {
        use ap_request::response::Response::Failure;
        use ap_request::response::Response::Success;

        let response = {
            let mut basket_pool = self.basket_pool.basket_pool.lock().await;
            let channel = basket_pool
                .get_mut_basket_channel(basket_id)
                .ok_or(GrpcFailure::Internal)?;
            channel
                .perform_ap(request)
                .await
                .map(|response| response.into_inner().response)
        };

        match response {
            Ok(Some(Success(ap_success))) => Ok(ap_success),
            Ok(Some(Failure(ap_failure))) => Err(GrpcFailure::Custom(ap_failure)),
            Ok(None) | Err(_) => Err(GrpcFailure::Internal),
        }
    }
}
