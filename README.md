# Baserow-rs

Baserow-rs is a Rust client for the Baserow API. It is a work in progress and is not yet ready for production use.


## Usage

```rust
let configuration = ConfigBuilder::new()
    .base_url(endpoint.as_str())
    .api_key(api_key.as_str())
    .build();

let baserow = Baserow::with_configuration(configuration);

// retrieve a table by id
let rows = baserow
    .table_by_id(176)
    // grab a request builder
    .rows()
    // filter by a field
    .filter_by("field_1529", Filter::Equal, "testaaaaaaaaaa")
    // order by a field
    .order_by("field_1529", OrderDirection::Asc)
    // execute the query
    .get()
    .await?;

println!("Rows: {:#?}", rows);
```

## License

Apache 2.0
