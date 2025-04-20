use basket_communication::add_to_queue_request;
use futures::stream::StreamExt;
use lapin::{message::Delivery, options::*, types::FieldTable, Channel, Connection};
use prost::Message;

mod basket;
mod callbacks;

async fn handle_message(_channel: &Channel, delivery: Delivery) -> anyhow::Result<()> {
    let message = add_to_queue_request::Message::decode(&delivery.data[..])?;
    println!("Received message #{}: {}", message.id, message.content);

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let connection = Connection::connect(
        "amqp://admin:admin@rabbit:5672/%2f",
        lapin::ConnectionProperties::default(),
    )
    .await?;
    let channel = connection.create_channel().await?;

    let queue_name = "request_queue";
    let _queue = channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            queue_name,
            "server_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    println!("Server is running, waiting for messages...");

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                if let Err(e) = handle_message(&channel, delivery).await {
                    eprintln!("Failed to handle message: {}", e);
                }
            }
            Err(e) => eprintln!("Error while consuming message: {}", e),
        }
    }

    Ok(())
}
