use super::error::BalancerError;
use super::response_sender::ResponseSender;
use crate::basket_pool::basket_pool::ProtectedBasketPool;
use crate::basket_set::MAX_BASKETS_COUNT;
use crate::requests::{self, GrpcFailure};
use crate::types::BasketId;
use basket_communication::basket_service::{ap_request, hp_request, sq_request};
use basket_communication::basket_service::{ru_request, ups_request};
use basket_communication::external;
use basket_communication::types::{ProductId, ProductStock, QueuePosition, UserId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
struct ProductBalancingInfo<BasketBalancer>
where
    BasketBalancer:
        Default + crate::basket_set::traits::BasketSet + crate::basket_set::traits::BasketBalancer,
{
    basket_balancer: BasketBalancer,
    queue_size: QueuePosition,
}

pub(crate) struct BalancingTable<RequestSender, BasketBalancer>
where
    RequestSender: requests::RequestSender,
    BasketBalancer:
        Default + crate::basket_set::traits::BasketSet + crate::basket_set::traits::BasketBalancer,
{
    products_to_balancing_info: Mutex<HashMap<ProductId, ProductBalancingInfo<BasketBalancer>>>, // TODO: think of using Vec instead of HashMap as a map
    request_sender: Arc<Mutex<RequestSender>>, //TODO: remove mutex
    response_sender: Arc<Mutex<ResponseSender>>,
    basket_pool: Arc<ProtectedBasketPool<BasketBalancer>>,
}

impl<'l, RequestSender, BasketBalancer> BalancingTable<RequestSender, BasketBalancer>
where
    RequestSender: requests::RequestSender,
    BasketBalancer:
        Default + crate::basket_set::traits::BasketSet + crate::basket_set::traits::BasketBalancer,
{
    #[inline(always)]
    pub(crate) fn new(
        request_sender: Arc<Mutex<RequestSender>>,
        response_sender: Arc<Mutex<ResponseSender>>,
        basket_pool: Arc<ProtectedBasketPool<BasketBalancer>>,
    ) -> Self {
        Self {
            products_to_balancing_info: Default::default(),
            response_sender,
            request_sender,
            basket_pool,
        }
    }

    // TODO: implement resilience strategy in case of some baskets getting dead
    #[inline(always)]
    pub(crate) async fn update_product_stock(
        &mut self,
        product_id: ProductId,
        stock: ProductStock,
    ) {
        let stock_distribution = distribute_stock(stock);
        let basket_balancer = {
            let mut basket_balancer = BasketBalancer::default();

            // TODO: rely on global balancer, not on 0..MAX_BASKETS_COUNT
            for (basket_id, stock_piece) in (0..MAX_BASKETS_COUNT).zip(stock_distribution) {
                // TODO: not ignore queue_shift
                let ups_request::Response { queue_shift } = self
                    .request_sender
                    .lock()
                    .await
                    .perform_ups_request(
                        basket_id,
                        ups_request::Request {
                            product_id,
                            product_stock_increase: stock_piece,
                        },
                    )
                    .await
                    .unwrap(); // TODO: remove this unwrap

                // We still send ups so that basket knows that the good exists
                // however the stock is 0, so we do not add it as available basket for the product.
                if stock_piece > 0 {
                    basket_balancer.add_basket_id(basket_id);
                }
            }

            basket_balancer
        };

        if let Some(balancing_info) = self
            .products_to_balancing_info
            .lock()
            .await
            .get_mut(&product_id)
        {
            balancing_info.basket_balancer = basket_balancer;
        } else {
            let replaced_balancing_info = self.products_to_balancing_info.lock().await.insert(
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
    pub(crate) async fn add_product_to_basket(
        &mut self,
        product_id: ProductId,
        user_id: UserId,
    ) -> Result<(), BalancerError> {
        let mut binding = self.products_to_balancing_info.lock().await;
        let balancing_info = binding
            .get_mut(&product_id)
            .ok_or(BalancerError::ProductNotFound(product_id, user_id.clone()))?;

        while let Some(next_basket_id) = {
            self.synchronize_with_global_basket_set(balancing_info)
                .await;
            balancing_info.basket_balancer.choose_next_basket()
        } {
            println!("LOOP | next_basket_id = {}", next_basket_id);

            let hp_response = self
                .request_sender
                .lock()
                .await
                .perform_hp_request(
                    next_basket_id,
                    hp_request::Request {
                        user_id: user_id.clone(),
                        product_id,
                    },
                )
                .await;

            match hp_response {
                Ok(hp_request::Success {
                    user_id,
                    product_id,
                }) => {
                    self.response_sender
                        .lock()
                        .await
                        .send_queue_position_update(external::QueuePositionUpdateMessage {
                            user_id,
                            update_message: Some(external::QueuePositionUpdate {
                                product_id,
                                queue_position: None,
                                acquisition_time: None,
                            }),
                        })
                        .await;

                    return Ok(());
                }

                Err(GrpcFailure::Custom(hp_request::Failure {
                    error_message,
                    status,
                })) => match hp_request::FailureStatus::from_i32(status) {
                    Some(hp_request::FailureStatus::HoldersQueueAlreadyFull) => {
                        balancing_info
                            .basket_balancer
                            .remove_basket_id(next_basket_id);
                    }
                    Some(hp_request::FailureStatus::ProductNotFound) => {
                        eprintln!(
                            "!<>! AddToCartRequest | ProductNotFound | {}",
                            error_message
                        );

                        return Ok(());
                    } // TODO: add handling of ProductNotFound
                    Some(hp_request::FailureStatus::UserAlreadyAdded) => {
                        eprintln!(
                            "!<>! AddToCartRequest | UserAlreadyAdded | {}",
                            error_message
                        );

                        return Ok(());
                    } // TODO: add handling of UserAlreadyAdded
                    None => {
                        eprintln!(
                            "!<>! AddToCartRequest | unknown status code of hp_request::FailureStatus: {}",
                            status
                        );

                        return Ok(());
                    }
                },

                Err(GrpcFailure::Internal) => {
                    eprintln!("!<>! AddToCartRequest | internal grpc error");
                } // TODO: add protection agains internal errors
            }
        }

        let next_basket_id = self
            .basket_pool
            .basket_pool
            .lock()
            .await
            .actual_basket_set()
            .choose_next_basket()
            .expect("We must have at least one basket");

        let response = self
            .request_sender
            .lock()
            .await
            .perform_ap_request(
                next_basket_id,
                ap_request::Request {
                    product_id,
                    user_id,
                    queue_position: balancing_info.queue_size,
                },
            )
            .await;

        match response {
            Ok(ap_request::Success {
                user_id,
                product_id,
                queue_position,
            }) => {
                self.response_sender
                    .lock()
                    .await
                    .send_queue_position_update(external::QueuePositionUpdateMessage {
                        user_id,
                        update_message: Some(external::QueuePositionUpdate {
                            product_id,
                            queue_position: Some(queue_position),
                            acquisition_time: Some(0),
                        }),
                    })
                    .await
            }

            Err(GrpcFailure::Custom(ap_request::Failure { status })) => {
                println!("!<>! Custom AP Error | status = {:?}", status);
            }

            Err(GrpcFailure::Internal) => {
                println!(
                    "!<>! Internal AP Error | basket with id = {} did not respond to ap_request",
                    next_basket_id
                );
            }
        }

        balancing_info.queue_size += 1;

        Ok(())
    }

    #[inline(always)]
    pub(crate) async fn remove_product_from_basket(
        &mut self,
        product_id: ProductId,
        user_id: UserId,
    ) {
        let present_basket_ids = self
            .basket_pool
            .basket_pool
            .lock()
            .await
            .actual_basket_set()
            .present_basket_ids();

        let mut ru_success: Option<(BasketId, ru_request::Success)> = None;

        let mut request_sender = self.request_sender.lock().await;

        // TODO: use select! here or some other thing to wait less for each removal
        for &basket_id in present_basket_ids.iter() {
            let response = request_sender
                .perform_ru_request(
                    basket_id,
                    ru_request::Request {
                        user_id: user_id.clone(),
                        product_id,
                    },
                )
                .await;

            match response {
                Ok(received_ru_success) => {
                    println!(
                        "debug | balancer | ru_request | user removed from basket_id = {}",
                        basket_id
                    );
                    ru_success = Some((basket_id, received_ru_success));
                }

                Err(GrpcFailure::Custom(ru_request::Failure { status })) => {
                    match ru_request::FailureStatus::from_i32(status) {
                        Some(ru_request::FailureStatus::UserNotFound) => {
                            println!(
                                "debug | balancer | ru_request | user not found for basket_id = {}",
                                basket_id
                            );
                        }
                        Some(ru_request::FailureStatus::ProductNotFound) => {
                            eprintln!("!<>! RemoveFromCartRequest | ProductNotFound");
                        }
                        Some(ru_request::FailureStatus::LoggedInternallyError) => {}
                        None => {
                            eprintln!(
                                "!<>! RemoveFromCartRequest | unknown status code of ru_request::FailureStatus: {}",
                                status
                            );
                        }
                    }
                }

                Err(GrpcFailure::Internal) => {
                    eprintln!(
                        "!<>! Internal RU Error | basket with id = {} did not respond to ru_request",
                        basket_id
                    );
                }
            }
        }

        if let Some((
            basket_remover_id,
            ru_request::Success {
                removed_user_id,
                removed_user_queue_position,
                queue_shifts,
            },
        )) = ru_success
        {
            let mut collected_queue_shifts = Vec::new();

            if let Some(removed_user_queue_position) = removed_user_queue_position {
                for basket_id in present_basket_ids {
                    if basket_id != basket_remover_id {
                        let sq_response = request_sender
                            .perform_sq_request(
                                basket_id,
                                sq_request::Request {
                                    product_id,
                                    max_removed_queue_position: removed_user_queue_position,
                                    shift: 1,
                                },
                            )
                            .await;

                        match sq_response {
                            Ok(sq_request::Success { queue_shifts }) => {
                                println!(
                                    "debug | balancer | sq_request | queue shifts fro basket (id = {}): {:?}",
                                    basket_id, queue_shifts
                                );
                                collected_queue_shifts.push(queue_shifts);
                            }

                            Err(GrpcFailure::Custom(sq_request::Failure { status })) => {
                                match sq_request::FailureStatus::from_i32(status) {
                                    Some(sq_request::FailureStatus::ProductNotFound) => {
                                        eprintln!("!<>! RemoveFromCartRequest | ProductNotFound");
                                    }
                                    None => {
                                        eprintln!(
                                            "!<>! RemoveFromCartRequest | unknown status code of sq_request::FailureStatus: {}",
                                            status
                                        );
                                    }
                                }
                            }

                            Err(GrpcFailure::Internal) => {
                                eprintln!(
                                    "!<>! Internal RU Error | basket with id = {} did not respond to ru_request",
                                    basket_id
                                );
                            }
                        }
                    }
                }
            } else {
                // No one to shift! No awaiter was removed!
            }

            let mut response_sender = self.response_sender.lock().await;

            for queue_shift in queue_shifts {
                response_sender
                    .send_queue_position_update(external::QueuePositionUpdateMessage {
                        user_id: queue_shift.user_id,
                        update_message: Some(external::QueuePositionUpdate {
                            product_id,
                            queue_position: queue_shift.new_queue_position,
                            acquisition_time: None,
                        }),
                    })
                    .await;
            }

            for queue_shifts in collected_queue_shifts {
                for queue_shift in queue_shifts {
                    response_sender
                        .send_queue_position_update(external::QueuePositionUpdateMessage {
                            user_id: queue_shift.user_id,
                            update_message: Some(external::QueuePositionUpdate {
                                product_id,
                                queue_position: queue_shift.new_queue_position,
                                acquisition_time: None,
                            }),
                        })
                        .await;
                }
            }
        } else {
            // User does not exist!
            eprintln!(
                "!<>! RemoveFromCartRequest | user has been found in no baskets: {}",
                user_id
            );
        }
    }

    #[inline(always)]
    pub(crate) async fn send_decrease_stock_request(
        &mut self,
        product_id: ProductId,
        user_id: UserId,
    ) {
        self.response_sender
            .lock()
            .await
            .send_decrease_stock_request(external::DecreaseStockRequest {
                user_id,
                product_id,
            })
            .await;
    }

    #[inline(always)]
    async fn synchronize_with_global_basket_set(
        &self,
        balancing_info: &mut ProductBalancingInfo<BasketBalancer>,
    ) {
        balancing_info.basket_balancer.intersect(
            &self
                .basket_pool
                .basket_pool
                .lock()
                .await
                .actual_basket_set(),
        );
    }
}

// TODO: use SmallVec here
#[inline(always)]
fn distribute_stock(stock: ProductStock) -> Vec<ProductStock> {
    let div = stock / (MAX_BASKETS_COUNT as ProductStock);
    let rem = stock % (MAX_BASKETS_COUNT as ProductStock);
    let mut distribution = Vec::new();

    for _ in 0..rem {
        distribution.push(div + 1);
    }

    for _ in rem..(MAX_BASKETS_COUNT as ProductStock) {
        distribution.push(div);
    }

    distribution
}

#[cfg(test)]
mod tests {
    use super::*;

    // const PRODUCT_ID: ProductId = 13124;
    // const INITIAL_STOCK: ProductStock = 1012382;
    // const MIN_DISTRIBUTION_VALUE: ProductStock = INITIAL_STOCK / MAX_BASKETS_COUNT as ProductStock;
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
    //             baskets: (0..MAX_BASKETS_COUNT)
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
