use std::time::Duration;
use tonic::transport::Server;
use basket_communication::basket_balancer_notifier::NotificationData;
use basket_communication::basket_balancer_notifier::balancer_notifier_client::BalancerNotifierClient;
use basket_communication::basket_service_processor::basket_processor_server::BasketProcessorServer;
use grpc_server::GrpcBasketServiceServer;

mod grpc_server;

async fn do_client() {
    let url = "http://basket_balancer:50051";
    let mut client = loop {
        match BalancerNotifierClient::connect(url).await {
            Ok(client) => {
                break client
            },
            Err(error) => {
                println!("# client | Connectivity error: {}", error);
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    };

    loop {
        let args = NotificationData {
            name: "DATA_FROM_BASKET_SERVICE".to_owned(),
        };
        let request = tonic::Request::new(args);

        match client.out_of_stock_notify(request).await {
            Ok(responce) => println!("# client | Received Responce: {}", responce.get_ref().message),
            Err(error) => println!("# client | Error: {}", error),
        };

        std::thread::sleep(Duration::from_secs(2));
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client_handle = tokio::spawn(async {
        do_client().await
    });

    let server_handle = tokio::spawn(async {
        let addr = "[::]:50052".parse().unwrap();
        let basket_service = GrpcBasketServiceServer::default();

        while let Err(error) = Server::builder()
            .add_service(BasketProcessorServer::new(basket_service.clone()))
            .serve(addr)
            .await
        {
            println!("# server | {}", error);
        }

        println!("BasketBalancer listening on {}", addr);
    });

    client_handle.await?;
    server_handle.await?;

    Ok(())
}
