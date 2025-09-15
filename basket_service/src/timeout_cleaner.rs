use crate::basket::{current_timestamp, Basket};
use basket_communication::basket_balancer_requests::eh_request;
use basket_communication::rabbit::sender::RabbitSender;
use basket_communication::retry_and_report_error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

pub(crate) async fn timeout_cleaner(basket: Arc<Mutex<Basket>>) {
    let mut ticker = interval(Duration::from_secs(1));
    let mut eh_sender = retry_and_report_error(async move || {
        RabbitSender::<eh_request::Request>::new(
            "amqp://guest:guest@rabbitmq:5672",
            "ExpiredHolders",
        )
        .await
    })
    .await;

    loop {
        ticker.tick().await;
        let current_timestamp = current_timestamp();
        let basket = basket.lock().await;
        let expired_holders = basket.get_expired_holders(current_timestamp);
        drop(basket);

        if expired_holders.expired_holders.is_empty() {
            continue;
        }

        if let Err(err) = eh_sender.send_message(expired_holders).await {
            println!("ERROR SENDING: {}", err);
        }
    }
}
