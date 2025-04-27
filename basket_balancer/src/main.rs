use crate::basket_set::traits::*;
use crate::listeners::*;
use crate::requests::BasicRequestSender;
use balancing_table::balancing_table::BalancingTable;
use basket_communication::basket_balancer::basket_balancer_server::BasketBalancerServer;
use basket_communication::external::ProductStockList;
use basket_communication::rabbit::error::RabbitError;
use basket_communication::rabbit::rabbit::RabbitListener;
use basket_pool::basket_pool::{BasketBalancerGrpcServer, ProtectedBasketPool};
use basket_set::vec_balancing_set::VecBalancingSet;
use std::sync::Arc;
use tonic::transport::Server;
use tokio::sync::Mutex;

mod balancing_table;
mod basket_pool;
mod basket_set;
mod listeners;
mod requests;
mod responses;
mod types;
mod utilities;

type BasketBalancingSetImplementation = VecBalancingSet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rabbit_connection_string = "amqp://guest:guest@rabbitmq:5672".to_owned();
    let basket_pool = Arc::new(ProtectedBasketPool::<BasketBalancingSetImplementation>::default());
    let request_sender = Arc::new(Mutex::new(BasicRequestSender::new(basket_pool.clone())));
    let mut balancing_table = BalancingTable::new(request_sender.clone(), basket_pool.clone());
    let grpc_server = BasketBalancerGrpcServer::new(basket_pool.clone());

    let grpc_server_handle = tokio::spawn(async move {
        let addr = "[::]:50051".parse().unwrap();
        println!("BasketBalancer is listening on {}", addr);

        Server::builder()
            .add_service(BasketBalancerServer::new(grpc_server))
            .serve(addr)
            .await
            .expect("Internal GRPC-server error");
    });

    let add_to_cart_request_listener = tokio::spawn({
        let connection_string = rabbit_connection_string.clone();

        async move {
            add_to_cart_request::listener(connection_string).await;
        }
    });

    let buy_product_request_listener = tokio::spawn({
        let connection_string = rabbit_connection_string.clone();

        async move {
            buy_product_request::listener(connection_string).await;
        }
    });

    let product_status_request_listener = tokio::spawn({
        let connection_string = rabbit_connection_string.clone();

        async move {
            product_status_request::listener(connection_string).await;
        }
    });

    let product_stock_info_listener = tokio::spawn({
        let connection_string = rabbit_connection_string.clone();

        async move {
            product_stock_info::listener(connection_string).await;
        }
    });

    let product_stock_list_listener = tokio::spawn({
        let connection_string = rabbit_connection_string.clone();

        async move {
            product_stock_list::listener(connection_string).await;
        }
    });

    let remove_from_cart_request_listener = tokio::spawn({
        let connection_string = rabbit_connection_string.clone();

        async move {
            remove_from_cart_request::listener(connection_string).await;
        }
    });

    add_to_cart_request_listener.await?;
    buy_product_request_listener.await?;
    product_status_request_listener.await?;
    product_stock_info_listener.await?;
    remove_from_cart_request_listener.await?;
    product_stock_list_listener.await?;
    grpc_server_handle.await?;

    Ok(())
}

// let hp_request_generator = tokio::spawn(async move {
//     while !BASKET_IS_READY.load(Ordering::Acquire) {
//         std::thread::yield_now();
//     }

//     println!("BASKET IS READY: Starting sending hp-requests");

//     let product_id = 128;

//     balancing_table
//         .update_product_stock(
//             product_id,
//             (10 * crate::basket_set::MAX_BASKETS_COUNT).into(),
//         )
//         .await;
//     println!("ups is done");

//     for id in 0..usize::MAX {
//         let user_id = id.to_string();
//         if let Err(err) = balancing_table
//             .add_product_to_basket(product_id, user_id)
//             .await
//         {
//             println!("add_product_to_basket error: {}", err);
//         }

//         std::thread::sleep(std::time::Duration::from_secs(2));
//     }
// });
