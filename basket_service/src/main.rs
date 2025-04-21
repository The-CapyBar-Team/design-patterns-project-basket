use basket::Basket;
use basket_communication::ups_request::ups_service_server::UpsServiceServer;
use callbacks::*;
use tonic::transport::Server;

mod basket;
mod callbacks;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let basket_context = BasketContext::new(Basket::new(|_, _, _| {}));
    let addr = "[::1]:50051".parse()?;

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(UpsServiceServer::new(basket_context))
        .serve(addr)
        .await?;

    Ok(())
}
