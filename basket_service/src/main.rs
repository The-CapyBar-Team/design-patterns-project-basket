use basket::Basket;
use basket_communication::basket_service::basket_service_server::BasketServiceServer;
use callbacks::*;
use tonic::transport::Server;

mod basket;
mod callbacks;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let basket_context = BasketContext::new(Basket::new(|_, _, _| {}));
    let addr = "[::]:50051".parse()?;

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(BasketServiceServer::new(basket_context))
        .serve(addr)
        .await?;

    Ok(())
}
