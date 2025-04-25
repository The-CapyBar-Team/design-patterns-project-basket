use crate::types::BasketId;
use basket_communication::basket_service::hp_request;
use basket_communication::basket_service::ups_request;
use basket_communication::types::{ProductId, UserId};

// UPS = "Update Product's Stock"
// HP = "Hold Product"
// AP = "Await Request"
pub(crate) trait RequestSender {
    fn perform_ups_request(&mut self, basket_id: BasketId, request_args: ups_request::Request);

    fn perform_hp_request(&mut self, basket_id: BasketId, request: hp_request::Request) -> bool;

    fn perform_ap_request(&mut self, basket_id: BasketId, product_id: ProductId, user_id: UserId);
}

#[derive(Default)]
pub(crate) struct BasicRequestSender;

impl RequestSender for BasicRequestSender {
    fn perform_ups_request(&mut self, basket_id: BasketId, request_args: ups_request::Request) {
        println!(
            "Sending stock (={}) of product #{} update to basket #{}",
            request_args.product_stock, request_args.product_id, basket_id
        );
    }

    fn perform_hp_request(&mut self, basket_id: BasketId, request: hp_request::Request) -> bool {
        true
    }

    fn perform_ap_request(&mut self, basket_id: BasketId, product_id: ProductId, user_id: UserId) {
        println!(
            "Delegating to basket #{} awaiting product: product_id = {}, user_id = {}",
            basket_id, product_id, user_id
        );
    }
}
