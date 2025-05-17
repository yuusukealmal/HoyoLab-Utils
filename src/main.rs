use dotenv::dotenv;

use redeem::handle::redeem;
use sign::handle::sign;

mod redeem;
mod sign;
mod structs;
mod utils;

#[macro_use]
extern crate ini;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    sign().await?;
    redeem().await?;
    Ok(())
}
