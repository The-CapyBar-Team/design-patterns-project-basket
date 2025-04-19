mod hold_or_await_product_request {
    include!(concat!(
        env!("OUT_DIR"),
        "/hold_or_await_product_request.rs"
    ));
}

pub use hold_or_await_product_request::*;
