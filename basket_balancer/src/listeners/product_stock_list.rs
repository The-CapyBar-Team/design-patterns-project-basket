use crate::BalancingTable;
use crate::basket_pool::basket_pool::wait_until_basket_is_ready;
use crate::requests;
use crate::utilities::retry_and_report_error;
use basket_communication::external::{ProductStockInfo, ProductStockList};
use basket_communication::rabbit::listener::RabbitListener;
use std::cell::RefCell;
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
        RabbitListener::<ProductStockList>::new(
            &rabbit_connection_string.clone(),
            "ProductStockLists",
        )
        .await
    })
    .await;

    wait_until_basket_is_ready().await;

    let balancing_table = balancing_table.clone();

    ups_rabbit_listener
        .listen_to_messages(async move |message| {
            let ProductStockList { ref products } = message;

            for ProductStockInfo {
                product_id,
                stock_change,
            } in products.into_iter()
            {
                balancing_table
                    .lock()
                    .await
                    .update_product_stock(*product_id, *stock_change)
                    .await;
            }
        })
        .await;
}
