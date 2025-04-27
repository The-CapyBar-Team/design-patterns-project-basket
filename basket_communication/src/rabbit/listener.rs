use futures::stream::StreamExt;
use lapin::{Channel, Connection, Consumer, message::Delivery, options::*, types::FieldTable};
use std::marker::PhantomData;

use super::error::RabbitError;

pub struct RabbitListener<MessageType>
where
    MessageType: Default + prost::Message,
{
    _connection: Connection,
    channel: Channel,
    consumer: Consumer,
    _message_phantom: PhantomData<MessageType>,
}

impl<MessageType> RabbitListener<MessageType>
where
    MessageType: Default + prost::Message,
{
    #[inline(always)]
    pub async fn new(connection_string: &str, queue_name: &str) -> Result<Self, RabbitError> {
        let connection =
            Connection::connect(connection_string, lapin::ConnectionProperties::default())
                .await
                .map_err(|err| {
                    RabbitError::ConnectionFailure(connection_string.to_owned(), err.to_string())
                })?;

        let channel = connection
            .create_channel()
            .await
            .map_err(|err| RabbitError::ChannelCreationFailure(err.to_string()))?;

        let _ = channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|err| {
                RabbitError::QueueDeclarationFailure(queue_name.to_string(), err.to_string())
            })?;

        let consumer = channel
            .basic_consume(
                queue_name,
                "",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|err| {
                RabbitError::ConsumerCreationFailure(queue_name.to_string(), err.to_string())
            })?;

        Ok(Self {
            _connection: connection,
            channel,
            consumer,
            _message_phantom: Default::default(),
        })
    }

    #[inline(always)]
    pub async fn listen_to_messages(&mut self, listener: impl AsyncFn(MessageType)) {
        while let Some(delivery) = self.consumer.next().await {
            if let Err(error) = self.process_delivery(delivery, &listener).await {
                eprintln!("!<>! Error while listening to rabbit: {:?}", error);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }

    #[inline(always)]
    async fn process_delivery(
        &mut self,
        delivery: Result<Delivery, lapin::Error>,
        listener: &impl AsyncFn(MessageType),
    ) -> Result<(), String> {
        match delivery {
            Ok(delivery) => {
                let message =
                    MessageType::decode(&delivery.data[..]).map_err(|err| err.to_string())?;
                listener(message).await;

                self.channel
                    .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                    .await
                    .map_err(|err| err.to_string())?;
            }
            Err(error) => Err(error.to_string())?,
        };

        Ok(())
    }
}
