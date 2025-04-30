use crate::Basket;
use basket_communication::external::{ProductStatusRequest, ProductStatusUpdate};
use basket_communication::rabbit::listener::RabbitListener;
use basket_communication::rabbit::sender::RabbitSender;
use basket_communication::retry_and_report_error;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) async fn product_status_request_handler(basket_id: u32, basket: Arc<Mutex<Basket>>) {
    let mut rabbit_listener = retry_and_report_error(async move || {
        RabbitListener::<ProductStatusRequest>::new(
            "amqp://guest:guest@rabbitmq:5672",
            &format!("PSRequestsBasket{}", basket_id),
        )
        .await
    })
    .await;

    let product_status_update_sender = Arc::new(Mutex::new(
        retry_and_report_error(async move || {
            RabbitSender::new("amqp://guest:guest@rabbitmq:5672", "ProductStatusUpdates").await
        })
        .await,
    ));

    let basket = basket.clone();
    rabbit_listener
        .listen_to_messages(async move |ProductStatusRequest { user_id }| {
            let product_status_update_sender = product_status_update_sender.clone();
            let mut product_status_update_sender = product_status_update_sender.lock().await;
            let mut basket = basket.lock().await;
            let restored_basket = basket.restore_basket_for_user(user_id.clone());
            if let Err(err) = product_status_update_sender
                .send_message(ProductStatusUpdate {
                    user_id,
                    new_queue_positions: restored_basket,
                })
                .await
            {
                println!("!<>! | product_status_update_sender | {}", err);
            }
        })
        .await;
}
