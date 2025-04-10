use redis::{AsyncCommands, Client};
use std::{thread, time::Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::open("redis://redis:6379")?;
    let mut con = client.get_multiplexed_async_connection().await?;

    for i in 0..usize::MAX {
        let key_value = format!("{}", i);
        println!("Basket Service is writting ({}, {})", key_value, key_value);
        con.set(&key_value, &key_value).await?;

        if let Some(value) = con.get::<_, Option<String>>(&key_value).await? {
            println!("Resolved value for key {}: {}", key_value, value);
        } else {
            println!("Could not resolve value for key: {}", key_value);
        }

        thread::sleep(Duration::from_secs(3));
    }

    Ok(())
}

// fn main() {
//     loop {
//         println!("Basket Service is up and running.");
//         thread::sleep(Duration::from_secs(3));
//     }
// }
