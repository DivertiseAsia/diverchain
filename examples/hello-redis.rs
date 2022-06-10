// use mini_redis::{client, Result};
use std::env;

// #[tokio::main]
// async fn main() -> Result<()> {
//     // Open a connection to the mini-redis address.
//     let mut client = client::connect("127.0.0.1:6379").await?;

//     let args: Vec<String> = env::args().collect();

//     println!("{:?}", args);

//     client.set("&args[1]", "args[2]".into()).await?;

//     Ok(())
// }

use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Open a connection to the mini-redis address.
    let mut client = client::connect("127.0.0.1:6379").await?;

    let args: Vec<String> = env::args().collect();

    // Set the key "hello" with value "world"
    let data = build_long_array_to_str(args)
    
    client.set(data, "lol".into()).await?;

    // Get key "hello"
    let result = client.get(&args[0]).await?; 

    println!("got value from the server; result={:?}", result);

    Ok(())
}