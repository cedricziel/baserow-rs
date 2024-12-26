# Baserow-rs

Baserow-rs is a Rust client for the Baserow API. It provides a comprehensive set of features for interacting with Baserow tables, including CRUD operations and file management.

## Authentication

Baserow supports two authentication methods:

* API Key (for server-to-server communication)
* JWT Token (for client-to-server communication)

**Note:** Some endpoints require a JWT token, some require an API key, and some require both.

### Authentication (API Key)

```rust
let configuration = ConfigBuilder::new()
    .base_url(endpoint.as_str())
    .api_key(api_key.as_str())
    .build();

let baserow = Baserow::with_configuration(configuration);
```

### Authentication (JWT Token)

```rust
let configuration = ConfigBuilder::new()
    .base_url(endpoint.as_str())
    .email("test@example.com")
    .password("password")
    .build();

let baserow = Baserow::with_configuration(configuration);
baserow.token_auth().await?;
```

## Table Operations

### Retrieve Table Rows

```rust
let baserow = Baserow::with_configuration(configuration);

// retrieve rows from a table
let rows = baserow
    .table_by_id(176)
    .rows()
    .filter_by("field_1529", Filter::Equal, "testaaaaaaaaaa")
    .order_by("field_1529", OrderDirection::Asc)
    .get()
    .await?;
```

### Create a Row

```rust
let mut record: HashMap<String, Value> = HashMap::new();
record.insert("field_1529".to_string(), Value::String("test".to_string()));

let row = baserow.table_by_id(176).create_one(record).await?;
```

### Update a Row

```rust
let mut record: HashMap<String, Value> = HashMap::new();
record.insert("field_1529".to_string(), Value::String("updated".to_string()));

let updated_row = baserow.table_by_id(176).update(row_id, record).await?;
```

### Get Table Fields

```rust
let fields = baserow.table_fields(table_id).await?;
```

### Map Rows to Structs

You can map table rows directly to your own structs using serde's Deserialize:

```rust
use serde::Deserialize;
use baserow_rs::{OrderDirection, filter::Filter};

#[derive(Debug, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    age: Option<i32>,
}

// First auto_map the table to ensure field mappings are available
let table = baserow.table_by_id(1234).auto_map().await?;

// Get a single row and deserialize it into your struct
let user: User = table.clone().get_one_typed::<User>(1).await?;
println!("Found user: {:?}", user);

// Query multiple rows with filtering, sorting, and pagination
let response = table.clone()
    .rows()
    .page_size(10)  // Get 10 rows per page
    .filter_by("age", Filter::HigherThan, "18")  // Only users over 18
    .order_by("name", OrderDirection::Asc)  // Sort by name
    .get_typed::<User>()
    .await?;

println!("Found {} total users", response.count);
for user in response.results {
    println!("User: {:?}", user);
}
```

The field names in your struct should match the column names in your Baserow table. Use `Option<T>` for nullable fields. Remember to clone the table when using it multiple times, as operations consume the table instance.

## File Operations

### Upload a File

```rust
let file = File::open("path/to/file").unwrap();
let result = baserow.upload_file(file, "filename.png".to_string()).await?;
```

### Upload a File via URL

```rust
let result = baserow.upload_file_via_url("https://example.com/image.png").await?;
```

## Tracing and Logging

The library includes comprehensive tracing support using the `tracing` crate, providing detailed insights into API operations, requests, and errors. All operations are instrumented with spans and events, making it easy to debug and monitor your application's interaction with Baserow.

### Enabling Tracing

To enable tracing in your application:

```rust
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

// Initialize tracing subscriber with default settings
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
    .with_file(true)
    .with_line_number(true)
    .init();
```

### What Gets Logged

The library logs the following information:
- HTTP request details (method, URL)
- Request/response status
- Operation context (table IDs, record counts)
- Detailed error information with context
- Field mapping operations
- Query construction details

Example log output:
```
INFO  [examples/tracing.rs:15] Initializing Baserow client
INFO  [examples/tracing.rs:29] Creating new record
INFO  [examples/tracing.rs:32] record_id=1234 Record created successfully
INFO  [examples/tracing.rs:41] Querying records
INFO  [examples/tracing.rs:47] record_count=1 Successfully retrieved records
```

You can control the log level using environment variables:
```bash
# Show info and above
RUST_LOG=info cargo run --example tracing

# Show debug and above
RUST_LOG=debug cargo run --example tracing

# Show trace level for baserow_rs, info for everything else
RUST_LOG=baserow_rs=trace,info cargo run --example tracing
```

See the [tracing example](examples/tracing.rs) for a complete demonstration of tracing capabilities.

## License

Apache 2.0
