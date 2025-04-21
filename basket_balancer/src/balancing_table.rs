use crate::error::BalancerError;
use crate::requests::*;
use crate::types::BasketId;
use basket_communication::hold_or_await_product_request::{
    Request as hp_or_ap_request, Response as hp_or_ap_response,
};
use basket_communication::types::{ProductId, ProductStock, QueuePosition, UserId};
use basket_communication::update_product_stock_request::Request as ups_request;
use std::collections::HashMap;

const NUMBER_OF_BASKETS: BasketId = 4;

#[derive(Default)]
struct ProductBalancingInfo {
    available_baskets: Vec<BasketId>,
    queue_size: QueuePosition,
}

#[derive(Default)]
pub(crate) struct BalancingTable {
    products_to_balancing_info: HashMap<ProductId, ProductBalancingInfo>,
}

impl BalancingTable {
    // TODO: implement resilience strategy in case of some baskets getting dead
    #[inline(always)]
    pub(crate) fn on_product_stock_updated(&mut self, product_id: ProductId, stock: ProductStock) {
        let stock_distribution = distribute_stock(stock);
        let available_baskets = {
            let mut available_baskets = Vec::new();

            for basket_id in 0..NUMBER_OF_BASKETS {
                perform_ups_request(
                    basket_id,
                    ups_request {
                        product_id,
                        product_stock: stock,
                    },
                );
                available_baskets.push(basket_id);
            }

            available_baskets
        };

        if let Some(balancing_info) = self.products_to_balancing_info.get_mut(&product_id) {
            balancing_info.available_baskets = available_baskets;
        } else {
            let replaced_balancing_info = self.products_to_balancing_info.insert(
                product_id,
                ProductBalancingInfo {
                    available_baskets,
                    queue_size: 0,
                },
            );

            debug_assert!(replaced_balancing_info.is_none());
        }
    }

    #[inline(always)]
    pub(crate) fn on_add_product_to_basket_request(
        &mut self,
        product_id: ProductId,
        user_id: UserId,
    ) -> Result<(), BalancerError> {
        let balancing_info = self
            .products_to_balancing_info
            .get_mut(&product_id)
            .ok_or(BalancerError::ProductNotFound(product_id, user_id))?;

        while let Some(next_basket_id) = choose_next_basket(&balancing_info.available_baskets) {
            if delegate_hp_request_to_backet(next_basket_id, product_id, user_id) {
                break;
            }

            if let Some(bad_service) = balancing_info
                .available_baskets
                .iter()
                .position(|basket_id| *basket_id == next_basket_id)
            {
                balancing_info.available_baskets.remove(bad_service);
            }
        }

        if balancing_info.available_baskets.is_empty() {
            let next_basket_id =
                choose_next_basket(&all_baskets()).expect("We must have at least one basket");
            delegate_ap_request_to_basket(next_basket_id, product_id, user_id);
        }

        Ok(())
    }
}

#[inline(always)]
fn distribute_stock(stock: ProductStock) -> Vec<ProductStock> {
    let div = stock / (NUMBER_OF_BASKETS as ProductStock);
    let rem = stock % (NUMBER_OF_BASKETS as ProductStock);
    let mut distribution = Vec::new();

    for _ in 0..rem {
        distribution.push(div + 1);
    }

    for _ in rem..(NUMBER_OF_BASKETS as ProductStock) {
        distribution.push(div);
    }

    distribution
}

#[inline(always)]
fn choose_next_basket(available_baskets: &[BasketId]) -> Option<BasketId> {
    use rand::Rng;

    if available_baskets.is_empty() {
        None
    } else {
        let mut rng = rand::thread_rng();
        Some(rng.gen_range(0..available_baskets.len()) as u8)
    }
}

#[inline(always)]
fn all_baskets() -> [BasketId; NUMBER_OF_BASKETS as usize] {
    std::array::from_fn(|id| id as BasketId)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_distribution() {
        assert_eq!(&distribute_stock(1), &[1, 0, 0, 0]);
        assert_eq!(&distribute_stock(2), &[1, 1, 0, 0]);
        assert_eq!(&distribute_stock(3), &[1, 1, 1, 0]);
        assert_eq!(&distribute_stock(4), &[1, 1, 1, 1]);
        assert_eq!(&distribute_stock(5), &[2, 1, 1, 1]);
        assert_eq!(&distribute_stock(6), &[2, 2, 1, 1]);
    }
}
