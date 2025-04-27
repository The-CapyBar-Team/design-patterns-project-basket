use crate::BalancingTable;
use crate::basket_pool::basket_pool::wait_until_basket_is_ready;
use crate::requests;
use crate::utilities::retry_and_report_error;
use basket_communication::external::AddToCartRequest;
use basket_communication::rabbit::listener::RabbitListener;
use std::sync::Arc;
use tokio::sync::Mutex;

#[inline(always)]
pub(crate) async fn listener<RequestSender, BasketBalancer>(
    rabbit_connection_string: String,
    balancing_table: Arc<Mutex<BalancingTable<RequestSender, BasketBalancer>>>,
) where
    RequestSender: requests::RequestSender + Send,
    BasketBalancer: Default
        + crate::basket_set::traits::BasketSet
        + crate::basket_set::traits::BasketBalancer
        + Send,
{
    let mut ups_rabbit_listener = retry_and_report_error(async move || {
        RabbitListener::<AddToCartRequest>::new(&rabbit_connection_string, "AddToCartRequest").await
    })
    .await;

    wait_until_basket_is_ready().await;

    ups_rabbit_listener
        .listen_to_messages(
            async move |AddToCartRequest {
                            user_id,
                            product_id,
                        }|
            {
                println!("debug | balancer | received AddToCartRequest: user_id = {}, product_id = {}", user_id, product_id);
                if let Err(hap_error) = balancing_table
                    .lock()
                    .await
                    .add_product_to_basket(product_id, user_id)
                    .await
                {
                    eprintln!("!<>! Adding to cart error: {}", hap_error);
                }
            },
        )
        .await;
}
