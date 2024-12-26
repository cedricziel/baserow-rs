use std::{collections::HashMap, env};

use baserow_rs::{api::client::BaserowClient, Baserow, BaserowTableOperations, ConfigBuilder};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = env::var("BASEROW_ENDPOINT").expect("BASEROW_ENDPOINT is not set");
    let api_key = env::var("BASEROW_API_KEY").expect("BASEROW_API_KEY is not set");

    let configuration = ConfigBuilder::new()
        .base_url(endpoint.as_str())
        .api_key(api_key.as_str())
        .build();

    let baserow = Baserow::with_configuration(configuration);
    let table = baserow.table_by_id(176);

    // Approach 1: Using user_field_names parameter
    let mut record = HashMap::new();
    record.insert("field_1529".to_string(), Value::String("test".to_string()));
    let result = table.clone().create_one(record, Some(true)).await?;
    println!("Created record with user_field_names=true: {:#?}", result);

    // Approach 2: Using auto_map()
    let mapped_table = table.auto_map().await?;
    let mut record = HashMap::new();
    record.insert("Name".to_string(), Value::String("test".to_string())); // Use actual field name
    let result = mapped_table.create_one(record, None).await?;
    println!("Created record with auto_map: {:#?}", result);

    Ok(())
}
