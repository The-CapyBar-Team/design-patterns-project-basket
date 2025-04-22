mod balancing_table;
mod basket_pool;
mod basket_set;
mod requests;
mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let connection = BasketServiceClient::connect(
    //     "http://design-patterns-project-basket-basket_service-1:50051",
    // )
    // .await?; // TODO: move this url to env
    Ok(())
}
