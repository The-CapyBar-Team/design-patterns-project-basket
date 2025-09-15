use basket_communication::external::ProductStockList;
use futures::stream::StreamExt;
use lapin::{
    BasicProperties, Channel, Connection, message::Delivery, options::*, types::FieldTable,
};
use prost::Message;

async fn handle_message(channel: &Channel, delivery: Delivery) -> anyhow::Result<()> {
    let message = ProductStockList::decode(&delivery.data[..])?;
    println!("Received message {:?}", message);

    channel
        .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let connection = Connection::connect(
        "amqp://guest:guest@localhost:5672",
        lapin::ConnectionProperties::default(),
    )
    .await?;
    let channel = connection.create_channel().await?;
    let _ = channel
        .queue_declare(
            "ProductStockLists",
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            "ProductStockLists",
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                if let Err(e) = handle_message(&channel, delivery).await {
                    println!("Failed to handle message: {}", e);
                }
            }
            Err(e) => println!("Error while consuming message: {}", e),
        }
    }

    Ok(())
}

// curl -X POST http://localhost:8080/api/admin/add -H "Content-Type: application/json" -d '{ "id": 1, "name": "Smartphone", "description": "Latest model with 128GB storage", "price": 699.99, "stock": 100}'
