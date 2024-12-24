use crate::{
    api::{client::BaserowClient, table::RowRequestBuilder},
    BaserowTable,
};
use async_trait::async_trait;
use reqwest::{header::AUTHORIZATION, StatusCode};
use serde_json::Value;
use std::{collections::HashMap, error::Error};

/// Trait defining the public operations available on a Baserow table
#[async_trait]
pub trait BaserowTableOperations {
    /// Automatically maps the table fields
    async fn auto_map(self) -> Result<BaserowTable, Box<dyn Error>>;

    /// Returns a builder for querying rows from the table
    fn rows(self) -> RowRequestBuilder;

    /// Creates a single record in the table
    async fn create_one(
        self,
        data: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, Box<dyn Error>>;

    /// Retrieves a single record from the table by ID
    async fn get_one(self, id: u64) -> Result<HashMap<String, Value>, Box<dyn Error>>;

    /// Updates a single record in the table
    async fn update(
        self,
        id: u64,
        data: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, Box<dyn Error>>;

    /// Deletes a single record from the table
    async fn delete(self, id: u64) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
impl BaserowTableOperations for BaserowTable {
    async fn auto_map(mut self) -> Result<BaserowTable, Box<dyn Error>> {
        let id = self.id.ok_or("Table ID is missing")?;

        let baserow = self.baserow.clone().ok_or("Baserow instance is missing")?;
        let fields = baserow.table_fields(id).await?;

        let mut mapper = crate::mapper::TableMapper::new();
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
