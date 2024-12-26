use baserow_rs::{api::client::BaserowClient, BaserowTableOperations, ConfigBuilder};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber with default settings
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Initializing Baserow client");

    // Create a configuration
    let config = ConfigBuilder::new()
        .base_url("https://api.baserow.io")
        .api_key("your-api-key")
        .build();

    // Initialize the client
    let baserow = baserow_rs::Baserow::with_configuration(config);

    // Get a table reference
    let table = baserow.table_by_id(1234);

    // Create a record with tracing enabled
    let mut data = HashMap::new();
    data.insert("Name".to_string(), Value::String("Test".to_string()));

    info!("Creating new record");
    match table.clone().create_one(data, None).await {
        Ok(record) => {
            info!(record_id = ?record.get("id"), "Record created successfully");
        }
        Err(e) => {
            // The error will be automatically logged with context due to the #[instrument] attribute
            eprintln!("Failed to create record: {}", e);
        }
    }

    // Query records with tracing
    info!("Querying records");
    match table.clone()
        .query()
        .filter_by("Name", baserow_rs::filter::Filter::Equal, "Test")
        .get::<HashMap<String, Value>>()
        .await
    {
        Ok(response) => {
            info!(
                record_count = response.count,
                "Successfully retrieved records"
            );
            for record in response.results {
                info!(?record, "Found record");
            }
        }
        Err(e) => {
            eprintln!("Failed to query records: {}", e);
        }
    }
}
