use crate::{
    api::{client::BaserowClient, table::RowRequestBuilder},
    mapper::{FieldMapper, TableMapper},
    BaserowTable,
};
use async_trait::async_trait;
use reqwest::{header::AUTHORIZATION, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{collections::HashMap, error::Error};

/// Trait defining the public operations available on a Baserow table
///
/// This trait provides the core CRUD operations for working with Baserow tables.
/// All operations are async and return Results to handle potential errors.
///
/// # Example
/// ```no_run
/// use baserow_rs::{ConfigBuilder, Baserow, BaserowTableOperations, api::client::BaserowClient};
/// use std::collections::HashMap;
/// use serde_json::Value;
///
/// #[tokio::main]
/// async fn main() {
///     let config = ConfigBuilder::new()
///         .base_url("https://api.baserow.io")
///         .api_key("your-api-key")
///         .build();
///
///     let baserow = Baserow::with_configuration(config);
///     let table = baserow.table_by_id(1234);
///
///     // Create a new record
///     let mut data = HashMap::new();
///     data.insert("Name".to_string(), Value::String("Test".to_string()));
///     let result = table.create_one(data).await.unwrap();
/// }
/// ```
#[async_trait]
pub trait BaserowTableOperations {
    /// Automatically maps the table fields to their corresponding types
    ///
    /// This method fetches the table schema and sets up field mappings for type conversion.
    /// Call this before performing operations if you need type-safe field access.
    async fn auto_map(self) -> Result<BaserowTable, Box<dyn Error>>;

    /// Returns a builder for constructing complex table queries
    ///
    /// The builder allows you to add filters, sorting, and pagination to your queries.
    /// Use this when you need more control over how you fetch rows from the table.
    ///
    /// # Example
    /// ```no_run
    /// use baserow_rs::{ConfigBuilder, Baserow, BaserowTableOperations, OrderDirection, api::client::BaserowClient};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = ConfigBuilder::new()
    ///         .base_url("https://api.baserow.io")
    ///         .api_key("your-api-key")
    ///         .build();
    ///
    ///     let baserow = Baserow::with_configuration(config);
    ///     let table = baserow.table_by_id(1234);
    ///
    ///     let results = table.rows()
    ///         .order_by("Created", OrderDirection::Desc)
    ///         .get()
    ///         .await
    ///         .unwrap();
    /// }
    /// ```
    fn rows(self) -> RowRequestBuilder;

    /// Creates a single record in the table
    ///
    /// # Arguments
    /// * `data` - A map of field names to values representing the record to create
    ///
    /// # Returns
    /// The created record including any auto-generated fields (like ID)
    async fn create_one(
        self,
        data: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, Box<dyn Error>>;

    /// Retrieves a single record from the table by ID
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the record to retrieve
    ///
    /// # Returns
    /// The requested record if found
    /// Retrieves a single record from the table by ID as a HashMap
    async fn get_one(self, id: u64) -> Result<HashMap<String, Value>, Box<dyn Error>>;

    /// Retrieves a single record from the table by ID and deserializes it into the specified type
    ///
    /// # Type Parameters
    /// * `T` - The type to deserialize into. Must implement DeserializeOwned.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the record to retrieve
    ///
    /// # Returns
    /// The requested record deserialized into type T if found
    async fn get_one_typed<T>(self, id: u64) -> Result<T, Box<dyn Error>>
    where
        T: DeserializeOwned;

    /// Updates a single record in the table
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the record to update
    /// * `data` - A map of field names to new values
    ///
    /// # Returns
    /// The updated record
    async fn update(
        self,
        id: u64,
        data: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, Box<dyn Error>>;

    /// Deletes a single record from the table
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the record to delete
    async fn delete(self, id: u64) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
impl BaserowTableOperations for BaserowTable {
    async fn get_one_typed<T>(mut self, id: u64) -> Result<T, Box<dyn Error>>
    where
        T: DeserializeOwned,
    {
        let mapper = self.mapper.clone().ok_or("Table mapper is missing")?;
        let row = self.get_one(id).await?;
        Ok(mapper.deserialize_row(row)?)
    }

    async fn auto_map(mut self) -> Result<BaserowTable, Box<dyn Error>> {
        let id = self.id.ok_or("Table ID is missing")?;

        let baserow = self.baserow.clone().ok_or("Baserow instance is missing")?;
        let fields = baserow.table_fields(id).await?;

        let mut mapper = TableMapper::new();
        mapper.map_fields(fields.clone());
        self.mapper = Some(mapper);

        Ok(self)
    }

    fn rows(self) -> RowRequestBuilder {
        RowRequestBuilder::new()
            .with_baserow(self.baserow.clone().unwrap())
            .with_table(self.clone())
    }

    async fn create_one(
        self,
        data: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        let baserow = self.baserow.expect("Baserow instance is missing");

        let url = format!(
            "{}/api/database/rows/table/{}/",
            &baserow.configuration.base_url,
            self.id.unwrap()
        );

        let mut req = baserow.client.post(url);

        if baserow.configuration.jwt.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!("JWT {}", &baserow.configuration.jwt.unwrap()),
            );
        } else if baserow.configuration.database_token.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!("Token {}", &baserow.configuration.database_token.unwrap()),
            );
        }

        let resp = req.json(&data).send().await?;

        match resp.status() {
            StatusCode::OK => Ok(resp.json::<HashMap<String, Value>>().await?),
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }

    async fn get_one(self, id: u64) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        let baserow = self.baserow.expect("Baserow instance is missing");

        let url = format!(
            "{}/api/database/rows/table/{}/{}/",
            &baserow.configuration.base_url,
            self.id.unwrap(),
            id
        );

        let mut req = baserow.client.get(url);

        if baserow.configuration.jwt.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!("JWT {}", &baserow.configuration.jwt.unwrap()),
            );
        } else if baserow.configuration.database_token.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!("Token {}", &baserow.configuration.database_token.unwrap()),
            );
        }

        let resp = req.send().await?;

        match resp.status() {
            StatusCode::OK => Ok(resp.json::<HashMap<String, Value>>().await?),
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }

    async fn update(
        self,
        id: u64,
        data: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        let baserow = self.baserow.expect("Baserow instance is missing");

        let url = format!(
            "{}/api/database/rows/table/{}/{}/",
            &baserow.configuration.base_url,
            self.id.unwrap(),
            id
        );

        let mut req = baserow.client.patch(url);

        if baserow.configuration.jwt.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!("JWT {}", &baserow.configuration.jwt.unwrap()),
            );
        } else if baserow.configuration.database_token.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!("Token {}", &baserow.configuration.database_token.unwrap()),
            );
        }

        let resp = req.json(&data).send().await?;

        match resp.status() {
            StatusCode::OK => Ok(resp.json::<HashMap<String, Value>>().await?),
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }

    async fn delete(self, id: u64) -> Result<(), Box<dyn Error>> {
        let baserow = self.baserow.expect("Baserow instance is missing");

        let url = format!(
            "{}/api/database/rows/table/{}/{}/",
            &baserow.configuration.base_url,
            self.id.unwrap(),
            id
        );

        let mut req = baserow.client.delete(url);

        if baserow.configuration.jwt.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!("JWT {}", &baserow.configuration.jwt.unwrap()),
            );
        } else if baserow.configuration.database_token.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!("Token {}", &baserow.configuration.database_token.unwrap()),
            );
        }

        let resp = req.send().await?;

        match resp.status() {
            StatusCode::OK => Ok(()),
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }
}
