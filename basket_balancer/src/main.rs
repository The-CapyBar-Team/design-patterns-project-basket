use crate::basket_set::traits::*;
use basket_communication::basket_balancer::basket_balancer_server::BasketBalancerServer;
use basket_pool::basket_pool::{BasketPool, ProtectedBasketPool};
use basket_set::vec_balancing_set::VecBalancingSet;
use tonic::transport::Server;

mod balancing_table;
mod basket_pool;
mod basket_set;
mod requests;
mod types;

type BasketBalancingSetImplementation = VecBalancingSet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let basket_pool = ProtectedBasketPool::<BasketBalancingSetImplementation>::default();
    let addr = "[::]:50051".parse()?;

    println!("BasketBalancer is listening on {}", addr);

    Server::builder()
        .add_service(BasketBalancerServer::new(basket_pool))
        .serve(addr)
        .await?;

    Ok(())
}
