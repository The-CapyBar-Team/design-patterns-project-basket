use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RabbitError {
    #[error("Unable to connect to rabbitmq | connection string = '{0}', error message = '{1}'")]
    ConnectionFailure(String, String),

    #[error("Unable to create a channel of rabbitmq | error message = '{0}'")]
    ChannelCreationFailure(String),

    #[error("Unable to declare a queue of rabbitmq | queue_name = '{0}', error message = '{1}'")]
    QueueDeclarationFailure(String, String),

    #[error("Unable to create a consumer of rabbitmq | queue_name = '{0}', error message = '{1}'")]
    ConsumerCreationFailure(String, String),
}
