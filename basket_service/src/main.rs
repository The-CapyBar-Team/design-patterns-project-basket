use basket::Basket;
use basket_communication::basket_balancer::basket_balancer_client::BasketBalancerClient;
use basket_communication::basket_balancer::cb_request::{self, Request as CbRequest};
use basket_communication::basket_balancer_requests::cb_request::ConnectionStatus;
use basket_communication::basket_service::basket_service_server::BasketServiceServer;
use callbacks::*;
use num::FromPrimitive;
use std::env;
use tonic::transport::Server;
use tonic::Request;

mod basket;
mod callbacks;
mod error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let basket_id: u32 = env::var("BASKET_ID")
        .expect("BASKET_ID not found.")
        .parse()?;
    let port: u32 = env::var("BASKET_SERVICE_PORT")
        .expect("BASKET_SERVICE_PORT not found.")
        .parse()?;
    let basket_service_uri = env::var("BASKET_SERVICE_URI").expect("BASKET_SERVICE_URI not found.");
    let basket_balancer_uri =
        env::var("BASKET_BALANCER_URI").expect("BASKET_BALANCER_URI not found.");

    let server_handle = tokio::spawn(async move {
        let basket_context = BasketContext::new(Basket::new(|_, _, _| {}));
        let addr = format!("[::]:{}", port).parse().expect("Invalid address");

        println!("Server listening on {}", addr);

        Server::builder()
            .add_service(BasketServiceServer::new(basket_context))
            .serve(addr)
            .await
            .expect("Server error");
    });

    std::thread::sleep(std::time::Duration::from_secs(5));

    let cb_client_handle = tokio::spawn(async move {
        let mut client = loop {
            if let Ok(success) = BasketBalancerClient::connect(basket_balancer_uri.clone()).await {
                break success;
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        };

        let response = loop {
            let request = Request::new(CbRequest {
                uri: basket_service_uri.clone(),
                basket_id,
            });

            match client.perform_cb(request).await {
                Ok(cb_response) => {
                    let response = cb_response.into_inner();

                    match (
                        ConnectionStatus::from_i32(response.status.clone())
                            .expect("Unexpected status."),
                        response.error_message.clone(),
                    ) {
                        (ConnectionStatus::ConnectionSuccess, _) => {
                            break response;
                        }
                        (_, Some(error_message)) => {
                            println!("Error connecting: {}", error_message);
                        }
                        _ => panic!("UNPREDICTABLE"),
                    };
                }
                Err(err) => {
                    println!("Error getting the response: {}", err);
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(3));
        };

        println!("Basket #{} connected to balancer.", basket_id);
    });

    server_handle.await?;
    cb_client_handle.await?;

    Ok(())
}
