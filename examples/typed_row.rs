use baserow_rs::{api::client::BaserowClient, Baserow, BaserowTableOperations, ConfigBuilder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    age: Option<i32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigBuilder::new()
        .base_url("https://api.baserow.io")
        .api_key("your-api-key")
        .build();

    let baserow = Baserow::with_configuration(config);

    // First auto_map the table to ensure field mappings are available
    let table = baserow.table_by_id(1234).auto_map().await?;

    // Get a row and deserialize it into our User struct
    match table.get_one::<User>(1).await {
        Ok(user) => println!("Found user: {:?}", user),
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}
