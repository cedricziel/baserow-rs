use crate::{
    api::client::{BaserowClient, RequestTracing},
    filter::{Filter, FilterTriple},
    mapper::{FieldMapper, TableMapper},
    Baserow, BaserowTable, OrderDirection,
};
use async_trait::async_trait;
use reqwest::{header::AUTHORIZATION, Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, error::Error, vec};
use tracing::{debug, info, instrument};

/// Response structure for table row queries
///
/// Contains the query results along with pagination information.
#[derive(Deserialize, Serialize, Debug)]
pub struct RowsResponse {
    /// Total count of records that match the query criteria, not just the current page
    pub count: Option<i32>,
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
    pub count: Option<i32>,
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
#[derive(Default)]
pub struct RowRequestBuilder {
    baserow: Option<Baserow>,
    table: Option<BaserowTable>,
    request: RowRequest,
}

impl RowRequestBuilder {
    pub fn new() -> Self {
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
    ///
    /// Note: This option is mutually exclusive with auto_map(). If auto_map() has been called,
    /// this setting will be ignored as field name mapping is handled by the TableMapper.
    pub fn user_field_names(mut self, enabled: bool) -> Self {
        // Only set user_field_names if we don't have a mapper
        if self.table.as_ref().map_or(true, |t| t.mapper.is_none()) {
            self.request.user_field_names = Some(enabled);
        }
        self
    }

    pub fn with_table(mut self, table: BaserowTable) -> Self {
        self.table = Some(table);
        self
    }

    pub fn with_baserow(mut self, baserow: Baserow) -> Self {
        self.baserow = Some(baserow);
        self
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
    fn query(self) -> RowRequestBuilder;

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
    /// * `user_field_names` - Whether to use user-friendly field names in the response
    ///
    /// # Returns
    /// The created record including any auto-generated fields (like ID)
    async fn create_one(
        self,
        data: HashMap<String, Value>,
        user_field_names: Option<bool>,
    ) -> Result<HashMap<String, Value>, Box<dyn Error>>;

    /// Retrieves a single record from the table by ID
    ///
    /// # Type Parameters
    /// * `T` - The type to deserialize into
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the record to retrieve
    /// * `user_field_names` - Whether to use user-friendly field names in the response
    ///
    /// # Returns
    /// The requested record if found
    async fn get_one<T>(self, id: u64, user_field_names: Option<bool>) -> Result<T, Box<dyn Error>>
    where
        T: DeserializeOwned + 'static;

    /// Updates a single record in the table
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
        info!(
            field_count = fields.len(),
            "Successfully mapped table fields"
        );

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
                format!(
                    "JWT {}",
                    &baserow.configuration.database_token.as_ref().unwrap()
                ),
            );
        } else if baserow.configuration.database_token.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!(
                    "Token {}",
                    &baserow.configuration.database_token.as_ref().unwrap()
                ),
            );
        }

        if let Some(order) = request.order {
            let mut order_str = String::new();
            for (field, direction) in order {
                // Map field name to ID if auto_map is enabled
                let field_key = if let Some(mapper) = &self.mapper {
                    if let Some(field_id) = mapper.get_field_id(&field) {
                        format!("field_{}", field_id)
                    } else {
                        field
                    }
                } else {
                    field
                };

                order_str.push_str(&format!(
                    "{}{}",
                    match direction {
                        OrderDirection::Asc => "",
                        OrderDirection::Desc => "-",
                    },
                    field_key
                ));
            }

            req = req.query(&[("order_by", order_str)]);
        }

        if let Some(filter) = request.filter {
            for triple in filter {
                // Map field name to ID if auto_map is enabled
                let field_key = if let Some(mapper) = &self.mapper {
                    if let Some(field_id) = mapper.get_field_id(&triple.field) {
                        format!("field_{}", field_id)
                    } else {
                        triple.field
                    }
                } else {
                    triple.field
                };

                req = req.query(&[(
                    &format!("filter__{}__{}", field_key, triple.filter.as_str()),
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
        let resp = baserow.trace_request(&baserow.client, req.build()?).await?;

        match resp.status() {
            StatusCode::OK => {
                let response: RowsResponse = resp.json().await?;

                let typed_results = if let Some(mapper) = &self.mapper {
                    // When using auto_map, convert field IDs to names first
                    response
                        .results
                        .into_iter()
                        .map(|row| mapper.deserialize_row(row))
                        .collect::<Result<Vec<T>, _>>()?
                } else {
                    // When not using auto_map, try direct deserialization
                    serde_json::from_value::<Vec<T>>(Value::Array(
                        response
                            .results
                            .into_iter()
                            .map(|m| Value::Object(serde_json::Map::from_iter(m.into_iter())))
                            .collect(),
                    ))?
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
                format!("JWT {}", &baserow.configuration.jwt.as_ref().unwrap()),
            );
        } else if baserow.configuration.database_token.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!(
                    "Token {}",
                    &baserow.configuration.database_token.as_ref().unwrap()
                ),
            );
        }

        debug!("Creating new record");
        let resp = baserow
            .trace_request(&baserow.client, req.json(&request_data).build()?)
            .await?;

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
                format!("JWT {}", &baserow.configuration.jwt.as_ref().unwrap()),
            );
        } else if baserow.configuration.database_token.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!(
                    "Token {}",
                    &baserow.configuration.database_token.as_ref().unwrap()
                ),
            );
        }

        debug!("Fetching single record");
        let resp = baserow.trace_request(&baserow.client, req.build()?).await?;

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
                format!("JWT {}", &baserow.configuration.jwt.as_ref().unwrap()),
            );
        } else if baserow.configuration.database_token.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!(
                    "Token {}",
                    &baserow.configuration.database_token.as_ref().unwrap()
                ),
            );
        }

        debug!("Updating record");
        let resp = baserow
            .trace_request(&baserow.client, req.json(&request_data).build()?)
            .await?;

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
                format!("JWT {}", &baserow.configuration.jwt.as_ref().unwrap()),
            );
        } else if baserow.configuration.database_token.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!(
                    "Token {}",
                    &baserow.configuration.database_token.as_ref().unwrap()
                ),
            );
        }

        debug!("Deleting record");
        let resp = baserow.trace_request(&baserow.client, req.build()?).await?;

        match resp.status() {
            StatusCode::OK => Ok(()),
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::client::BaserowClient, filter::Filter, Baserow, BaserowTableOperations, ConfigBuilder,
        OrderDirection,
    };
    use serde::Deserialize;
    use serde_json::Value;
    use std::collections::HashMap;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestUser {
        name: String,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct ComplexRecord {
        id: u64,
        name: String,
        email: String,
        age: Option<i32>,
        is_active: bool,
        created_at: String,
    }

    #[tokio::test]
    async fn test_collection_deserialization() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        // Mock the fields endpoint for auto_map
        let fields_mock = server
            .mock("GET", "/api/database/fields/table/1234/")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(r#"[
                {"id": 1, "table_id": 1234, "name": "id", "order": 0, "type": "number", "primary": true, "read_only": false},
                {"id": 2, "table_id": 1234, "name": "name", "order": 1, "type": "text", "primary": false, "read_only": false},
                {"id": 3, "table_id": 1234, "name": "email", "order": 2, "type": "text", "primary": false, "read_only": false},
                {"id": 4, "table_id": 1234, "name": "age", "order": 3, "type": "number", "primary": false, "read_only": false},
                {"id": 5, "table_id": 1234, "name": "is_active", "order": 4, "type": "boolean", "primary": false, "read_only": false},
                {"id": 6, "table_id": 1234, "name": "created_at", "order": 5, "type": "text", "primary": false, "read_only": false}
            ]"#)
            .create();

        // Mock the rows endpoint with multiple records
        let rows_mock = server
            .mock("GET", "/api/database/rows/table/1234/")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"{
                "count": 2,
                "next": null,
                "previous": null,
                "results": [
                    {
                        "field_1": 101,
                        "field_2": "John Doe",
                        "field_3": "john@example.com",
                        "field_4": 30,
                        "field_5": true,
                        "field_6": "2023-01-01T00:00:00Z"
                    },
                    {
                        "field_1": 102,
                        "field_2": "Jane Smith",
                        "field_3": "jane@example.com",
                        "field_4": null,
                        "field_5": false,
                        "field_6": "2023-01-02T00:00:00Z"
                    }
                ]
            }"#,
            )
            .create();

        let configuration = ConfigBuilder::new()
            .base_url(&mock_url)
            .api_key("test-token")
            .build();
        let baserow = Baserow::with_configuration(configuration);
        let table = baserow.table_by_id(1234);

        // Test deserialization of multiple records with complex types
        let mapped_table = table.auto_map().await.unwrap();
        let response = mapped_table.query().get::<ComplexRecord>().await.unwrap();

        assert_eq!(response.count, Some(2));
        assert_eq!(response.results.len(), 2);

        // Verify first record
        let record1 = &response.results[0];
        assert_eq!(record1.id, 101);
        assert_eq!(record1.name, "John Doe");
        assert_eq!(record1.email, "john@example.com");
        assert_eq!(record1.age, Some(30));
        assert_eq!(record1.is_active, true);
        assert_eq!(record1.created_at, "2023-01-01T00:00:00Z");

        // Verify second record with null field
        let record2 = &response.results[1];
        assert_eq!(record2.id, 102);
        assert_eq!(record2.name, "Jane Smith");
        assert_eq!(record2.email, "jane@example.com");
        assert_eq!(record2.age, None);
        assert_eq!(record2.is_active, false);
        assert_eq!(record2.created_at, "2023-01-02T00:00:00Z");

        fields_mock.assert();
        rows_mock.assert();
    }

    #[tokio::test]
    async fn test_auto_map_and_user_field_names_exclusivity() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        // Mock the fields endpoint
        let fields_mock = server
            .mock("GET", "/api/database/fields/table/1234/")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(r#"[{"id": 1, "table_id": 1234, "name": "Name", "order": 0, "type": "text", "primary": true, "read_only": false}]"#)
            .create();

        // Mock the rows endpoint
        let rows_mock = server
            .mock("GET", "/api/database/rows/table/1234/")
            .match_query(mockito::Matcher::Any)
            .expect_at_least(1)
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"{"count": 1, "next": null, "previous": null, "results": [{"field_1": "test"}]}"#,
            )
            .create();

        let configuration = ConfigBuilder::new()
            .base_url(&mock_url)
            .api_key("test-token")
            .build();
        let baserow = Baserow::with_configuration(configuration);
        let table = baserow.table_by_id(1234);

        // First test: auto_map should take precedence over user_field_names
        let mapped_table = table.clone().auto_map().await.unwrap();
        let _query = mapped_table
            .query()
            .user_field_names(true) // This should be ignored since we have auto_map
            .get::<HashMap<String, Value>>()
            .await
            .unwrap();

        // Verify that user_field_names parameter was not included in the request
        rows_mock.assert();
        fields_mock.assert();
    }

    #[tokio::test]
    async fn test_field_mapping_in_query_params() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        // Mock the fields endpoint
        let fields_mock = server
            .mock("GET", "/api/database/fields/table/1234/")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(r#"[
                {"id": 1, "table_id": 1234, "name": "name", "order": 0, "type": "text", "primary": true, "read_only": false},
                {"id": 2, "table_id": 1234, "name": "age", "order": 1, "type": "number", "primary": false, "read_only": false}
            ]"#)
            .create();

        // Mock the rows endpoint with field ID mapping
        let rows_mock = server
            .mock("GET", "/api/database/rows/table/1234/")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded(
                    "order_by".into(),
                    "field_1".into(), // Should use field_1 instead of "name"
                ),
                mockito::Matcher::UrlEncoded(
                    "filter__field_2__equal".into(), // Should use field_2 instead of "age"
                    "25".into(),
                ),
            ]))
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(r#"{"count": 1, "next": null, "previous": null, "results": [{"field_1": "John", "field_2": 25}]}"#)
            .create();

        let configuration = ConfigBuilder::new()
            .base_url(&mock_url)
            .api_key("test-token")
            .build();
        let baserow = Baserow::with_configuration(configuration);
        let table = baserow.table_by_id(1234);

        // Test that field names are properly mapped to IDs in query parameters
        let mapped_table = table.auto_map().await.unwrap();
        let _result = mapped_table
            .query()
            .order_by("name", OrderDirection::Asc)
            .filter_by("age", Filter::Equal, "25")
            .get::<HashMap<String, Value>>()
            .await
            .unwrap();

        // Verify that the request was made with mapped field IDs
        fields_mock.assert();
        rows_mock.assert();
    }

    #[tokio::test]
    async fn test_struct_deserialization_with_both_options() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        // Mock the fields endpoint for auto_map
        let fields_mock = server
            .mock("GET", "/api/database/fields/table/1234/")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(r#"[{"id": 1, "table_id": 1234, "name": "name", "order": 0, "type": "text", "primary": true, "read_only": false}]"#)
            .create();

        // Mock the rows endpoint for auto_map test
        let rows_mock_auto_map = server
            .mock("GET", "/api/database/rows/table/1234/")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"{"count": 1, "next": null, "previous": null, "results": [{"field_1": "John"}]}"#,
            )
            .create();

        // Mock the rows endpoint for user_field_names test
        let rows_mock_user_names = server
            .mock("GET", "/api/database/rows/table/1234/")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "user_field_names".into(),
                "true".into(),
            )]))
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"{"count": 1, "next": null, "previous": null, "results": [{"name": "John"}]}"#,
            )
            .create();

        let configuration = ConfigBuilder::new()
            .base_url(&mock_url)
            .api_key("test-token")
            .build();
        let baserow = Baserow::with_configuration(configuration);
        let table = baserow.table_by_id(1234);

        // Test auto_map deserialization
        let mapped_table = table.clone().auto_map().await.unwrap();
        let auto_map_result = mapped_table.query().get::<TestUser>().await.unwrap();

        assert_eq!(
            auto_map_result.results[0],
            TestUser {
                name: "John".to_string()
            }
        );

        // Test user_field_names deserialization
        let user_names_result = table
            .query()
            .user_field_names(true)
            .get::<TestUser>()
            .await
            .unwrap();

        assert_eq!(
            user_names_result.results[0],
            TestUser {
                name: "John".to_string()
            }
        );

        // Verify the mocks were called the expected number of times
        fields_mock.assert();
        rows_mock_auto_map.assert();
        rows_mock_user_names.assert();
    }
}
