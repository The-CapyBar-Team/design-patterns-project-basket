use crate::error::{ApRequestError, HpRequestError, RuRequestError};
use basket_communication::basket_service_requests::sq_request::QueueShift;
use basket_communication::types::{ProductId, ProductStock, QueuePosition, UserId};
use std::collections::{HashMap, VecDeque};

// TODO: think of replacing VecDeque with HashMap

#[derive(Debug, Default, Clone)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
struct UserInfo {
    user_id: UserId,
    queue_position: Option<QueuePosition>,
}

struct ProductContext {
    product_holders: VecDeque<UserInfo>,
    product_awaiters: VecDeque<UserInfo>,
    product_stock: ProductStock,
}

pub(crate) struct HaRemovalResult {
    pub(crate) removed_user_id: UserId,
    pub(crate) removed_user_queue_position: Option<QueuePosition>,
}

pub(crate) struct Basket {
    product_to_context: HashMap<ProductId, ProductContext>,
    on_product_stock_changed:
        Box<dyn Fn(ProductId, ProductStock, ProductStock) + Send + Sync + 'static>,
}

impl Default for ProductContext {
    fn default() -> Self {
        Self {
            product_holders: create_contiguous_users_deque(),
            product_awaiters: create_contiguous_users_deque(),
            product_stock: Default::default(),
        }
    }
}

impl Basket {
    #[inline(always)]
    pub(crate) fn new(
        on_product_stock_changed: impl Fn(ProductId, ProductStock, ProductStock) + Send + Sync + 'static,
    ) -> Self {
        Self {
            product_to_context: Default::default(),
            on_product_stock_changed: Box::new(on_product_stock_changed),
        }
    }

    #[inline(always)]
    pub(crate) fn update_product_stock(
        &mut self,
        product_id: ProductId,
        product_stock_increase: ProductStock,
    ) {
        let old_stock = if let Some(entry) = self.product_to_context.get_mut(&product_id) {
            let old_stock = entry.product_stock;
            entry.product_stock += product_stock_increase;
            old_stock
        } else {
            let new_context = ProductContext {
                product_stock: product_stock_increase,
                ..Default::default()
            };

            self.product_to_context
                .insert(product_id, new_context)
                .map(|context| context.product_stock)
                .unwrap_or(0)
        };

        (self.on_product_stock_changed)(product_id, old_stock, old_stock + product_stock_increase);
    }

    #[inline(always)]
    pub(crate) fn add_product_holder(
        &mut self,
        product_id: ProductId,
        holder_id: UserId,
    ) -> Result<(), HpRequestError> {
        let product_context =
            self.product_to_context
                .get_mut(&product_id)
                .ok_or(HpRequestError::ProductNotFound(
                    product_id,
                    holder_id.clone(),
                ))?;

        if product_context
            .product_holders
            .iter()
            .chain(product_context.product_awaiters.iter())
            .any(|existent_holder_info| existent_holder_info.user_id == holder_id)
        {
            return Err(HpRequestError::UserAlreadyAdded(product_id, holder_id));
        }

        if !Self::can_hold(product_context) {
            return Err(HpRequestError::HoldersQueueAlreadyFull(
                product_id, holder_id,
            ));
        }

        product_context.product_holders.push_back(UserInfo {
            user_id: holder_id,
            queue_position: None,
        });

        Ok(())
    }

    #[inline(always)]
    pub(crate) fn add_product_awaiter(
        &mut self,
        product_id: ProductId,
        awaiter_id: UserId,
        queue_position: QueuePosition,
    ) -> Result<(), ApRequestError> {
        let product_context =
            self.product_to_context
                .get_mut(&product_id)
                .ok_or(ApRequestError::ProductNotFound(
                    product_id,
                    awaiter_id.clone(),
                ))?;

        if product_context
            .product_holders
            .iter()
            .chain(product_context.product_awaiters.iter())
            .any(|existent_user_info| existent_user_info.user_id == awaiter_id)
        {
            return Err(ApRequestError::UserAlreadyAdded(product_id, awaiter_id));
        }

        #[cfg(feature = "extra_protection")]
        if Self::queue_position_already_exists(product_context, queue_position) {
            return Err(ApRequestError::QueuePositionIsIncorrect(
                product_id,
                awaiter_id,
                queue_position,
            ));
        }

        if Self::can_hold(product_context) {
            return Err(ApRequestError::PrematureAwait(product_id, awaiter_id));
        }

        product_context.product_awaiters.push_back(UserInfo {
            user_id: awaiter_id,
            queue_position: Some(queue_position),
        });

        Ok(())
    }

    // TODO: we could use binary search here if user sent their queue_position
    // together with its user_id. The queue_positions could be ordered.
    // TODO: add decreasing of queue position to be done within this function
    // so that encapsulation is adhered to
    #[inline(always)]
    pub(crate) fn remove_holder_or_awaiter_of_product(
        &mut self,
        product_id: ProductId,
        user_id: UserId,
    ) -> Result<HaRemovalResult, RuRequestError> {
        let product_context = self
            .product_to_context
            .get_mut(&product_id)
            .ok_or(RuRequestError::ProductNotFound(product_id, user_id.clone()))?;

        println!(
            "debug | >RemovalBeginning | holders = {:?}, awaiters = {:?}",
            product_context.product_holders, product_context.product_awaiters
        );

        let removed_user_info =
            if let Some(found_index) = product_context
                .product_holders
                .iter()
                .position(|info| info.user_id == user_id)
            {
                product_context.product_holders.remove(found_index).ok_or(
                    RuRequestError::DebugError("The found holder should be present".to_owned()),
                )
            } else if let Some(found_index) = product_context
                .product_awaiters
                .iter()
                .position(|info| info.user_id == user_id)
            {
                product_context.product_awaiters.remove(found_index).ok_or(
                    RuRequestError::DebugError("The found awaiter should be present".to_owned()),
                )
            } else {
                Err(RuRequestError::UserNotFound(product_id, user_id.clone()))
            }?;

        println!(
            "debug | <RemovalEnding | holders = {:?}, awaiters = {:?}",
            product_context.product_holders, product_context.product_awaiters
        );

        Ok(HaRemovalResult {
            removed_user_id: removed_user_info.user_id,
            removed_user_queue_position: removed_user_info.queue_position,
        })
    }

    #[inline(always)]
    pub(crate) fn shift_queue_positions_of_product(
        &mut self,
        product_id: ProductId,
        max_removed_queue_position: QueuePosition,
        shift: QueuePosition,
    ) -> Option<Vec<QueueShift>> {
        let product_context = self.product_to_context.get_mut(&product_id)?;
        Some(Self::shift_queue_positions(
            product_context,
            max_removed_queue_position,
            shift,
        ))
    }

    pub(crate) fn force_remove_primary_awaiter(&mut self, product_id: ProductId) -> Option<UserId> {
        println!("debug | force_remove_primary_awaiter | trying to remove primary awaiter");
        let product_context = self.product_to_context.get_mut(&product_id)?;
        let front = product_context.product_awaiters.front()?;

        if front.queue_position == Some(0) {
            let front = product_context
                .product_awaiters
                .pop_front()
                .map(|user_info| user_info.user_id);

            println!(
                "debug | force_remove_primary_awaiter | success removed: {:?}",
                front
            );
            front
        } else {
            None
        }
    }

    #[inline(always)]
    fn shift_queue_positions(
        product_context: &mut ProductContext,
        max_removed_queue_position: QueuePosition,
        shift: QueuePosition,
    ) -> Vec<QueueShift> {
        let mut queue_shifts = Vec::new();

        // TODO: turn deque into slice and use partition_point (lower_bound)
        for awaiter_info in product_context.product_awaiters.iter_mut() {
            // TODO: add better overflow protection!
            if let Some(queue_position) = awaiter_info.queue_position.as_mut() {
                if *queue_position <= max_removed_queue_position {
                    continue;
                }

                if let Some(new_queue_position) = queue_position.checked_sub(shift) {
                    *queue_position = new_queue_position;

                    queue_shifts.push(QueueShift {
                        user_id: awaiter_info.user_id.clone(),
                        new_queue_position: Some(new_queue_position),
                    });
                }
            }
        }

        queue_shifts
    }

    #[inline(always)]
    fn can_hold(product_context: &ProductContext) -> bool {
        product_context.product_holders.len() < product_context.product_stock as usize
    }

    #[cfg(test)]
    pub(crate) fn product_context(
        &self,
        product_id: ProductId,
    ) -> Option<(&[UserInfo], &[UserInfo], ProductStock)> {
        self.product_to_context
            .get(&product_id)
            .map(|context| {
                (
                    context.product_holders.as_slices(),
                    context.product_awaiters.as_slices(),
                    context.product_stock,
                )
            })
            .map(|((holders, _), (awaiters, _), stock)| (holders, awaiters, stock))
    }

    #[inline(always)]
    #[cfg(feature = "extra_protection")]
    fn queue_position_already_exists(
        product_context: &ProductContext,
        queue_position: QueuePosition,
    ) -> bool {
        product_context
            .product_holders
            .iter()
            .chain(product_context.product_awaiters.iter())
            .any(|existent_user_info| existent_user_info.queue_position == Some(queue_position))
    }
}

#[inline(always)]
fn create_contiguous_users_deque() -> VecDeque<UserInfo> {
    let mut deque = VecDeque::default();
    deque.make_contiguous();
    deque
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queue_position_from(user_id: UserId) -> QueuePosition {
        user_id * 2
    }

    #[test]
    fn basket_holders_and_awaiters_insertion() {
        const PRODUCT_ID: ProductId = 123;
        const INITIAL_STOCK: ProductStock = 100;
        const AWAITERS_TOTAL_COUNT: UserId = 1000;
        const RANDOM_USER_ID: UserId = (INITIAL_STOCK + AWAITERS_TOTAL_COUNT) * 2;
        const RANDOM_QUEUE_POSITION: QueuePosition = RANDOM_USER_ID;

        let mut basket = Basket::new(|_, _, _| {});
        assert!(basket.product_context(PRODUCT_ID).is_none());

        let holder_addition_try =
            basket.add_product_holder(PRODUCT_ID, RANDOM_USER_ID, RANDOM_QUEUE_POSITION);
        let awaiter_addition_try =
            basket.add_product_awaiter(PRODUCT_ID, RANDOM_USER_ID, RANDOM_QUEUE_POSITION);

        assert_eq!(
            holder_addition_try,
            Err(HpRequestError::ProductNotFound(PRODUCT_ID, RANDOM_USER_ID))
        );
        assert_eq!(
            awaiter_addition_try,
            Err(HpRequestError::ProductNotFound(PRODUCT_ID, RANDOM_USER_ID))
        );

        basket.update_product_stock(PRODUCT_ID, INITIAL_STOCK);

        let product_context = basket.product_context(PRODUCT_ID);
        assert_eq!(
            product_context,
            Some(([].as_slice(), [].as_slice(), INITIAL_STOCK))
        );

        let awaiter_addition_try =
            basket.add_product_awaiter(PRODUCT_ID, RANDOM_USER_ID, RANDOM_QUEUE_POSITION);
        assert_eq!(
            awaiter_addition_try,
            Err(HpRequestError::PrematureAwait(PRODUCT_ID, RANDOM_USER_ID))
        );

        for user_id in 0..(INITIAL_STOCK - 1) {
            let queue_position = queue_position_from(user_id);
            let holder_addition_try =
                basket.add_product_holder(PRODUCT_ID, user_id, queue_position);
            assert!(holder_addition_try.is_ok());
        }

        let awaiter_addition_try =
            basket.add_product_awaiter(PRODUCT_ID, RANDOM_USER_ID, RANDOM_QUEUE_POSITION);
        assert_eq!(
            awaiter_addition_try,
            Err(HpRequestError::PrematureAwait(PRODUCT_ID, RANDOM_USER_ID))
        );

        let last_holder_id = INITIAL_STOCK - 1;
        let holder_addition_try = basket.add_product_holder(
            PRODUCT_ID,
            last_holder_id,
            queue_position_from(last_holder_id),
        );
        assert!(holder_addition_try.is_ok());

        for user_id in 0..INITIAL_STOCK {
            let holder_addition_try =
                basket.add_product_holder(PRODUCT_ID, user_id, queue_position_from(user_id));
            assert_eq!(
                holder_addition_try,
                Err(HpRequestError::UserAlreadyAdded(PRODUCT_ID, user_id))
            );
        }

        let not_added_holder = INITIAL_STOCK;
        let holder_addition_try = basket.add_product_holder(
            PRODUCT_ID,
            not_added_holder,
            queue_position_from(not_added_holder),
        );

        assert_eq!(
            holder_addition_try,
            Err(HpRequestError::HoldersQueueAlreadyFull(
                PRODUCT_ID,
                not_added_holder
            ))
        );

        for user_id in 0..INITIAL_STOCK {
            let awaiter_addition_try =
                basket.add_product_awaiter(PRODUCT_ID, user_id, queue_position_from(user_id));
            assert_eq!(
                awaiter_addition_try,
                Err(HpRequestError::UserAlreadyAdded(PRODUCT_ID, user_id))
            );
        }

        for user_id in not_added_holder..(not_added_holder + AWAITERS_TOTAL_COUNT) {
            let awaiter_addition_try =
                basket.add_product_awaiter(PRODUCT_ID, user_id, queue_position_from(user_id));
            assert!(awaiter_addition_try.is_ok());
        }

        let (holders, awaiters, stock) = basket.product_context(PRODUCT_ID).unwrap();
        let expected_holders: Box<[UserInfo; INITIAL_STOCK as usize]> =
            Box::new(std::array::from_fn(|idx| UserInfo {
                user_id: idx as UserId,
                queue_position: queue_position_from(idx as UserId),
            }));
        let expected_awaiters: Box<[UserInfo; AWAITERS_TOTAL_COUNT as usize]> =
            Box::new(std::array::from_fn(|idx| UserInfo {
                user_id: INITIAL_STOCK + idx as UserId,
                queue_position: queue_position_from(INITIAL_STOCK + idx as UserId),
            }));

        assert_eq!(holders, expected_holders.as_slice());
        assert_eq!(awaiters, expected_awaiters.as_slice());
        assert_eq!(stock, INITIAL_STOCK);
    }
}
