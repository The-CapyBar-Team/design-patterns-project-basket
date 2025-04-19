use basket_communication::types::{ProductId, UserId};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

pub(crate) type ProductStock = u32;

struct ProductContext {
    product_holders: VecDeque<UserId>,
    product_awaiters: VecDeque<UserId>,
    product_stock: ProductStock,
}

pub(crate) struct Basket {
    product_to_context: HashMap<ProductId, ProductContext>,
    on_product_stock_changed: Box<dyn Fn(ProductId, ProductStock, ProductStock)>,
}

impl Default for ProductContext {
    #[inline(always)]
    fn default() -> Self {
        Self {
            product_holders: create_contiguous_users_deque(),
            product_awaiters: create_contiguous_users_deque(),
            product_stock: 0,
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub(crate) enum AddProductHolderError {
    #[error("[UserId='`{1}`'] Product with id '`{0}`' is not available")]
    ProductNotFound(ProductId, UserId),

    #[error(
        "[UserId='`{1}`'] Cannot hold product with id `'{0}'`: holders queue reached its max size"
    )]
    HoldersQueueAlreadyFull(ProductId, UserId),

    #[error("[UserId='`{1}`'] Trying to await product with id '`{0}`' while its holders queue has not reached its max size")]
    PrematureAwait(ProductId, UserId),

    #[error(
        "[UserId='`{1}`'] The user is already added to the context of the product with id {0}"
    )]
    UserAlreadyAdded(ProductId, UserId),
}

impl Basket {
    #[inline(always)]
    pub(crate) fn new(
        on_product_stock_changed: impl Fn(ProductId, ProductStock, ProductStock) + 'static,
    ) -> Self {
        Self {
            product_to_context: Default::default(),
            on_product_stock_changed: Box::new(on_product_stock_changed),
        }
    }

    #[inline(always)]
    pub(crate) fn update_product_stock(&mut self, product_id: ProductId, new_stock: ProductStock) {
        let old_stock = if let Some(entry) = self.product_to_context.get_mut(&product_id) {
            let old_stock = entry.product_stock;
            (*entry).product_stock = new_stock;
            old_stock
        } else {
            let new_context = ProductContext {
                product_stock: new_stock,
                ..Default::default()
            };

            self.product_to_context
                .insert(product_id, new_context)
                .map(|context| context.product_stock)
                .unwrap_or(0)
        };

        (self.on_product_stock_changed)(product_id, old_stock, new_stock);
    }

    #[inline(always)]
    pub(crate) fn add_product_holder(
        &mut self,
        product_id: ProductId,
        holder_id: UserId,
    ) -> Result<(), AddProductHolderError> {
        let product_context = self.product_to_context.get_mut(&product_id).ok_or(
            AddProductHolderError::ProductNotFound(product_id, holder_id),
        )?;

        if let Some(_) = product_context
            .product_holders
            .iter()
            .find(|&&existent_holder_id| existent_holder_id == holder_id)
        {
            return Err(AddProductHolderError::UserAlreadyAdded(
                product_id, holder_id,
            ));
        }

        if !Self::can_hold(&product_context) {
            return Err(AddProductHolderError::HoldersQueueAlreadyFull(
                product_id, holder_id,
            ));
        }

        product_context.product_holders.push_back(holder_id);

        Ok(())
    }

    #[inline(always)]
    pub(crate) fn add_product_awaiter(
        &mut self,
        product_id: ProductId,
        awaiter_id: UserId,
    ) -> Result<(), AddProductHolderError> {
        let product_context = self.product_to_context.get_mut(&product_id).ok_or(
            AddProductHolderError::ProductNotFound(product_id, awaiter_id),
        )?;

        if let Some(_) = product_context
            .product_holders
            .iter()
            .chain(product_context.product_awaiters.iter())
            .find(|&&existent_user_id| existent_user_id == awaiter_id)
        {
            return Err(AddProductHolderError::UserAlreadyAdded(
                product_id, awaiter_id,
            ));
        }

        if Self::can_hold(&product_context) {
            return Err(AddProductHolderError::PrematureAwait(
                product_id, awaiter_id,
            ));
        }

        product_context.product_awaiters.push_back(awaiter_id);

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn product_context(
        &self,
        product_id: ProductId,
    ) -> Option<(&[UserId], &[UserId], ProductStock)> {
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
    fn can_hold(product_context: &ProductContext) -> bool {
        product_context.product_holders.len() < product_context.product_stock as usize
    }
}

#[inline(always)]
fn create_contiguous_users_deque() -> VecDeque<UserId> {
    let mut deque = VecDeque::default();
    deque.make_contiguous();
    deque
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basket_holders_and_awaiters_insertion() {
        const PRODUCT_ID: ProductId = 123;
        const INITIAL_STOCK: ProductStock = 10000;
        const AWAITERS_TOTAL_COUNT: UserId = 10000;
        const RANDOM_USER_ID: UserId = (INITIAL_STOCK + AWAITERS_TOTAL_COUNT) * 2;

        let mut basket = Basket::new(|_, _, _| {});
        assert!(basket.product_context(PRODUCT_ID).is_none());

        let holder_addition_try = basket.add_product_holder(PRODUCT_ID, RANDOM_USER_ID);
        let awaiter_addition_try = basket.add_product_awaiter(PRODUCT_ID, RANDOM_USER_ID);

        assert_eq!(
            holder_addition_try,
            Err(AddProductHolderError::ProductNotFound(
                PRODUCT_ID,
                RANDOM_USER_ID
            ))
        );
        assert_eq!(
            awaiter_addition_try,
            Err(AddProductHolderError::ProductNotFound(
                PRODUCT_ID,
                RANDOM_USER_ID
            ))
        );

        basket.update_product_stock(PRODUCT_ID, INITIAL_STOCK);

        let product_context = basket.product_context(PRODUCT_ID);
        assert_eq!(
            product_context,
            Some(([].as_slice(), [].as_slice(), INITIAL_STOCK))
        );

        let awaiter_addition_try = basket.add_product_awaiter(PRODUCT_ID, RANDOM_USER_ID);
        assert_eq!(
            awaiter_addition_try,
            Err(AddProductHolderError::PrematureAwait(
                PRODUCT_ID,
                RANDOM_USER_ID
            ))
        );

        for user_id in 0..(INITIAL_STOCK - 1) {
            let holder_addition_try = basket.add_product_holder(PRODUCT_ID, user_id);
            assert!(holder_addition_try.is_ok());
        }

        let awaiter_addition_try = basket.add_product_awaiter(PRODUCT_ID, RANDOM_USER_ID);
        assert_eq!(
            awaiter_addition_try,
            Err(AddProductHolderError::PrematureAwait(
                PRODUCT_ID,
                RANDOM_USER_ID
            ))
        );

        let last_holder_id = INITIAL_STOCK - 1;
        let holder_addition_try = basket.add_product_holder(PRODUCT_ID, last_holder_id);
        assert!(holder_addition_try.is_ok());

        for user_id in 0..INITIAL_STOCK {
            let holder_addition_try = basket.add_product_holder(PRODUCT_ID, user_id);
            assert_eq!(
                holder_addition_try,
                Err(AddProductHolderError::UserAlreadyAdded(PRODUCT_ID, user_id))
            );
        }

        let not_added_holder = INITIAL_STOCK;
        let holder_addition_try = basket.add_product_holder(PRODUCT_ID, not_added_holder);

        assert_eq!(
            holder_addition_try,
            Err(AddProductHolderError::HoldersQueueAlreadyFull(
                PRODUCT_ID,
                not_added_holder
            ))
        );

        for user_id in 0..INITIAL_STOCK {
            let awaiter_addition_try = basket.add_product_awaiter(PRODUCT_ID, user_id);
            assert_eq!(
                awaiter_addition_try,
                Err(AddProductHolderError::UserAlreadyAdded(PRODUCT_ID, user_id))
            );
        }

        for user_id in not_added_holder..(not_added_holder + AWAITERS_TOTAL_COUNT) {
            let awaiter_addition_try = basket.add_product_awaiter(PRODUCT_ID, user_id);
            assert!(awaiter_addition_try.is_ok());
        }

        let (holders, awaiters, stock) = basket.product_context(PRODUCT_ID).unwrap();
        let expected_holders: Box<[UserId; INITIAL_STOCK as usize]> =
            Box::new(std::array::from_fn(|idx| idx as UserId));
        let expected_awaiters: Box<[UserId; AWAITERS_TOTAL_COUNT as usize]> =
            Box::new(std::array::from_fn(|idx| INITIAL_STOCK + idx as UserId));
        assert_eq!(holders, expected_holders.as_slice());
        assert_eq!(awaiters, expected_awaiters.as_slice());
        assert_eq!(stock, INITIAL_STOCK);
    }
}
