use super::error::BasketPoolError;
use crate::basket_pool::error::basket_pool_error_to_status;
use crate::basket_set::MAX_BASKETS_COUNT;
use crate::types::BasketId;
use basket_communication::basket_balancer::{basket_balancer_server, cb_request};
use basket_communication::basket_service::basket_service_client::BasketServiceClient;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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
    pub basket_pool: Mutex<BasketPool<BasketSet>>,
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
            return Err(BasketPoolError::ConnectionPoolIsFull(basket_id));
        }

        if self
            .channels
            .iter()
            .find(|(found_channel_id, _)| *found_channel_id == basket_id)
            .is_some()
        {
            return Err(BasketPoolError::AlreadyConnected(basket_id));
        }

        let channel =
            BasketServiceClient::connect(uri.to_owned())
                .await
                .map_err(|internal_error| {
                    BasketPoolError::InternalError(basket_id, Box::new(internal_error))
                })?;

        self.channels.push((basket_id, channel));
        self.basket_set.add_basket_id(basket_id);

        Ok(())
    }

    #[inline(always)]
    pub(crate) fn get_mut_basket_channel(
        &mut self,
        basket_id: BasketId,
    ) -> Option<&mut BasketServiceClient<Channel>> {
        self.channels
            .iter_mut()
            .find(|(found_channel_id, _)| *found_channel_id == basket_id)
            .map(|(_, channel)| channel)
    }
}

pub(crate) struct BasketBalancerGrpcServer<BasketSet>
where
    BasketSet: Default + crate::basket_set::traits::BasketSet + Send + Sync + 'static,
{
    basket_pool: Arc<ProtectedBasketPool<BasketSet>>,
}

impl<BasketSet> BasketBalancerGrpcServer<BasketSet>
where
    BasketSet: Default + crate::basket_set::traits::BasketSet + Send + Sync + 'static,
{
    #[inline(always)]
    pub(crate) fn new(basket_pool: Arc<ProtectedBasketPool<BasketSet>>) -> Self {
        Self { basket_pool }
    }
}

#[tonic::async_trait]
impl<BasketSet> basket_balancer_server::BasketBalancer for BasketBalancerGrpcServer<BasketSet>
where
    BasketSet: Default + crate::basket_set::traits::BasketSet + Send + Sync + 'static,
{
    #[inline(always)]
    async fn perform_cb(
        &self,
        request: Request<cb_request::Request>,
    ) -> Result<Response<cb_request::Response>, Status> {
        let cb_request::Request { uri, basket_id } = request.into_inner();

        let mut basket_pool = self.basket_pool.basket_pool.lock().await;
        let (error_message, status) = basket_pool
            .establish_new_channel(basket_id as BasketId, &uri)
            .await
            .map(|_| (None, cb_request::ConnectionStatus::ConnectionSuccess))
            .unwrap_or_else(|err| (Some(err.to_string()), basket_pool_error_to_status(err)));

        if basket_pool.is_ready() {
            subscribe_to_rabbit_and_start_processing_messages();
        }

        Ok(Response::new(cb_request::Response {
            error_message,
            status: status.into(),
        }))
    }
}

pub(crate) static BASKET_IS_READY: AtomicBool = AtomicBool::new(false);

fn subscribe_to_rabbit_and_start_processing_messages() {
    if let Ok(_) =
        BASKET_IS_READY.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
    {
        println!("SUCCESS: Subscribing to rabbit and starting to handle requests.");
    }
}
