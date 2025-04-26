use crate::basket_pool::basket_pool::ProtectedBasketPool;
use crate::types::BasketId;
use basket_communication::basket_service::hp_request;
use basket_communication::basket_service::ups_request;
use basket_communication::types::{ProductId, UserId};
use std::sync::Arc;

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

    fn perform_ap_request(&mut self, basket_id: BasketId, product_id: ProductId, user_id: UserId);
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

        let mut basket_pool = self.basket_pool.basket_pool.lock().await;
        let channel = basket_pool.get_mut_basket_channel(basket_id)?;

        match channel
            .perform_hp(request)
            .await
            .map(|response| response.into_inner().response)
        {
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

    fn perform_ap_request(&mut self, basket_id: BasketId, product_id: ProductId, user_id: UserId) {
        println!(
            "Delegating to basket #{} awaiting product: product_id = {}, user_id = {}",
            basket_id, product_id, user_id
        );
    }
}
