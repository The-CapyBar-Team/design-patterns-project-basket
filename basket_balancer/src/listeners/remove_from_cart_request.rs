use crate::basket_pool::basket_pool::wait_until_basket_is_ready;
use crate::utilities::retry_and_report_error;
use basket_communication::external::RemoveFromCartRequest;
use basket_communication::rabbit::listener::RabbitListener;

#[inline(always)]
pub(crate) async fn listener(rabbit_connection_string: String) {
    let mut ups_rabbit_listener = retry_and_report_error(async move || {
        RabbitListener::<RemoveFromCartRequest>::new(
            &rabbit_connection_string,
            "RemoveFromCartRequest",
        )
        .await
    })
    .await;

    wait_until_basket_is_ready().await;

    ups_rabbit_listener
        .listen_to_messages(|message| {
            println!("Rabbit | RemoveFromCartRequest: {:?}", message);
        })
        .await;
}
