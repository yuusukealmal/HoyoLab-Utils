use dotenv::dotenv;

use sign::handle::sign;

mod sign;
mod structs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    sign().await?;

    Ok(())
}
