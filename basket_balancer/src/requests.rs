use crate::types::BasketId;
use basket_communication::types::{ProductId, UserId};
use basket_communication::update_product_stock_request;

// UPS = "Update Product's Stock"
pub(crate) fn perform_ups_request(
    basket_id: BasketId,
    request_args: update_product_stock_request::Request,
) {
    println!(
        "Sending stock (={}) of product #{} update to basket #{}",
        request_args.product_stock, request_args.product_id, basket_id
    );
}

// HP = "Hold Product"
pub(crate) fn delegate_hp_request_to_backet(
    basket_id: BasketId,
    product_id: ProductId,
    user_id: UserId,
) -> bool {
    println!(
        "Delegating to basket #{} holding product: product_id = {}, user_id = {}",
        basket_id, product_id, user_id
    );

    true
}

// AP = "Await Request"
pub(crate) fn delegate_ap_request_to_basket(
    basket_id: BasketId,
    product_id: ProductId,
    user_id: UserId,
) {
    println!(
        "Delegating to basket #{} awaiting product: product_id = {}, user_id = {}",
        basket_id, product_id, user_id
    );
}
