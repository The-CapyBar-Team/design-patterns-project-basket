use basket::Basket;
use basket_communication::basket_balancer::basket_balancer_client::BasketBalancerClient;
use basket_communication::basket_balancer::cb_request::Request as CbRequest;
use basket_communication::basket_service::basket_service_server::BasketServiceServer;
use callbacks::*;
use tonic::transport::Server;
use tonic::Request;

mod basket;
mod callbacks;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::thread::sleep(std::time::Duration::from_secs(5));
    let server_handle = tokio::spawn(async move {
        let basket_context = BasketContext::new(Basket::new(|_, _, _| {}));
        let addr = "[::]:50051".parse().expect("Invalid address");

        println!("Server listening on {}", addr);

        Server::builder()
            .add_service(BasketServiceServer::new(basket_context))
            .serve(addr)
            .await
            .expect("Server error");
    });

    let cb_client_handle = tokio::spawn(async move {
        let mut client = loop {
            if let Ok(success) = BasketBalancerClient::connect("http://basket_balancer:50051").await {
                break success;
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        };


        let request = Request::new(CbRequest {
            hostname: "basket_service".to_owned(),
            port: 50051,
        });

        let response = client.perform_cb(request).await.unwrap().into_inner();

        println!(
            "!!!!!RESPONSE: error_message = '{}', status = {}",
            response.error_message, response.status
        );
    });

    server_handle.await?;
    cb_client_handle.await?;

    Ok(())
}
