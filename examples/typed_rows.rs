use baserow_rs::{
    api::client::BaserowClient, filter::Filter, Baserow, BaserowTableOperations, ConfigBuilder,
    OrderDirection,
};
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

    // Get rows and deserialize them into User structs with filtering and pagination
    let response = table
        .clone()
        .rows()
        .page_size(10) // Get 10 rows per page
        .filter_by("age", Filter::HigherThan, "18") // Only users over 18
        .order_by("name", OrderDirection::Asc) // Sort by name
        .get_typed::<User>()
        .await?;

    println!("Found {} total users", response.count);

    // Process the typed results
    for user in response.results {
        println!(
            "User {}: {} ({}) - Age: {:?}",
            user.id, user.name, user.email, user.age
        );
    }

    // Get next page if available
    if response.next.is_some() {
        let next_page = table
            .clone()
            .rows()
            .page_size(10)
            .offset(10) // Skip first 10 results
            .filter_by("age", Filter::HigherThan, "18")
            .order_by("name", OrderDirection::Asc)
            .get_typed::<User>()
            .await?;

        println!("\nNext page users:");
        for user in next_page.results {
            println!(
                "User {}: {} ({}) - Age: {:?}",
                user.id, user.name, user.email, user.age
            );
        }
    }

    Ok(())
}
