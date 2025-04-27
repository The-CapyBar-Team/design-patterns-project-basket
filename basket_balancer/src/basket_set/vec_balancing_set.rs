use super::MAX_BASKETS_COUNT;
use super::traits::{BasketBalancer, BasketSet};
use crate::types::BasketId;

#[derive(Default)]
pub(crate) struct VecBalancingSet {
    available_baskets: Vec<BasketId>,
}

impl VecBalancingSet {
    #[inline(always)]
    fn contains(&self, basket_id: BasketId) -> bool {
        self.available_baskets
            .iter()
            .find(|&&found_basket_id| found_basket_id == basket_id)
            .is_some()
    }
}

impl BasketSet for VecBalancingSet {
    #[inline(always)]
    fn add_basket_id(&mut self, basket_id: BasketId) {
        if basket_id >= MAX_BASKETS_COUNT {
            return;
        }

        if !self.contains(basket_id) {
            self.available_baskets.push(basket_id);
        }
    }

    #[inline(always)]
    fn remove_basket_id(&mut self, basket_id: BasketId) {
        if let Some(position) = self
            .available_baskets
            .iter()
            .position(|&found_basket_id| found_basket_id == basket_id)
        {
            self.available_baskets.swap_remove(position);
            println!(
                "REMOVE | basket_id = {}, current_list = {:?}",
                basket_id, self.available_baskets
            );
        }
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.available_baskets.is_empty()
    }

    #[inline(always)]
    fn is_full(&self) -> bool {
        self.available_baskets.len() >= MAX_BASKETS_COUNT.into()
    }

    #[inline(always)]
    fn dump(&self) -> String {
        format!("{:?}", self.available_baskets)
    }
}

impl BasketBalancer for VecBalancingSet {
    #[inline(always)]
    fn choose_next_basket(&self) -> Option<BasketId> {
        use rand::Rng;

        if self.available_baskets.is_empty() {
            None
        } else {
            let mut rng = rand::thread_rng();
            Some(rng.gen_range(0..self.available_baskets.len()) as u8)
        }
    }

    #[inline(always)]
    fn intersect(&mut self, other: &Self) {
        self.available_baskets
            .retain(|&basket_id| other.contains(basket_id));
        println!("INTERSECT | current_list = {:?}", self.available_baskets);
    }
}
