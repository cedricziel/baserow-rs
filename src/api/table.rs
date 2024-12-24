use std::{collections::HashMap, error::Error, vec};

use reqwest::{header::AUTHORIZATION, Client, StatusCode};
use serde::{Deserialize, Serialize};
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

/// Builder for constructing table row queries
///
/// Provides a fluent interface for building queries with filtering, sorting,
/// and other options.
///
/// # Example
/// ```no_run
/// use baserow_rs::{ConfigBuilder, Baserow, BaserowTableOperations, OrderDirection, filter::Filter, api::client::BaserowClient};
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
///     // Build a query with filters and sorting
///     let results = table.rows()
///         .filter_by("Status", Filter::Equal, "Active")
///         .order_by("Created", OrderDirection::Desc)
///         .get()
///         .await
///         .unwrap();
///
///     println!("Found {} matching rows", results.count);
/// }
/// ```
pub struct RowRequestBuilder {
    baserow: Option<Baserow>,
    table: Option<BaserowTable>,
    order: Option<HashMap<String, OrderDirection>>,
    filter: Option<Vec<FilterTriple>>,
}

impl RowRequestBuilder {
    pub(crate) fn new() -> Self {
        Self {
            baserow: None,
            table: None,
            order: None,
            filter: None,
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
        let baserow = self.baserow.expect("Baserow instance is missing");

        let url = format!(
            "{}/api/database/rows/table/{}/",
            &baserow.configuration.base_url,
            self.table.as_ref().unwrap().id.unwrap()
        );

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
