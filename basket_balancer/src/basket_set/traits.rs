use crate::types::BasketId;

pub(crate) trait BasketSet {
    fn add_basket_id(&mut self, basket_id: BasketId);
    fn remove_basket_id(&mut self, basket_id: BasketId);
    fn is_full(&self) -> bool;
    fn present_basket_ids(&self) -> Vec<BasketId>;
}

pub(crate) trait BasketBalancer {
    fn choose_next_basket(&self) -> Option<BasketId>;
    fn intersect(&mut self, other: &Self);
}
