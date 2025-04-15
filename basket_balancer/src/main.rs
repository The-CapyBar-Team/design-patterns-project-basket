use tonic::transport::Server;
use basket_communication::basket_balancer_notifier::balancer_notifier_server::BalancerNotifierServer;
use grpc_server::GrpcBasketBalancerServer;

mod grpc_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:50051".parse()?;
    let basket_balancer = GrpcBasketBalancerServer::default();

    println!("BasketBalancer listening on {}", addr);

    Server::builder()
        .add_service(BalancerNotifierServer::new(basket_balancer))
        .serve(addr)
        .await?;

    Ok(())
}
