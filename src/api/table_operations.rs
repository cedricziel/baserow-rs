use crate::{
    api::client::BaserowClient,
    filter::{Filter, FilterTriple},
    mapper::{FieldMapper, TableMapper},
    Baserow, BaserowTable, OrderDirection,
};
use async_trait::async_trait;
use reqwest::{header::AUTHORIZATION, Client, StatusCode};
use tracing::{debug, error, info, instrument, warn, Instrument, span, Level};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, error::Error, vec};

/// Response structure for table row queries
///
/// Contains the query results along with pagination information.
#[derive(Deserialize, Serialize, Debug)]
pub struct RowsResponse {
    /// Total count of records that match the query criteria, not just the current page
    pub count: i32,
    /// URL for the next page of results, if available
    pub next: Option<String>,
    /// URL for the previous page of results, if available
    pub previous: Option<String>,
    /// The actual rows returned by the query
    pub results: Vec<HashMap<String, Value>>,
}

/// Response structure for typed table row queries
///
/// Contains the query results along with pagination information, where results
/// are deserialized into the specified type.
#[derive(Deserialize, Serialize, Debug)]
pub struct TypedRowsResponse<T> {
    /// Total count of records that match the query criteria, not just the current page
    pub count: i32,
    /// URL for the next page of results, if available
    pub next: Option<String>,
    /// URL for the previous page of results, if available
    pub previous: Option<String>,
    /// The actual rows returned by the query, deserialized into type T
    pub results: Vec<T>,
}

/// Represents a query request for table rows
///
/// This struct encapsulates all the parameters that can be used to query rows
/// from a Baserow table, including filtering, sorting, and pagination options.
#[derive(Clone, Debug)]
pub struct RowRequest {
    /// Optional view ID to query rows from a specific view
    pub view_id: Option<i32>,
    /// Optional sorting criteria
    pub order: Option<HashMap<String, OrderDirection>>,
    /// Optional filter conditions
    pub filter: Option<Vec<FilterTriple>>,
    /// Optional page size for pagination
    pub page_size: Option<i32>,
    /// Optional page number for pagination
    pub page: Option<i32>,
    /// Optional flag to use user-friendly field names in the response
    pub user_field_names: Option<bool>,
}

impl Default for RowRequest {
    fn default() -> Self {
        Self {
            view_id: None,
            order: None,
            filter: None,
            page_size: Some(100),
            page: Some(1),
            user_field_names: None,
        }
    }
}

/// Builder for constructing table row queries
///
/// Provides a fluent interface for building queries with filtering, sorting,
/// and other options.
pub struct RowRequestBuilder {
    baserow: Option<Baserow>,
    table: Option<BaserowTable>,
    request: RowRequest,
}

impl RowRequestBuilder {
    pub(crate) fn new() -> Self {
        Self {
            baserow: None,
            table: None,
            request: RowRequest::default(),
        }
    }

    /// Set the view ID to query rows from a specific view
    pub fn view(mut self, id: i32) -> Self {
        self.request.view_id = Some(id);
        self
    }

    /// Set the number of rows to return per page
    ///
    /// # Arguments
    /// * `size` - The number of rows per page (must be positive)
    #[deprecated(since = "1.1.0", note = "Use `size` instead")]
    pub fn page_size(mut self, size: i32) -> Self {
        self.request.page_size = Some(size);
        self
    }

    /// Set the number of rows to return per page
    ///
    /// # Arguments
    /// * `size` - The number of rows per page (must be positive)
    pub fn size(mut self, size: i32) -> Self {
        self.request.page_size = Some(size);
        self
    }

    /// Set the page number for pagination
    ///
    /// # Arguments
    /// * `page` - The page number (must be positive)
    pub fn page(mut self, page: i32) -> Self {
        self.request.page = Some(page);
        self
    }

    /// Set whether to use user-friendly field names in the response
    pub fn user_field_names(mut self, enabled: bool) -> Self {
        self.request.user_field_names = Some(enabled);
        self
    }

    pub fn with_table(self, table: BaserowTable) -> Self {
        Self {
            table: Some(table),
            ..self
        }
    }

    pub fn with_baserow(self, baserow: Baserow) -> Self {
        Self {
            baserow: Some(baserow),
            ..self
        }
    }

    /// Add sorting criteria to the query
    pub fn order_by(mut self, field: &str, direction: OrderDirection) -> Self {
        match self.request.order {
            Some(mut order) => {
                order.insert(String::from(field), direction);
                self.request.order = Some(order);
            }
            None => {
                let mut order = HashMap::new();
                order.insert(String::from(field), direction);
                self.request.order = Some(order);
            }
        }
        self
    }

    /// Add a filter condition to the query
    pub fn filter_by(mut self, field: &str, filter_op: Filter, value: &str) -> Self {
        match self.request.filter {
            Some(mut filter) => {
                filter.push(FilterTriple {
                    field: String::from(field),
                    filter: filter_op,
                    value: String::from(value),
                });
                self.request.filter = Some(filter);
            }
            None => {
                let mut filter: Vec<FilterTriple> = vec![];
                filter.push(FilterTriple {
                    field: String::from(field),
                    filter: filter_op,
                    value: String::from(value),
                });
                self.request.filter = Some(filter);
            }
        }
        self
    }

    /// Execute the query and return typed results
    pub async fn get<T>(self) -> Result<TypedRowsResponse<T>, Box<dyn Error>>
    where
        T: DeserializeOwned + 'static,
    {
        let table = self.table.expect("Table instance is missing");
        let baserow = self.baserow.expect("Baserow instance is missing");
        table.get(baserow, self.request).await
    }
}

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
///     let result = table.create_one(data, None).await.unwrap();
/// }
/// ```
#[async_trait]
pub trait BaserowTableOperations {
    /// Automatically maps the table fields to their corresponding types
    ///
    /// This method fetches the table schema and sets up field mappings for type conversion.
    /// Call this before performing operations if you need type-safe field access.
    async fn auto_map(self) -> Result<BaserowTable, Box<dyn Error>>;

    /// Creates a new query builder for constructing complex table queries
    ///
    /// This is the preferred method for building queries with filters, sorting,
    /// and pagination options. The builder provides a fluent interface for
    /// constructing queries.
    ///
    /// # Example
    /// ```no_run
    /// use baserow_rs::{ConfigBuilder, Baserow, BaserowTableOperations, OrderDirection, filter::Filter};
    /// use baserow_rs::api::client::BaserowClient;
    /// use std::collections::HashMap;
    /// use serde_json::Value;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let baserow = Baserow::with_configuration(ConfigBuilder::new().build());
    ///     let table = baserow.table_by_id(1234);
    ///
    ///     let results = table.query()
    ///         .filter_by("Status", Filter::Equal, "Active")
    ///         .order_by("Created", OrderDirection::Desc)
    ///         .get::<HashMap<String, Value>>()
    ///         .await
    ///         .unwrap();
    /// }
    /// ```
    fn query(self) -> RowRequestBuilder;

    #[deprecated(since = "0.1.0", note = "Use the query() method instead")]
    fn rows(self) -> RowRequestBuilder;

    /// Execute a row request and return typed results
    ///
    /// # Type Parameters
    /// * `T` - The type to deserialize the results into
    ///
    /// # Arguments
    /// * `request` - The query parameters encapsulated in a RowRequest
    ///
    /// # Returns
    /// A TypedRowsResponse containing the query results and pagination information
    async fn get<T>(
        &self,
        baserow: Baserow,
        request: RowRequest,
    ) -> Result<TypedRowsResponse<T>, Box<dyn Error>>
    where
        T: DeserializeOwned + 'static;

    /// Creates a single record in the table
    ///
    /// # Arguments
    /// * `data` - A map of field names to values representing the record to create
    ///
    /// # Returns
    /// The created record including any auto-generated fields (like ID)
    /// Creates a single record in the table with optional user-friendly field names
    ///
    /// # Arguments
    /// * `data` - A map of field names to values representing the record to create
    /// * `user_field_names` - Whether to use user-friendly field names in the response
    ///
    /// # Returns
    /// The created record including any auto-generated fields (like ID)
    async fn create_one(
        self,
        data: HashMap<String, Value>,
        user_field_names: Option<bool>,
    ) -> Result<HashMap<String, Value>, Box<dyn Error>>;

    /// Retrieves a single record from the table by ID with optional user-friendly field names
    ///
    /// # Type Parameters
    /// * `T` - The type to deserialize into. Defaults to HashMap<String, Value>.
    ///         When using a custom type, the table must be mapped using `auto_map()` first.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the record to retrieve
    ///
    /// # Returns
    /// The requested record if found, either as a HashMap or deserialized into type T
    ///
    /// # Example
    /// ```no_run
    /// use baserow_rs::{ConfigBuilder, Baserow, BaserowTableOperations, api::client::BaserowClient};
    /// use serde::Deserialize;
    /// use std::collections::HashMap;
    /// use serde_json::Value;
    ///
    /// #[derive(Deserialize)]
    /// struct User {
    ///     name: String,
    ///     email: String,
    /// }
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
    ///     // Get as HashMap (default)
    ///     let row: HashMap<String, Value> = table.clone().get_one(1, None).await.unwrap();
    ///
    ///     // Get as typed struct
    ///     let user: User = table.auto_map()
    ///         .await
    ///         .unwrap()
    ///         .get_one(1, None)
    ///         .await
    ///         .unwrap();
    /// }
    /// ```
    async fn get_one<T>(self, id: u64, user_field_names: Option<bool>) -> Result<T, Box<dyn Error>>
    where
        T: DeserializeOwned + 'static;

    /// Updates a single record in the table
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the record to update
    /// * `data` - A map of field names to new values
    ///
    /// # Returns
    /// The updated record
    /// Updates a single record in the table with optional user-friendly field names
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the record to update
    /// * `data` - A map of field names to new values
    /// * `user_field_names` - Whether to use user-friendly field names in the response
    ///
    /// # Returns
    /// The updated record
    async fn update(
        self,
        id: u64,
        data: HashMap<String, Value>,
        user_field_names: Option<bool>,
    ) -> Result<HashMap<String, Value>, Box<dyn Error>>;

    /// Deletes a single record from the table
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the record to delete
    async fn delete(self, id: u64) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
impl BaserowTableOperations for BaserowTable {
    #[instrument(skip(self), fields(table_id = ?self.id), err)]
    async fn auto_map(mut self) -> Result<BaserowTable, Box<dyn Error>> {
        let id = self.id.ok_or("Table ID is missing")?;

        let baserow = self.baserow.clone().ok_or("Baserow instance is missing")?;
        debug!("Fetching table fields for mapping");
        let fields = baserow.table_fields(id).await?;
        info!(field_count = fields.len(), "Successfully mapped table fields");

        let mut mapper = TableMapper::new();
        mapper.map_fields(fields.clone());
        self.mapper = Some(mapper);

        Ok(self)
    }

    fn query(self) -> RowRequestBuilder {
        RowRequestBuilder::new()
            .with_baserow(self.baserow.clone().unwrap())
            .with_table(self.clone())
    }

    fn rows(self) -> RowRequestBuilder {
        self.query()
    }

    #[instrument(skip(self, baserow), fields(table_id = ?self.id), err)]
    async fn get<T>(
        &self,
        baserow: Baserow,
        request: RowRequest,
    ) -> Result<TypedRowsResponse<T>, Box<dyn Error>>
    where
        T: DeserializeOwned + 'static,
    {
        // Validate pagination parameters
        if let Some(size) = request.page_size {
            if size <= 0 {
                return Err("Page size must be a positive integer".into());
            }
        }
        if let Some(page) = request.page {
            if page <= 0 {
                return Err("Page number must be a positive integer".into());
            }
        }

        let url = format!(
            "{}/api/database/rows/table/{}/",
            &baserow.configuration.base_url,
            self.id.unwrap()
        );

        let mut req = Client::new().get(url);

        if let Some(view_id) = request.view_id {
            req = req.query(&[("view_id", view_id.to_string())]);
        }

        if baserow.configuration.jwt.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!("JWT {}", &baserow.configuration.database_token.unwrap()),
            );
        } else if baserow.configuration.database_token.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!("Token {}", &baserow.configuration.database_token.unwrap()),
            );
        }

        if let Some(order) = request.order {
            let mut order_str = String::new();
            for (field, direction) in order {
                order_str.push_str(&format!(
                    "{}{}",
                    match direction {
                        OrderDirection::Asc => "",
                        OrderDirection::Desc => "-",
                    },
                    field
                ));
            }

            req = req.query(&[("order_by", order_str)]);
        }

        if let Some(filter) = request.filter {
            for triple in filter {
                req = req.query(&[(
                    &format!("filter__{}__{}", triple.field, triple.filter.as_str()),
                    triple.value,
                )]);
            }
        }

        if let Some(size) = request.page_size {
            req = req.query(&[("size", size.to_string())]);
        }

        if let Some(page) = request.page {
            req = req.query(&[("page", page.to_string())]);
        }

        if let Some(user_field_names) = request.user_field_names {
            req = req.query(&[("user_field_names", user_field_names.to_string())]);
        }

        debug!("Executing table query");
        let resp = baserow.client.execute(req.build()?).await?;

        match resp.status() {
            StatusCode::OK => {
                let response: RowsResponse = resp.json().await?;

                // Try direct deserialization first
                let results_clone = response.results.clone();
                let typed_results = match serde_json::from_value::<Vec<T>>(Value::Array(
                    results_clone
                        .into_iter()
                        .map(|m| Value::Object(serde_json::Map::from_iter(m.into_iter())))
                        .collect(),
                )) {
                    Ok(results) => results,
                    Err(_) => {
                        // Fall back to mapper for custom types
                        let mapper = self.mapper.clone().ok_or("Table mapper is missing. Call auto_map() first when using typed responses.")?;
                        response
                            .results
                            .into_iter()
                            .map(|row| mapper.deserialize_row(row))
                            .collect::<Result<Vec<T>, _>>()?
                    }
                };

                Ok(TypedRowsResponse {
                    count: response.count,
                    next: response.next,
                    previous: response.previous,
                    results: typed_results,
                })
            }
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }

    #[instrument(skip(self, data), fields(table_id = ?self.id, field_count = data.len()), err)]
    async fn create_one(
        self,
        data: HashMap<String, Value>,
        user_field_names: Option<bool>,
    ) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        let baserow = self.baserow.expect("Baserow instance is missing");

        let url = format!(
            "{}/api/database/rows/table/{}/",
            &baserow.configuration.base_url,
            self.id.unwrap()
        );

        let mut req = baserow.client.post(url);

        // Convert field names to IDs if auto_map is enabled
        let request_data = if self.mapper.is_some() {
            self.mapper.as_ref().unwrap().convert_to_field_ids(data)
        } else {
            data
        };

        // Use user_field_names parameter for response format
        if let Some(use_names) = user_field_names {
            req = req.query(&[("user_field_names", use_names.to_string())]);
        }

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

        debug!("Creating new record");
        let resp = baserow.client.execute(req.json(&request_data).build()?).await?;

        match resp.status() {
            StatusCode::OK => {
                let response_data = resp.json::<HashMap<String, Value>>().await?;

                // Convert response field IDs to names if auto_map is enabled
                if self.mapper.is_some() && user_field_names != Some(true) {
                    Ok(self
                        .mapper
                        .as_ref()
                        .unwrap()
                        .convert_to_field_names(response_data))
                } else {
                    Ok(response_data)
                }
            }
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }

    #[instrument(skip(self), fields(table_id = ?self.id, record_id = %id), err)]
    async fn get_one<T>(
        mut self,
        id: u64,
        user_field_names: Option<bool>,
    ) -> Result<T, Box<dyn Error>>
    where
        T: DeserializeOwned + 'static,
    {
        let baserow = self.baserow.expect("Baserow instance is missing");

        let url = format!(
            "{}/api/database/rows/table/{}/{}/",
            &baserow.configuration.base_url,
            self.id.unwrap(),
            id
        );

        let mut req = baserow.client.get(url);

        if let Some(use_names) = user_field_names {
            req = req.query(&[("user_field_names", use_names.to_string())]);
        }

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

        debug!("Fetching single record");
        let resp = baserow.client.execute(req.build()?).await?;

        match resp.status() {
            StatusCode::OK => {
                let row: HashMap<String, Value> = resp.json().await?;

                // For HashMap<String, Value>, use serde to convert
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<HashMap<String, Value>>() {
                    Ok(serde_json::from_value(serde_json::to_value(row)?)?)
                } else {
                    // For other types, use the mapper if available
                    let mapper = self.mapper.clone().ok_or("Table mapper is missing. Call auto_map() first when using typed responses.")?;
                    Ok(mapper.deserialize_row(row)?)
                }
            }
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }

    #[instrument(skip(self, data), fields(table_id = ?self.id, record_id = %id, field_count = data.len()), err)]
    async fn update(
        self,
        id: u64,
        data: HashMap<String, Value>,
        user_field_names: Option<bool>,
    ) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        let baserow = self.baserow.expect("Baserow instance is missing");

        let url = format!(
            "{}/api/database/rows/table/{}/{}/",
            &baserow.configuration.base_url,
            self.id.unwrap(),
            id
        );

        let mut req = baserow.client.patch(url);

        // Convert field names to IDs if auto_map is enabled
        let request_data = if self.mapper.is_some() {
            self.mapper.as_ref().unwrap().convert_to_field_ids(data)
        } else {
            data
        };

        // Use user_field_names parameter for response format
        if let Some(use_names) = user_field_names {
            req = req.query(&[("user_field_names", use_names.to_string())]);
        }

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

        debug!("Updating record");
        let resp = baserow.client.execute(req.json(&request_data).build()?).await?;

        match resp.status() {
            StatusCode::OK => {
                let response_data = resp.json::<HashMap<String, Value>>().await?;

                // Convert response field IDs to names if auto_map is enabled
                if self.mapper.is_some() && user_field_names != Some(true) {
                    Ok(self
                        .mapper
                        .as_ref()
                        .unwrap()
                        .convert_to_field_names(response_data))
                } else {
                    Ok(response_data)
                }
            }
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }

    #[instrument(skip(self), fields(table_id = ?self.id, record_id = %id), err)]
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

        debug!("Deleting record");
        let resp = baserow.client.execute(req.build()?).await?;

        match resp.status() {
            StatusCode::OK => Ok(()),
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }
}
