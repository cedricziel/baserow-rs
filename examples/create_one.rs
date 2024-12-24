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

    let mut record: HashMap<String, Value> = HashMap::new();
    record.insert("field_1529".to_string(), Value::String("test".to_string()));

    // retrieve a table by id
    let rows = baserow.table_by_id(176).create_one(record).await?;

    println!("Rows: {:#?}", rows);

    Ok(())
}
