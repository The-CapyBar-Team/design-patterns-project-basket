use crate::BalancingTable;
use crate::basket_pool::basket_pool::wait_until_basket_is_ready;
use crate::requests;
use crate::utilities::retry_and_report_error;
use basket_communication::external::BuyProductRequest;
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
        RabbitListener::<BuyProductRequest>::new(&rabbit_connection_string, "BuyProductRequests")
            .await
    })
    .await;

    wait_until_basket_is_ready().await;

    let balancing_table = balancing_table.clone();

    ups_rabbit_listener
        .listen_to_messages(
            async move |BuyProductRequest {
                            user_id,
                            product_id,
                        }| {
                let mut balancing_table = balancing_table.lock().await;
                balancing_table
                    .remove_product_from_basket(product_id, user_id.clone())
                    .await;
                balancing_table
                    .send_decrease_stock_request(product_id, user_id)
                    .await;
            },
        )
        .await;
}
