use super::error::BasketPoolError;
use crate::basket_pool::error::basket_pool_error_to_status;
use crate::basket_set::MAX_BASKETS_COUNT;
use crate::types::BasketId;
use basket_communication::basket_balancer::{basket_balancer_server, cb_request};
use basket_communication::basket_service::basket_service_client::BasketServiceClient;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::{Code, Request, Response, Status};

// pub(crate) trait BasketSerBounds: Default + crate::basket_set::traits::BasketSet
// // + std::marker::Sync + std::marker::Send + 'static
// {
// }

#[derive(Default)]
pub(crate) struct BasketPool<BasketSet>
where
    BasketSet: Default + crate::basket_set::traits::BasketSet,
{
    basket_set: BasketSet,
    channels: Vec<(BasketId, BasketServiceClient<Channel>)>,
}

#[derive(Default)]
pub(crate) struct ProtectedBasketPool<BasketSet>
where
    BasketSet: Default + crate::basket_set::traits::BasketSet,
{
    basket_pool: Mutex<BasketPool<BasketSet>>,
}

impl<BasketSet> BasketPool<BasketSet>
where
    BasketSet: Default + crate::basket_set::traits::BasketSet,
{
    #[inline(always)]
    pub(crate) fn actual_basket_set(&self) -> &BasketSet {
        &self.basket_set
    }

    #[inline(always)]
    pub(crate) fn is_ready(&self) -> bool {
        self.basket_set.is_full()
    }

    #[inline(always)]
    pub(crate) async fn establish_new_channel(
        &mut self,
        basket_id: BasketId,
        uri: &str,
    ) -> Result<(), BasketPoolError> {
        if basket_id >= MAX_BASKETS_COUNT {
            return Err(BasketPoolError::InvalidBasketId(basket_id));
        }

        if self.channels.len() as BasketId >= MAX_BASKETS_COUNT {
            eprintln!("ERROR: exceeding max baskets count");
            return Err(BasketPoolError::ConnectionPoolIsFull(basket_id));
        }

        if self
            .channels
            .iter()
            .find(|(found_channel_id, _)| *found_channel_id == basket_id)
            .is_some()
        {
            eprintln!("ERROR: Already connected!");
            return Err(BasketPoolError::AlreadyConnected(basket_id));
        }

        let channel =
            BasketServiceClient::connect(uri.to_owned())
                .await
                .map_err(|internal_error| {
                    eprintln!("ERROR: INTERNAL: {}", internal_error);
                    BasketPoolError::InternalError(basket_id, Box::new(internal_error))
                })?;

        self.channels.push((basket_id, channel));
        self.basket_set.add_basket_id(basket_id);

        eprintln!(
            "SUCCESS: channels_count = {}, basket_set = {}",
            self.channels.len(),
            self.basket_set.dump()
        );

        Ok(())
    }
}

#[tonic::async_trait]
impl<BasketSet> basket_balancer_server::BasketBalancer for ProtectedBasketPool<BasketSet>
where
    BasketSet: Default + crate::basket_set::traits::BasketSet + Send + Sync + 'static,
{
    #[inline(always)]
    async fn perform_cb(
        &self,
        request: Request<cb_request::Request>,
    ) -> Result<Response<cb_request::Response>, Status> {
        let args = request.into_inner();
        println!("!!!!PERFORMING CB: ");
        let basket_id = 0;
        let uri = format!("http://{}:{}", args.hostname, args.port);

        eprintln!("!!!!!URI URI URI: {}", uri);

        let mut basket_pool = self.basket_pool.lock().await;
        basket_pool
            .establish_new_channel(basket_id, &uri)
            .await
            .map_err(|err| Status::new(Code::Internal, format!("{}", err)))?;

        if basket_pool.is_ready() {
            subscribe_to_rabbit_and_start_processing_messages();
        }

        Ok(Response::new(cb_request::Response {
            error_message: "Successfuly connected".to_owned(),
            status: cb_request::ConnectionStatus::ConnectionSuccess.into(),
        }))
    }
}

fn subscribe_to_rabbit_and_start_processing_messages() {
    println!("Subscribing to rabbit and starting to handle requests.");
}
