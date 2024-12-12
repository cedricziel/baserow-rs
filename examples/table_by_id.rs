use std::env;

use baserow_rs::{filter::Filter, Baserow, ConfigBuilder, OrderDirection};

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
        .get()
        .await?;

    println!("Rows: {:#?}", rows);

    Ok(())
}
