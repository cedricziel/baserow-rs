use std::{collections::HashMap, error::Error, vec};

use reqwest::{header::AUTHORIZATION, Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::{
    filter::{Filter, FilterTriple},
    Baserow, BaserowTable, OrderDirection,
};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_parameters() {
        let table = BaserowTable {
            id: Some(1234),
            database_id: Some(1),
            name: Some("Test".to_string()),
            baserow: None,
            mapper: None,
            order: None,
        };

        let builder = RowRequestBuilder::new()
            .with_table(table)
            .page_size(25)
            .offset(50);

        // Access internal state to verify pagination parameters
        assert_eq!(builder.page_size, Some(25));
        assert_eq!(builder.offset, Some(50));
    }

    #[test]
    fn test_view_parameter() {
        let table = BaserowTable {
            id: Some(1234),
            database_id: Some(1),
            name: Some("Test".to_string()),
            baserow: None,
            mapper: None,
            order: None,
        };

        let builder = RowRequestBuilder::new().with_table(table).view(456);

        // Verify view ID is set
        assert_eq!(builder.view_id, Some(456));
    }
}

/// Builder for constructing table row queries
///
/// Provides a fluent interface for building queries with filtering, sorting,
/// and other options.
///
/// # Examples
///
/// Basic query with filters, sorting, and view selection:
/// ```no_run
/// use baserow_rs::{ConfigBuilder, Baserow, BaserowTableOperations, OrderDirection, filter::Filter};
/// use baserow_rs::api::client::BaserowClient;
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
///     // Build a query with filters, sorting, and view selection
///     let results = table.rows()
///         .view(456)  // Query from a specific view
///         .filter_by("Status", Filter::Equal, "Active")
///         .order_by("Created", OrderDirection::Desc)
///         .get()
///         .await
///         .unwrap();
///
///     println!("Found {} matching rows", results.count);
/// }
/// ```
///
/// Paginated query:
/// ```no_run
/// # use baserow_rs::{ConfigBuilder, Baserow, BaserowTableOperations};
/// # use baserow_rs::api::client::BaserowClient;
/// # #[tokio::main]
/// # async fn main() {
/// #     let baserow = Baserow::with_configuration(ConfigBuilder::new().build());
/// #     let table = baserow.table_by_id(1234);
/// // Get first page of 25 rows
/// let page1 = table.clone().rows()
///     .page_size(25)
///     .offset(0)
///     .get()
///     .await
///     .unwrap();
///
/// // Get second page
/// let page2 = table.clone().rows()
///     .page_size(25)
///     .offset(25)
///     .get()
///     .await
///     .unwrap();
///
/// println!("Total rows: {}", page1.count);
/// println!("First page rows: {}", page1.results.len());
/// println!("Second page rows: {}", page2.results.len());
/// # }
/// ```
pub struct RowRequestBuilder {
    baserow: Option<Baserow>,
    table: Option<BaserowTable>,
    order: Option<HashMap<String, OrderDirection>>,
    filter: Option<Vec<FilterTriple>>,
    page_size: Option<i32>,
    offset: Option<i32>,
    view_id: Option<i32>,
}

impl RowRequestBuilder {
    pub(crate) fn new() -> Self {
        Self {
            baserow: None,
            table: None,
            order: None,
            filter: None,
            page_size: None,
            offset: None,
            view_id: None,
        }
    }

    /// Set the view ID to query rows from a specific view
    ///
    /// # Arguments
    /// * `id` - The ID of the view to query
    ///
    /// # Example
    /// ```no_run
    /// # use baserow_rs::{BaserowTable, BaserowTableOperations};
    /// # let table = BaserowTable::default();
    /// table.rows()
    ///     .view(123);
    /// ```
    pub fn view(self, id: i32) -> Self {
        Self {
            view_id: Some(id),
            ..self
        }
    }

    /// Set the number of rows to return per page
    ///
    /// # Arguments
    /// * `size` - The number of rows to return per page
    ///
    /// # Example
    /// ```no_run
    /// # use baserow_rs::{BaserowTable, BaserowTableOperations};
    /// # let table = BaserowTable::default();
    /// table.rows()
    ///     .page_size(25);
    /// ```
    pub fn page_size(self, size: i32) -> Self {
        Self {
            page_size: Some(size),
            ..self
        }
    }

    /// Set the offset for pagination
    ///
    /// # Arguments
    /// * `offset` - The number of rows to skip
    ///
    /// # Example
    /// ```no_run
    /// # use baserow_rs::{BaserowTable, BaserowTableOperations};
    /// # let table = BaserowTable::default();
    /// table.rows()
    ///     .offset(50);
    /// ```
    pub fn offset(self, offset: i32) -> Self {
        Self {
            offset: Some(offset),
            ..self
        }
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
    ///
    /// # Arguments
    /// * `field` - The field name to sort by
    /// * `direction` - The sort direction (Asc or Desc)
    ///
    /// # Example
    /// ```no_run
    /// # use baserow_rs::{BaserowTable, OrderDirection, BaserowTableOperations};
    /// # let table = BaserowTable::default();
    /// table.rows()
    ///     .order_by("Created", OrderDirection::Desc);
    /// ```
    pub fn order_by(self, field: &str, direction: OrderDirection) -> Self {
        match self.order {
            Some(mut order) => {
                order.insert(String::from(field), direction);
                Self {
                    order: Some(order),
                    ..self
                }
            }
            None => {
                let mut order = HashMap::new();
                order.insert(String::from(field), direction);
                Self {
                    order: Some(order),
                    ..self
                }
            }
        }
    }

    /// Add a filter condition to the query
    ///
    /// # Arguments
    /// * `field` - The field name to filter on
    /// * `filter_op` - The filter operation (Equal, Contains, etc.)
    /// * `value` - The value to filter against
    ///
    /// # Example
    /// ```no_run
    /// # use baserow_rs::{BaserowTable, filter::Filter, BaserowTableOperations};
    /// # let table = BaserowTable::default();
    /// table.rows()
    ///     .filter_by("Status", Filter::Equal, "Active");
    /// ```
    pub fn filter_by(self, field: &str, filter_op: Filter, value: &str) -> Self {
        match self.filter {
            Some(mut filter) => {
                filter.push(FilterTriple {
                    field: String::from(field),
                    filter: filter_op,
                    value: String::from(value),
                });
                Self {
                    filter: Some(filter),
                    ..self
                }
            }
            None => {
                let mut filter: Vec<FilterTriple> = vec![];
                filter.push(FilterTriple {
                    field: String::from(field),
                    filter: filter_op,
                    value: String::from(value),
                });
                Self {
                    filter: Some(filter),
                    ..self
                }
            }
        }
    }

    /// Execute the query and return the results
    ///
    /// Sends the constructed query to Baserow and returns the matching rows
    /// along with pagination information.
    ///
    /// # Returns
    /// A RowsResponse containing the matching rows and metadata
    ///
    /// # Errors
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get(self) -> Result<RowsResponse, Box<dyn Error>> {
        self.get_internal().await
    }

    /// Execute the query and return typed results
    ///
    /// Similar to get(), but deserializes the rows into the specified type.
    ///
    /// # Type Parameters
    /// * `T` - The type to deserialize each row into. Must implement DeserializeOwned.
    ///
    /// # Returns
    /// A TypedRowsResponse containing the matching rows deserialized as type T
    ///
    /// # Errors
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get_typed<T>(self) -> Result<TypedRowsResponse<T>, Box<dyn Error>>
    where
        T: DeserializeOwned,
    {
        let table = self.table.clone().ok_or("Table instance is missing")?;
        let mapper = table.mapper.ok_or("Table mapper is missing")?;
        let response = self.get_internal().await?;

        let typed_results = response
            .results
            .into_iter()
            .map(|row| mapper.deserialize_row(row))
            .collect::<Result<Vec<T>, _>>()?;

        Ok(TypedRowsResponse {
            count: response.count,
            next: response.next,
            previous: response.previous,
            results: typed_results,
        })
    }

    async fn get_internal(self) -> Result<RowsResponse, Box<dyn Error>> {
        let baserow = self.baserow.expect("Baserow instance is missing");

        let url = if let Some(view_id) = self.view_id {
            format!(
                "{}/api/database/views/{}/",
                &baserow.configuration.base_url, view_id
            )
        } else {
            format!(
                "{}/api/database/rows/table/{}/",
                &baserow.configuration.base_url,
                self.table.as_ref().unwrap().id.unwrap()
            )
        };

        let mut req = Client::new().get(url);

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

        if self.order.is_some() {
            let mut order = String::new();
            for (field, direction) in self.order.unwrap() {
                order.push_str(&format!(
                    "{}{}",
                    match direction {
                        OrderDirection::Asc => "",
                        OrderDirection::Desc => "-",
                    },
                    field
                ));
            }

            req = req.query(&[("order_by", order)]);
        }

        if self.filter.is_some() {
            for triple in self.filter.unwrap() {
                req = req.query(&[(
                    &format!("filter__{}__{}", triple.field, triple.filter.as_str()),
                    triple.value,
                )]);
            }
        }

        if let Some(size) = self.page_size {
            req = req.query(&[("size", size.to_string())]);
        }

        if let Some(offset) = self.offset {
            req = req.query(&[("offset", offset.to_string())]);
        }

        let resp = req.send().await?;

        match resp.status() {
            StatusCode::OK => {
                let rows: RowsResponse = resp.json().await?;
                Ok(rows)
            }
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }
}
