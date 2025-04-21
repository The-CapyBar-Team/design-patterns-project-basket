mod hp_request {
    include!(concat!(
        env!("OUT_DIR"),
        "/hp_request.rs"
    ));
}

pub use hp_request::*;
