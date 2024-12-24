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
