mod ups_request {
    include!(concat!(env!("OUT_DIR"), "/ups_request.rs"));
}

pub use ups_request::*;
