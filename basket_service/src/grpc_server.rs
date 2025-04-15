use tonic::{Request, Response, Status};

use basket_communication::basket_service_processor::{EnqueueRequestData, EnqueueRequestResponse};
use basket_communication::basket_service_processor::basket_processor_server::BasketProcessor;

#[derive(Default, Clone)]
pub struct GrpcBasketServiceServer {}

#[tonic::async_trait]
impl BasketProcessor for GrpcBasketServiceServer {
    async fn process_enqueue_request(&self, request_data: Request<EnqueueRequestData>) -> Result<Response<EnqueueRequestResponse>, Status> {
        let reply = EnqueueRequestResponse {
            message: "responce from process_enqueue_request from basket_service".to_owned()
        };

        println!("# server | request_data: {}", request_data.get_ref().name);

        Ok(Response::new(reply))
    }
}
