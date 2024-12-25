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

#[derive(Debug, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    age: Option<i32>,
}

// First auto_map the table to ensure field mappings are available
let table = baserow.table_by_id(1234).auto_map().await?;

// Get a row and deserialize it into your struct
let user: User = table.get_one_typed::<User>(1).await?;
println!("Found user: {:?}", user);
```

The field names in your struct should match the column names in your Baserow table. Use `Option<T>` for nullable fields.

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

## License

Apache 2.0
