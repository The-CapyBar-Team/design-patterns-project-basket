use super::error::BalancerError;
use crate::basket_pool::basket_pool::BasketPool;
use crate::requests;
use crate::types::BasketId;
use basket_communication::basket_service::hp_request::{
    Request as hp_or_ap_request, Response as hp_or_ap_response,
};
use basket_communication::basket_service::ups_request::Request as ups_request;
use basket_communication::types::{ProductId, ProductStock, QueuePosition, UserId};
use std::cell::RefCell;
use std::collections::HashMap;

const NUMBER_OF_BASKETS: BasketId = 4;

#[derive(Default)]
struct ProductBalancingInfo<BasketBalancer>
where
    BasketBalancer:
        Default + crate::basket_set::traits::BasketSet + crate::basket_set::traits::BasketBalancer,
{
    basket_balancer: BasketBalancer,
    queue_size: QueuePosition,
}

pub(crate) struct BalancingTable<'l, RequestSender, BasketBalancer>
where
    RequestSender: Default + requests::RequestSender,
    BasketBalancer:
        Default + crate::basket_set::traits::BasketSet + crate::basket_set::traits::BasketBalancer,
{
    products_to_balancing_info: RefCell<HashMap<ProductId, ProductBalancingInfo<BasketBalancer>>>, // TODO: think of using Vec instead of HashMap as a map
    request_sender: RequestSender,
    basket_pool: &'l BasketPool<BasketBalancer>,
}

impl<'l, RequestSender, BasketBalancer> BalancingTable<'l, RequestSender, BasketBalancer>
where
    RequestSender: Default + requests::RequestSender,
    BasketBalancer:
        Default + crate::basket_set::traits::BasketSet + crate::basket_set::traits::BasketBalancer,
{
    #[inline(always)]
    pub(crate) fn new(basket_pool: &'l BasketPool<BasketBalancer>) -> Self {
        Self {
            products_to_balancing_info: Default::default(),
            request_sender: Default::default(),
            basket_pool,
        }
    }

    // TODO: implement resilience strategy in case of some baskets getting dead
    #[inline(always)]
    pub(crate) fn update_product_stock(&mut self, product_id: ProductId, stock: ProductStock) {
        let stock_distribution = distribute_stock(stock);
        let basket_balancer = {
            let mut basket_balancer = BasketBalancer::default();

            for (basket_id, stock_piece) in (0..NUMBER_OF_BASKETS).zip(stock_distribution) {
                self.request_sender.perform_ups_request(
                    basket_id,
                    ups_request {
                        product_id,
                        product_stock: stock_piece,
                    },
                );
                basket_balancer.add_basket_id(basket_id);
            }

            basket_balancer
        };

        if let Some(balancing_info) = self
            .products_to_balancing_info
            .borrow_mut()
            .get_mut(&product_id)
        {
            balancing_info.basket_balancer = basket_balancer;
        } else {
            let replaced_balancing_info = self.products_to_balancing_info.borrow_mut().insert(
                product_id,
                ProductBalancingInfo {
                    basket_balancer,
                    queue_size: 0,
                },
            );

            debug_assert!(replaced_balancing_info.is_none());
        }
    }

    #[inline(always)]
    pub(crate) fn add_product_to_basket(
        &mut self,
        product_id: ProductId,
        user_id: UserId,
    ) -> Result<(), BalancerError> {
        let mut binding = self.products_to_balancing_info.borrow_mut();
        let balancing_info = binding
            .get_mut(&product_id)
            .ok_or(BalancerError::ProductNotFound(product_id, user_id))?;

        while let Some(next_basket_id) = {
            self.synchronize_with_global_basket_set(balancing_info);
            balancing_info.basket_balancer.choose_next_basket()
        } {
            if self
                .request_sender
                .perform_hp_request(next_basket_id, product_id, user_id)
            {
                break;
            }

            balancing_info
                .basket_balancer
                .remove_basket_id(next_basket_id);
        }

        if balancing_info.basket_balancer.is_empty() {
            let next_basket_id = self
                .basket_pool
                .actual_basket_set()
                .choose_next_basket()
                .expect("We must have at least one basket");
            self.request_sender
                .perform_ap_request(next_basket_id, product_id, user_id);
        }

        Ok(())
    }

    #[inline(always)]
    fn synchronize_with_global_basket_set(
        &self,
        balancing_info: &mut ProductBalancingInfo<BasketBalancer>,
    ) {
        balancing_info
            .basket_balancer
            .intersect(&self.basket_pool.actual_basket_set());
    }
}

// TODO: use SmallVec here
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

#[cfg(test)]
mod tests {
    use super::*;

    // const PRODUCT_ID: ProductId = 13124;
    // const INITIAL_STOCK: ProductStock = 1012382;
    // const MIN_DISTRIBUTION_VALUE: ProductStock = INITIAL_STOCK / NUMBER_OF_BASKETS as ProductStock;
    // const MAX_DISTRIBUTION_VALUE: ProductStock = MIN_DISTRIBUTION_VALUE + 1;

    // #[derive(Default)]
    // struct MockPerProductData {
    //     stock: ProductStock,
    // }

    // struct MockRequestSender {
    //     baskets: Vec<(BasketId, HashMap<ProductId, MockPerProductData>)>,
    // }

    // impl Default for MockRequestSender {
    //     fn default() -> Self {
    //         Self {
    //             baskets: (0..NUMBER_OF_BASKETS)
    //                 .map(|basket_id| (basket_id, Default::default()))
    //                 .collect(),
    //         }
    //     }
    // }

    // impl MockRequestSender {
    //     fn resolve_basket_map(
    //         &mut self,
    //         basket_id: BasketId,
    //     ) -> Option<&mut HashMap<ProductId, MockPerProductData>> {
    //         self.baskets
    //             .iter_mut()
    //             .find(|(found_basket_id, _)| *found_basket_id == basket_id)
    //             .map(|(_, basket_map)| basket_map)
    //     }
    // }

    // impl requests::RequestSender for MockRequestSender {
    //     fn perform_ups_request(
    //         &mut self,
    //         basket_id: BasketId,
    //         request_args: basket_communication::ups_request::Request,
    //     ) {
    //         let basket_data = self.resolve_basket_map(basket_id).unwrap();
    //         basket_data.insert(
    //             request_args.product_id,
    //             MockPerProductData {
    //                 stock: request_args.product_stock,
    //             },
    //         );
    //     }

    //     fn perform_hp_request(
    //         &mut self,
    //         basket_id: BasketId,
    //         product_id: ProductId,
    //         user_id: UserId,
    //     ) -> bool {
    //         todo!()
    //     }

    //     fn perform_ap_request(
    //         &mut self,
    //         basket_id: BasketId,
    //         product_id: ProductId,
    //         user_id: UserId,
    //     ) {
    //         todo!()
    //     }
    // }

    #[test]
    fn basic_distribution() {
        assert_eq!(&distribute_stock(1), &[1, 0, 0, 0]);
        assert_eq!(&distribute_stock(2), &[1, 1, 0, 0]);
        assert_eq!(&distribute_stock(3), &[1, 1, 1, 0]);
        assert_eq!(&distribute_stock(4), &[1, 1, 1, 1]);
        assert_eq!(&distribute_stock(5), &[2, 1, 1, 1]);
        assert_eq!(&distribute_stock(6), &[2, 2, 1, 1]);
    }

    // #[test]
    // fn balancing_table_ups() {
    //     let mut balancing_table = BalancingTable::<MockRequestSender>::default();
    //     balancing_table.update_product_stock(PRODUCT_ID, INITIAL_STOCK);

    //     for (_, basket_map) in balancing_table.request_sender.baskets {
    //         let stock = basket_map
    //             .get(&PRODUCT_ID)
    //             .map(|product_info| product_info.stock)
    //             .unwrap();
    //         assert!(stock >= MIN_DISTRIBUTION_VALUE && stock <= MAX_DISTRIBUTION_VALUE);
    //     }
    // }
}
