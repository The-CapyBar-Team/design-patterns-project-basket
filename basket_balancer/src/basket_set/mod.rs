use crate::types::BasketId;
use crate::utilities::parse_u8;

pub mod traits;
pub mod vec_balancing_set;

const DEFAULT_MAX_BASKETS_COUNT: BasketId = 2;

pub(crate) const MAX_BASKETS_COUNT: BasketId = match option_env!("MAX_BASKETS_COUNT") {
    Some(s) =>  {
        let parsed_u8 = parse_u8(s);
        assert!(parsed_u8 > 0);
        parsed_u8
    },
    None => DEFAULT_MAX_BASKETS_COUNT,
};
