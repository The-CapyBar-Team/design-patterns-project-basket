use std::{thread, time::Duration};

fn main() {
    loop {
        println!("Basket Service is up and running.");
        thread::sleep(Duration::from_secs(3));
    }
}
