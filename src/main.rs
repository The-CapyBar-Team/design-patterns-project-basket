use redis::{AsyncCommands, Client};
use std::{thread, time::Duration};

const REDIS_CONNECTION_STRING: &str = "redis://redis:6379";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::open(REDIS_CONNECTION_STRING)?;
    let mut con = client.get_multiplexed_async_connection().await?;

    for i in 0..usize::MAX {
        let key_value = format!("{}", i);

        println!("Basket Service is writting ({}, {})", key_value, key_value);

        let set_result: Result<(), _> = con.set(&key_value, &key_value).await;

        if let Err(err) = set_result {
            println!("Setting error: {}", err);
        }

        let get_result: Result<Option<String>, _> = con.get(&key_value).await;

        match get_result {
            Ok(Some(resolved_value)) => {
                println!("Resolved value for key {}: {}", key_value, resolved_value)
            }
            Ok(None) => {
                println!("Value for key {} does not exist.", key_value)
            }
            Err(err) => {
                println!("Could not resolve value for key {}: {}", key_value, err)
            }
        }

        thread::sleep(Duration::from_secs(3));
    }

    Ok(())
}
