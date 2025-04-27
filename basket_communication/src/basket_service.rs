mod basket_service_proto {
    tonic::include_proto!("basket_service");
}

pub use crate::basket_service_requests::*;
pub use basket_service_proto::*;
