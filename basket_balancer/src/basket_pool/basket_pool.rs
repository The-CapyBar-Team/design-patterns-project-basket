use super::error::BasketPoolError;
use crate::basket_set::MAX_BASKETS_COUNT;
use crate::types::BasketId;
use basket_communication::basket_balancer::{basket_balancer_server, cb_request};
use basket_communication::basket_service::basket_service_client::BasketServiceClient;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

pub(crate) trait BasketSetBounds:
    Default + crate::basket_set::traits::BasketSet + std::marker::Sync + std::marker::Send + 'static
{
}

#[derive(Default)]
pub(crate) struct BasketPool<BasketSet>
where
    BasketSet: BasketSetBounds,
{
    basket_set: BasketSet,
    channels: Vec<(BasketId, BasketServiceClient<Channel>)>,
}

impl<BasketSet> BasketPool<BasketSet>
where
    BasketSet: BasketSetBounds,
{
    #[inline(always)]
    pub(crate) fn actual_basket_set(&self) -> &BasketSet {
        &self.basket_set
    }

    #[inline(always)]
    pub(crate) async fn establish_new_channel(
        &mut self,
        basket_id: BasketId,
        uri: &str,
    ) -> Result<(), BasketPoolError> {
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
}

#[tonic::async_trait]
impl<BasketSet> basket_balancer_server::BasketBalancer for BasketPool<BasketSet>
where
    BasketSet: BasketSetBounds,
{
    #[inline(always)]
    async fn perform_cb(
        &self,
        request: Request<cb_request::Request>,
    ) -> Result<Response<cb_request::Response>, Status> {
        todo!()
    }
}
