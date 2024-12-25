use std::{collections::HashMap, env};

use baserow_rs::{
    api::client::BaserowClient, filter::Filter, Baserow, BaserowTableOperations, ConfigBuilder,
    OrderDirection,
};
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

    // retrieve a table by id
    let rows = baserow
        .table_by_id(176)
        .rows()
        .filter_by("field_1529", Filter::Equal, "testaaaaaaaaaa")
        .order_by("field_1529", OrderDirection::Asc)
        .get::<HashMap<String, Value>>()
        .await?;

    println!("Rows: {:#?}", rows);

    Ok(())
}
