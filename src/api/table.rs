use std::{collections::HashMap, error::Error, vec};

use reqwest::{header::AUTHORIZATION, Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    filter::{Filter, FilterTriple},
    Baserow, BaserowTable, OrderDirection,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct RowsResponse {
    count: i32,
    next: Option<String>,
    previous: Option<String>,
    results: Vec<HashMap<String, Value>>,
}

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
                // convert json to vector of hashmaps
                let rows: RowsResponse = resp.json().await?;

                Ok(rows)
            }
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }
}
