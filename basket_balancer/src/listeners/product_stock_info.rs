use crate::BalancingTable;
use crate::basket_pool::basket_pool::wait_until_basket_is_ready;
use crate::requests;
use crate::utilities::retry_and_report_error;
use basket_communication::external::ProductStockInfo;
use basket_communication::rabbit::listener::RabbitListener;
use std::cell::RefCell;
use std::sync::Arc;
use tokio::sync::Mutex;

#[inline(always)]
pub(crate) async fn listener<RequestSender, BasketBalancer>(
    rabbit_connection_string: String,
    _balancing_table: Arc<Mutex<BalancingTable<RequestSender, BasketBalancer>>>,
) where
    RequestSender: requests::RequestSender + Send,
    BasketBalancer: Default
        + crate::basket_set::traits::BasketSet
        + crate::basket_set::traits::BasketBalancer
        + Send,
{
    let mut ups_rabbit_listener = retry_and_report_error(async move || {
        RabbitListener::<ProductStockInfo>::new(&rabbit_connection_string, "ProductStockInfo").await
    })
    .await;

    wait_until_basket_is_ready().await;

    ups_rabbit_listener
        .listen_to_messages(async |message| {
            println!("Rabbit | ProductStockInfo: {:?}", message);
        })
        .await;
}
