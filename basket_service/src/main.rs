use basket_communication::add_to_queue_request;
use prost::Message;
use lapin::{options::*, types::FieldTable, BasicProperties, Connection, Channel, message::Delivery};
use futures::stream::StreamExt;

async fn handle_message(channel: &Channel, delivery: Delivery) -> anyhow::Result<()> {
    let message = add_to_queue_request::Message::decode(&delivery.data[..])?;

    println!("Received message #{}: {}", message.id, message.content);

    let response = add_to_queue_request::Message {
        content: format!("Echo: {}", message.content),
        id: message.id,
    };

    let payload = response.encode_to_vec();

    channel.basic_publish(
        "",
        "reply_to_queue",
        BasicPublishOptions::default(),
        &payload,
        BasicProperties::default(),
    )
    .await?
    .await?;

    channel.basic_ack(delivery.delivery_tag, BasicAckOptions::default()).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let connection = Connection::connect("amqp://admin:admin@rabbit:5672/%2f", lapin::ConnectionProperties::default()).await?;
    // let connection = Connection::connect("amqp://localhost", lapin::ConnectionProperties::default()).await?;
    // println!("Trying to create a channel");
    let channel = connection.create_channel().await?;
    // println!("Channel created");

    let queue_name = "request_queue";
    // println!("Trying to queue_declare");
    let _queue = channel.queue_declare(queue_name, QueueDeclareOptions::default(), FieldTable::default()).await?;
    // println!("queue_declare done. Trying to declare reply_to_queue");
    let _reply_to_queue = channel.queue_declare("reply_to_queue", QueueDeclareOptions::default(), FieldTable::default()).await?;
    // println!("reply_to_queue declared.");

    // loop {
    //     println!("Dodododo");
    //     std::thread::sleep(std::time::Duration::from_secs(3));
    // }

    let mut consumer = channel.basic_consume(
        queue_name,
        "server_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default(),
    ).await?;

    println!("Server is running, waiting for messages...");

    while let Some(delivery) = consumer.next().await {
        // println!("Here's the delivery!");
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
