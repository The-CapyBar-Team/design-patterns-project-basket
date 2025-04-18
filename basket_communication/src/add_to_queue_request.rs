mod basket_communication {
    include!(concat!(env!("OUT_DIR"), "/basket_communication.rs"));
}

pub use basket_communication::Message;
