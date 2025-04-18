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

    let confirm = channel.basic_publish(
        "",
        queue_name,
        BasicPublishOptions::default(),
        &payload,
        BasicProperties::default(),
    )
    .await?
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let connection = Connection::connect("amqp://admin:admin@rabbit:5672/%2f", lapin::ConnectionProperties::default()).await?;

    let channel = connection.create_channel().await?;

    let message = add_to_queue_request::Message {
        content: "Hello, server!".to_string(),
        id: next_message_id(),
    };

    loop {
        send_message(&channel, message.clone()).await?;
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
