# Baserow-rs

Baserow-rs is a Rust client for the Baserow API. It is a work in progress and is not yet ready for production use.

## Authentication

Baserow supports two authentication methods:

* Database Token
* JWT Token

You should use the database token for server-to-server communication and the JWT token for client-to-server communication.

**Note:** Some endpoints require a JWT token, some require a database token, and some require both.

## Usage

### Authentication (Database Token)

```rust
let configuration = ConfigBuilder::new()
    .base_url(endpoint.as_str())
    .database_token(api_key.as_str())
    .build();
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

### Retrieve a tables' rows by id

```rust
let configuration = ConfigBuilder::new()
    .base_url(endpoint.as_str())
    .database_token(api_key.as_str())
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
