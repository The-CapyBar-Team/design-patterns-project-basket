use lapin::{options::*, types::FieldTable, BasicProperties, Connection, Channel};
use prost::Message;
use basket_communication::add_to_queue_request;
use futures::stream::StreamExt;
use rand::Rng;

fn next_message_id() -> u32 {
    let mut rng = rand::rng();
    rng.random()
}

async fn send_message(channel: &Channel, message: add_to_queue_request::Message) -> anyhow::Result<()> {
    let payload = message.encode_to_vec();
    let queue_name = "request_queue";

    // println!("Trying to send message");

    let confirm = channel.basic_publish(
        "",
        queue_name,
        BasicPublishOptions::default(),
        &payload,
        BasicProperties::default(),
    )
    .await?;

    // println!("Message is sent. Trying to confirm.");

    confirm.await?;

    // println!("Message confirmed.");

    Ok(())
}

async fn listen_for_response(channel: &Channel) -> anyhow::Result<()> {
    let _queue = channel.queue_declare("reply_to_queue", QueueDeclareOptions::default(), FieldTable::default()).await?;

    // println!("Trying to do basic_consume");

    let mut consumer = channel.basic_consume(
        "reply_to_queue",
        "client_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default(),
    )
    .await?;

    // println!("basic_consume done. Trying delivering.");

    while let Some(delivery) = consumer.next().await {
        // println!("Delivery spotted");
        match delivery {
            Ok(delivery) => {
                let message = add_to_queue_request::Message::decode(&delivery.data[..])?;
                channel.basic_ack(delivery.delivery_tag, BasicAckOptions::default()).await?;
                println!("Received response from server #{}: {}", message.id, message.content);
                break;
            }
            Err(e) => eprintln!("Error while consuming response: {}", e),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let connection = Connection::connect("amqp://admin:admin@rabbit:5672/%2f", lapin::ConnectionProperties::default()).await?;
    // let connection = Connection::connect("amqp://localhost", lapin::ConnectionProperties::default()).await?;

    // println!("Trying to create a channel");
    let channel = connection.create_channel().await?;
    // println!("Channel created");

    let message = add_to_queue_request::Message {
        content: "Hello, server!".to_string(),
        id: next_message_id(),
    };

    // loop {
    //     println!("Dodododo");
    //     std::thread::sleep(std::time::Duration::from_secs(3));
    // }
// println!("Entering the loop");
    loop {
        send_message(&channel, message.clone()).await?;
        listen_for_response(&channel).await?;
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    // Ok(())
}
