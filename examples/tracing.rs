use baserow_rs::{api::client::BaserowClient, BaserowTableOperations, ConfigBuilder};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, error, info, Level};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber with custom settings
    // You can control the log level via RUST_LOG environment variable, e.g.:
    // RUST_LOG=debug cargo run --example tracing
    // RUST_LOG=baserow_rs=trace cargo run --example tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(Level::INFO.into())
                // Enable debug logs for baserow_rs crate
                .add_directive("baserow_rs=debug".parse().unwrap()),
        )
        .with_target(true) // Include the target (module path) in output
        .with_thread_ids(true) // Include thread IDs
        .with_file(true) // Include file name
        .with_line_number(true) // Include line number
        .with_thread_names(true) // Include thread names
        .pretty() // Use pretty formatter
        .init();

    info!("Starting Baserow client example");
    debug!("Initializing client configuration");

    // Create a configuration
    let config = ConfigBuilder::new()
        .base_url("https://api.baserow.io")
        .api_key("your-api-key")
        .build();

    // Initialize the client
    let baserow = baserow_rs::Baserow::with_configuration(config);
    debug!("Client initialized successfully");

    // Get a table reference
    let table = baserow.table_by_id(1234);
    debug!(table_id = 1234, "Retrieved table reference");

    // Create a record with tracing enabled
    let mut data = HashMap::new();
    data.insert("Name".to_string(), Value::String("Test".to_string()));

    info!("Creating new record");
    debug!(?data, "Record data prepared");

    match table.clone().create_one(data, None).await {
        Ok(record) => {
            info!(record_id = ?record.get("id"), "Record created successfully");
            debug!(?record, "Full record details");
        }
        Err(e) => {
            // The error will be automatically logged with context due to the #[instrument] attribute
            error!(error = %e, "Failed to create record");
        }
    }

    // Query records with tracing
    info!("Querying records");
    debug!("Preparing query with filter");

    match table
        .clone()
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
            debug!("Processing query results");
            for record in response.results {
                debug!(?record, "Processing record");
                info!(id = ?record.get("id"), "Found record");
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to query records");
        }
    }

    info!("Example completed");
}
