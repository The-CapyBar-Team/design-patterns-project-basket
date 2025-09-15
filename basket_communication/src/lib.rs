pub mod basket_balancer;
pub mod basket_balancer_requests;
pub mod basket_service;
pub mod basket_service_requests;
pub mod external;
pub mod rabbit;
pub mod types;

pub async fn retry_and_report_error<F, R, E>(action: F) -> R
where
    F: AsyncFn() -> Result<R, E>,
    E: std::error::Error,
{
    loop {
        match action().await {
            Ok(result) => return result,
            Err(error) => println!("Retry Error: {}", error),
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
