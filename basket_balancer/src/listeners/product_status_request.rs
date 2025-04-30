use crate::BalancingTable;
use crate::basket_pool::basket_pool::wait_until_basket_is_ready;
use crate::requests;
use basket_communication::external::ProductStatusRequest;
use basket_communication::rabbit::listener::RabbitListener;
use basket_communication::rabbit::sender::RabbitSender;
use basket_communication::retry_and_report_error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[inline(always)]
pub(crate) async fn listener<RequestSender, BasketBalancer>(
    rabbit_connection_string: String,
    _balancing_table: Arc<Mutex<BalancingTable<RequestSender, BasketBalancer>>>,
    senders: Arc<Mutex<Vec<RabbitSender<ProductStatusRequest>>>>,
) where
    RequestSender: requests::RequestSender + Send,
    BasketBalancer: Default
        + crate::basket_set::traits::BasketSet
        + crate::basket_set::traits::BasketBalancer
        + Send,
{
    let mut ups_rabbit_listener = retry_and_report_error(async move || {
        RabbitListener::<ProductStatusRequest>::new(
            &rabbit_connection_string,
            "ProductStatusRequests",
        )
        .await
    })
    .await;

    wait_until_basket_is_ready().await;

    let senders = senders.clone();
    ups_rabbit_listener
        .listen_to_messages(async move |message| {
            let mut senders = senders.lock().await;
            for sender in senders.iter_mut() {
                if let Err(err) = sender.send_message(message.clone()).await {
                    println!("!<>! | product_status_request sender | {}", err);
                }
            }
        })
        .await;
}
