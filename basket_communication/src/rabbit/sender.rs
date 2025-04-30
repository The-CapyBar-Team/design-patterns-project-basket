use lapin::{BasicProperties, Channel, Connection, options::*, types::FieldTable};
use std::marker::PhantomData;

use super::error::{RabbitError, RabbitMessageSendError};

pub struct RabbitSender<MessageType>
where
    MessageType: Default + prost::Message,
{
    _connection: Connection,
    channel: Channel,
    queue_name: String,
    _message_phantom: PhantomData<MessageType>,
}

impl<MessageType> RabbitSender<MessageType>
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

        Ok(Self {
            _connection: connection,
            channel,
            queue_name: queue_name.to_owned(),
            _message_phantom: Default::default(),
        })
    }

    #[inline(always)]
    pub async fn send_message(
        &mut self,
        message: MessageType,
    ) -> Result<(), RabbitMessageSendError> {
        // println!("debug | balancer | sending message: {:?}", message);

        let payload = message.encode_to_vec();

        self.channel
            .basic_publish(
                "",
                &self.queue_name,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await
            .map_err(|err| RabbitMessageSendError::PublishError(err.to_string()))?
            .await
            .map_err(|err| RabbitMessageSendError::ConfirmationError(err.to_string()))?;

        Ok(())
    }
}
