use crate::balancing_table::balancing_table::ResponseSender;
use crate::basket_pool::basket_pool::wait_until_basket_is_ready;
use crate::basket_set::traits::*;
use crate::listeners::*;
use crate::requests::BasicRequestSender;
use balancing_table::balancing_table::BalancingTable;
use basket_communication::external::ProductStockList;
use basket_communication::rabbit::error::RabbitError;
use basket_communication::rabbit::listener::RabbitListener;
use basket_communication::{
    basket_balancer::basket_balancer_server::BasketBalancerServer, rabbit::sender::RabbitSender,
};
use basket_pool::basket_pool::{BasketBalancerGrpcServer, ProtectedBasketPool};
use basket_set::vec_balancing_set::VecBalancingSet;
use std::cell::RefCell;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;
use utilities::retry_and_report_error;

mod balancing_table;
mod basket_pool;
mod basket_set;
mod listeners;
mod requests;
mod responses;
mod types;
mod utilities;

type BasketBalancingSetImplementation = VecBalancingSet;

async fn create_response_sender(rabbit_connection_string: String) -> Arc<Mutex<ResponseSender>> {
    let queue_position_update_message_sender = {
        let connection_string = rabbit_connection_string.clone();
        retry_and_report_error(async move || {
            RabbitSender::new(&connection_string, "QueuePositionUpdateMessage").await
        })
        .await
    };

    let lost_product_sender = {
        let connection_string = rabbit_connection_string.clone();
        retry_and_report_error(async move || {
            RabbitSender::new(&connection_string, "LostProduct").await
        })
        .await
    };

    let product_status_update_sender = {
        let connection_string = rabbit_connection_string.clone();
        retry_and_report_error(async move || {
            RabbitSender::new(&connection_string, "ProductStatusUpdate").await
        })
        .await
    };

    let decrease_stock_request_sender = {
        let connection_string = rabbit_connection_string.clone();
        retry_and_report_error(async move || {
            RabbitSender::new(&connection_string, "DecreaseStockRequest").await
        })
        .await
    };

    Arc::new(Mutex::new(ResponseSender {
        queue_position_update_message_sender,
        lost_product_sender,
        product_status_update_sender,
        decrease_stock_request_sender,
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rabbit_connection_string = "amqp://guest:guest@rabbitmq:5672".to_owned();

    let basket_pool = Arc::new(ProtectedBasketPool::<BasketBalancingSetImplementation>::default());
    let request_sender = Arc::new(Mutex::new(BasicRequestSender::new(basket_pool.clone())));
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

    wait_until_basket_is_ready();

    let response_sender = create_response_sender(rabbit_connection_string.clone()).await;
    let mut balancing_table = Arc::new(Mutex::new(BalancingTable::new(
        request_sender.clone(),
        response_sender,
        basket_pool.clone(),
    )));

    let add_to_cart_request_listener = tokio::spawn({
        let connection_string = rabbit_connection_string.clone();
        let balancing_table = balancing_table.clone();

        async move {
            add_to_cart_request::listener(connection_string, balancing_table).await;
        }
    });

    let buy_product_request_listener = tokio::spawn({
        let connection_string = rabbit_connection_string.clone();
        let balancing_table = balancing_table.clone();

        async move {
            buy_product_request::listener(connection_string, balancing_table).await;
        }
    });

    let product_status_request_listener = tokio::spawn({
        let connection_string = rabbit_connection_string.clone();
        let balancing_table = balancing_table.clone();

        async move {
            product_status_request::listener(connection_string, balancing_table).await;
        }
    });

    let product_stock_info_listener = tokio::spawn({
        let connection_string = rabbit_connection_string.clone();
        let balancing_table = balancing_table.clone();

        async move {
            product_stock_info::listener(connection_string, balancing_table).await;
        }
    });

    let product_stock_list_listener = tokio::spawn({
        let connection_string = rabbit_connection_string.clone();
        let balancing_table = balancing_table.clone();

        async move {
            product_stock_list::listener(connection_string, balancing_table).await;
        }
    });

    let remove_from_cart_request_listener = tokio::spawn({
        let connection_string = rabbit_connection_string.clone();
        let balancing_table = balancing_table.clone();

        async move {
            remove_from_cart_request::listener(connection_string, balancing_table).await;
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
