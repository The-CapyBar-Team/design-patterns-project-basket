use basket_communication::external;
use basket_communication::rabbit::sender::RabbitSender;

pub(crate) struct ResponseSender {
    queue_position_update_message_sender: RabbitSender<external::QueuePositionUpdateMessage>,
    decrease_stock_request_sender: RabbitSender<external::DecreaseStockRequest>,
}

impl ResponseSender {
    #[inline(always)]
    pub(crate) fn new(
        queue_position_update_message_sender: RabbitSender<external::QueuePositionUpdateMessage>,
        decrease_stock_request_sender: RabbitSender<external::DecreaseStockRequest>,
    ) -> Self {
        Self {
            queue_position_update_message_sender,
            decrease_stock_request_sender,
        }
    }

    #[inline(always)]
    pub(crate) async fn send_queue_position_update(
        &mut self,
        message: external::QueuePositionUpdateMessage,
    ) {
        let _ = self
            .queue_position_update_message_sender
            .send_message(message)
            .await
            .map_err(|err| log_error("QueuePositionUpdateMessage", err));
    }

    #[inline(always)]
    pub(crate) async fn send_decrease_stock_request(
        &mut self,
        message: external::DecreaseStockRequest,
    ) {
        let _ = self
            .decrease_stock_request_sender
            .send_message(message)
            .await
            .map_err(|err| log_error("DecreaseStockRequest", err));
    }
}

#[inline(always)]
fn log_error(sender_name: &str, error: impl std::error::Error) -> () {
    println!("!<>! Sender Error | {} | {}", sender_name, error,);
}
