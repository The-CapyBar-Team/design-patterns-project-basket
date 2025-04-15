use tonic::{Request, Response, Status};
use basket_communication::basket_balancer_notifier::{NotificationData, NotificationResponce};
use basket_communication::basket_balancer_notifier::balancer_notifier_server::BalancerNotifier;

#[derive(Default)]
pub struct GrpcBasketBalancerServer {}

#[tonic::async_trait]
impl BalancerNotifier for GrpcBasketBalancerServer {
    async fn out_of_stock_notify(&self, notification_data: Request<NotificationData>) -> Result<Response<NotificationResponce>, Status> {
        println!("# server | notification_data: {}", notification_data.get_ref().name);
        let reply = NotificationResponce {
            message: "MESSAGE_FROM_BASKET_BALANCER".to_owned()
        };

        Ok(Response::new(reply))
    }
}