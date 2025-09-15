use crate::BalancingTable;
use crate::basket_pool::basket_pool::wait_until_basket_is_ready;
use crate::requests;
use basket_communication::basket_balancer_requests::eh_request;
use basket_communication::rabbit::listener::RabbitListener;
use basket_communication::rabbit::sender::RabbitSender;
use basket_communication::{external, retry_and_report_error};
use std::sync::Arc;
use tokio::sync::Mutex;

#[inline(always)]
pub(crate) async fn listener<RequestSender, BasketBalancer>(
    rabbit_connection_string: String,
    balancing_table: Arc<Mutex<BalancingTable<RequestSender, BasketBalancer>>>,
    lost_product_sender: RabbitSender<external::LostProduct>,
) where
    RequestSender: requests::RequestSender + Send,
    BasketBalancer: Default
        + crate::basket_set::traits::BasketSet
        + crate::basket_set::traits::BasketBalancer
        + Send,
{
    let mut eh_listener = retry_and_report_error(async move || {
        RabbitListener::<eh_request::Request>::new(&rabbit_connection_string, "ExpiredHolders")
            .await
    })
    .await;

    wait_until_basket_is_ready().await;

    let balancing_table = balancing_table.clone();
    let lost_product_sender = Arc::new(Mutex::new(lost_product_sender));

    eh_listener
        .listen_to_messages(async move |eh_request::Request { expired_holders }| {
            let mut balancing_table = balancing_table.lock().await;

            for eh_request::ExpiredHolder {
                user_id,
                product_id,
            } in expired_holders
            {
                let _ = balancing_table
                    .remove_product_from_basket(product_id, user_id.clone(), false)
                    .await;

                println!(
                    "debug | timeout removing | product_id = {}, user_id = {}",
                    product_id, user_id
                );

                if let Err(err) = lost_product_sender
                    .lock()
                    .await
                    .send_message(external::LostProduct {
                        user_id,
                        product_id,
                    })
                    .await
                {
                    println!("!<>! LostProductSender | {}", err);
                }
            }
        })
        .await;
}
