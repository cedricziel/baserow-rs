//! A Rust client for the Baserow API
//!
//! This crate provides a strongly-typed client for interacting with Baserow's REST API.
//! It supports authentication, table operations, file uploads, and more.
//!
//! # Example
//! ```no_run
//! use baserow_rs::{ConfigBuilder, Baserow, BaserowTableOperations, api::client::BaserowClient};
//! use std::collections::HashMap;
//! use serde_json::Value;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a configuration
//!     let config = ConfigBuilder::new()
//!         .base_url("https://api.baserow.io")
//!         .api_key("your-api-key")
//!         .build();
//!
//!     // Initialize the client
//!     let baserow = Baserow::with_configuration(config);
//!
//!     // Get a table reference
//!     let table = baserow.table_by_id(1234);
//!
//!     // Create a record
//!     let mut data = HashMap::new();
//!     data.insert("Name".to_string(), Value::String("Test".to_string()));
//!
//!     let result = table.create_one(data, None).await.unwrap();
//!     println!("Created record: {:?}", result);
//! }
//! ```

use std::{error::Error, fs::File};

use api::{
    authentication::{LoginRequest, TokenResponse, User},
    client::BaserowClient,
};
use error::{FileUploadError, TokenAuthError};
use mapper::TableMapper;
use reqwest::{
    header::AUTHORIZATION,
    multipart::{self, Form},
    Body, Client, StatusCode,
};
use serde::{Deserialize, Serialize};
use tokio_util::codec::{BytesCodec, FramedRead};

pub mod api;

#[macro_use]
extern crate async_trait;

pub mod error;
pub mod filter;
pub mod mapper;

/// Configuration for the Baserow client
///
/// This struct holds all the configuration options needed to connect to a Baserow instance,
/// including authentication credentials and API endpoints.
#[derive(Clone, Debug)]
pub struct Configuration {
    base_url: String,

    email: Option<String>,
    password: Option<String>,
    jwt: Option<String>,

    database_token: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,

    user: Option<User>,
}

/// Builder for creating Configuration instances
///
/// Provides a fluent interface for constructing Configuration objects with the required parameters.
///
/// # Example
/// ```
/// use baserow_rs::ConfigBuilder;
///
/// let config = ConfigBuilder::new()
///     .base_url("https://api.baserow.io")
///     .api_key("your-api-key")
///     .build();
/// ```
#[derive(Default)]
pub struct ConfigBuilder {
    base_url: Option<String>,
    api_key: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            base_url: None,
            api_key: None,
            email: None,
            password: None,
        }
    }

    pub fn base_url(mut self, base_url: &str) -> Self {
        self.base_url = Some(base_url.to_string());
        self
    }

    pub fn api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    pub fn email(mut self, email: &str) -> Self {
        self.email = Some(email.to_string());
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }

    pub fn build(self) -> Configuration {
        Configuration {
            base_url: self.base_url.unwrap(),

            email: self.email,
            password: self.password,
            jwt: None,

            database_token: self.api_key,
            access_token: None,
            refresh_token: None,

            user: None,
        }
    }
}

/// Main client for interacting with the Baserow API
///
/// This struct implements the BaserowClient trait and provides methods for all API operations.
/// It handles authentication, request signing, and maintains the client state.
#[derive(Clone, Debug)]
pub struct Baserow {
    configuration: Configuration,
    client: Client,
}

impl Baserow {
    pub fn with_configuration(configuration: Configuration) -> Self {
        Self {
            configuration,
            client: Client::new(),
        }
    }

    pub fn with_database_token(self, token: String) -> Self {
        let mut configuration = self.configuration.clone();
        configuration.database_token = Some(token);

        Self {
            configuration,
            client: self.client,
        }
    }

    fn with_access_token(&self, access_token: String) -> Self {
        let mut configuration = self.configuration.clone();
        configuration.access_token = Some(access_token);

        Self {
            configuration,
            client: self.client.clone(),
        }
    }

    fn with_refresh_token(&self, refresh_token: String) -> Self {
        let mut configuration = self.configuration.clone();
        configuration.refresh_token = Some(refresh_token);

        Self {
            configuration,
            client: self.client.clone(),
        }
    }

    fn with_user(&self, user: User) -> Self {
        let mut configuration = self.configuration.clone();
        configuration.user = Some(user);

        Self {
            configuration,
            client: self.client.clone(),
        }
    }
}

#[async_trait]
impl BaserowClient for Baserow {
    fn get_configuration(&self) -> Configuration {
        self.configuration.clone()
    }

    fn get_client(&self) -> Client {
        self.client.clone()
    }

    async fn token_auth(&self) -> Result<Box<dyn BaserowClient>, TokenAuthError> {
        let url = format!("{}/api/user/token-auth/", &self.configuration.base_url);

        let email = self
            .configuration
            .email
            .as_ref()
            .ok_or(TokenAuthError::MissingCredentials("email"))?;

        let password = self
            .configuration
            .password
            .as_ref()
            .ok_or(TokenAuthError::MissingCredentials("password"))?;

        let auth_request = LoginRequest {
            email: email.clone(),
            password: password.clone(),
        };

        let req = Client::new().post(url).json(&auth_request);

        let resp = req.send().await?;

        match resp.status() {
            StatusCode::OK => {
                let token_response: TokenResponse = resp.json().await?;
                let client = self
                    .clone()
                    .with_database_token(token_response.token)
                    .with_access_token(token_response.access_token)
                    .with_refresh_token(token_response.refresh_token)
                    .with_user(token_response.user);
                Ok(Box::new(client) as Box<dyn BaserowClient>)
            }
            _ => Err(TokenAuthError::AuthenticationFailed(resp.text().await?)),
        }
    }

    async fn table_fields(&self, table_id: u64) -> Result<Vec<TableField>, Box<dyn Error>> {
        let url = format!(
            "{}/api/database/fields/table/{}/",
            &self.configuration.base_url, table_id
        );

        let mut req = self.client.get(url);

        if let Some(token) = &self.configuration.jwt {
            req = req.header(AUTHORIZATION, format!("JWT {}", token));
        } else if let Some(token) = &self.configuration.database_token {
            req = req.header(AUTHORIZATION, format!("Token {}", token));
        } else {
            return Err("No authentication token provided".into());
        }

        let resp = req.send().await?;
        match resp.status() {
            StatusCode::OK => {
                let fields: Vec<TableField> = resp.json().await?;
                Ok(fields)
            }
            status => {
                let error_text = resp.text().await?;
                Err(format!(
                    "Failed to retrieve table fields (status: {}): {}",
                    status, error_text
                )
                .into())
            }
        }
    }

    fn table_by_id(&self, id: u64) -> BaserowTable {
        BaserowTable::default()
            .with_id(id)
            .with_baserow(self.clone())
    }

    async fn upload_file(
        &self,
        file: File,
        filename: String,
    ) -> Result<api::file::File, FileUploadError> {
        let url = format!(
            "{}/api/user-files/upload-file/",
            &self.configuration.base_url
        );

        let file = tokio::fs::File::from_std(file);
        let stream = FramedRead::new(file, BytesCodec::new());
        let file_body = Body::wrap_stream(stream);

        let mime_type = mime_guess::from_path(&filename).first_or_octet_stream();

        let file_part = multipart::Part::stream(file_body)
            .file_name(filename)
            .mime_str(mime_type.as_ref())?;

        let form = Form::new().part("file", file_part);

        let mut req = self.client.post(url);

        if let Some(token) = &self.configuration.jwt {
            req = req.header(AUTHORIZATION, format!("JWT {}", token));
        } else if let Some(api_key) = &self.configuration.database_token {
            req = req.header(AUTHORIZATION, format!("Token {}", api_key));
        }

        let resp = req.multipart(form).send().await;

        match resp {
            Ok(resp) => match resp.status() {
                StatusCode::OK => {
                    let json: api::file::File = resp.json().await?;
                    Ok(json)
                }
                _ => Err(FileUploadError::UnexpectedStatusCode(resp.status())),
            },
            Err(e) => Err(FileUploadError::UploadError(e)),
        }
    }

    async fn upload_file_via_url(&self, url: &str) -> Result<api::file::File, FileUploadError> {
        // Validate URL format and scheme
        let file_url = url
            .parse::<reqwest::Url>()
            .map_err(|_| FileUploadError::InvalidURL(url.to_string()))?
            .to_string();

        let upload_request = api::file::UploadFileViaUrlRequest {
            url: file_url.clone(),
        };

        let url = format!(
            "{}/api/user-files/upload-via-url/",
            &self.configuration.base_url
        );

        let mut req = self.client.post(url).json(&upload_request);

        if let Some(token) = &self.configuration.jwt {
            req = req.header(AUTHORIZATION, format!("JWT {}", token));
        } else if let Some(api_key) = &self.configuration.database_token {
            req = req.header(AUTHORIZATION, format!("Token {}", api_key));
        }

        let resp = req.send().await;

        match resp {
            Ok(resp) => match resp.status() {
                StatusCode::OK => {
                    let json: api::file::File = resp.json().await?;
                    Ok(json)
                }
                _ => Err(FileUploadError::UnexpectedStatusCode(resp.status())),
            },
            Err(e) => Err(FileUploadError::UploadError(e)),
        }
    }
}

/// Represents a table in Baserow
///
/// This struct provides methods for interacting with a specific table, including
/// creating, reading, updating and deleting records.
#[derive(Deserialize, Serialize, Default, Clone)]
pub struct BaserowTable {
    #[serde(skip)]
    baserow: Option<Baserow>,

    #[serde(skip)]
    mapper: Option<TableMapper>,

    id: Option<u64>,
    pub name: Option<String>,
    order: Option<i64>,
    database_id: Option<i64>,
}

impl BaserowTable {
    fn with_baserow(mut self, baserow: Baserow) -> BaserowTable {
        self.baserow = Some(baserow);
        self
    }

    fn with_id(mut self, id: u64) -> BaserowTable {
        self.id = Some(id);
        self
    }
}

pub use api::table_operations::BaserowTableOperations;

/// Represents a field in a Baserow table
///
/// Contains metadata about a table column including its type, name, and other attributes.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TableField {
    pub id: u64,
    pub table_id: u64,
    pub name: String,
    pub order: u32,
    pub r#type: String,
    pub primary: bool,
    pub read_only: bool,
    pub description: Option<String>,
}

/// Specifies the sort direction for table queries
///
/// Used when ordering table results to determine ascending or descending order.
#[derive(Clone, Debug)]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn test() {
        let configuration = Configuration {
            base_url: "https://baserow.io".to_string(),
            database_token: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
            access_token: None,
            refresh_token: None,
            user: None,
        };
        let baserow = Baserow::with_configuration(configuration);
        let _table = baserow.table_by_id(1234);
    }

    #[tokio::test]
    async fn test_create_record() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("POST", "/api/database/rows/table/1234/")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_header(AUTHORIZATION, format!("Token {}", "123").as_str())
            .with_body(r#"{"id": 1234, "field_1": "test"}"#)
            .create();

        let configuration = Configuration {
            base_url: mock_url,
            database_token: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
            access_token: None,
            refresh_token: None,
            user: None,
        };
        let baserow = Baserow::with_configuration(configuration);
        let table = baserow.table_by_id(1234);

        let mut record = HashMap::new();
        record.insert("field_1".to_string(), Value::String("test".to_string()));

        let result = table.create_one(record, None).await;
        assert!(result.is_ok());

        let created_record = result.unwrap();
        assert_eq!(created_record["field_1"], Value::String("test".to_string()));

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_record() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("GET", "/api/database/rows/table/1234/5678/")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_header(AUTHORIZATION, format!("Token {}", "123").as_str())
            .with_body(r#"{"id": 5678, "field_1": "test"}"#)
            .create();

        let configuration = Configuration {
            base_url: mock_url,
            database_token: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
            access_token: None,
            refresh_token: None,
            user: None,
        };
        let baserow = Baserow::with_configuration(configuration);
        let table = baserow.table_by_id(1234);

        let result: Result<HashMap<String, Value>, Box<dyn Error>> =
            table.get_one(5678, None).await;
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record["id"], Value::Number(5678.into()));
        assert_eq!(record["field_1"], Value::String("test".to_string()));

        mock.assert();
    }

    #[tokio::test]
    async fn test_update_record() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("PATCH", "/api/database/rows/table/1234/5678/")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_header(AUTHORIZATION, format!("Token {}", "123").as_str())
            .with_body(r#"{"id": 5678, "field_1": "updated"}"#)
            .create();

        let configuration = Configuration {
            base_url: mock_url,
            database_token: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
            access_token: None,
            refresh_token: None,
            user: None,
        };
        let baserow = Baserow::with_configuration(configuration);
        let table = baserow.table_by_id(1234);

        let mut record = HashMap::new();
        record.insert("field_1".to_string(), Value::String("updated".to_string()));

        let result = table.update(5678, record, None).await;
        assert!(result.is_ok());

        let updated_record = result.unwrap();
        assert_eq!(
            updated_record["field_1"],
            Value::String("updated".to_string())
        );

        mock.assert();
    }

    #[tokio::test]
    async fn test_delete_record() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("DELETE", "/api/database/rows/table/1234/5678/")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_header(AUTHORIZATION, format!("Token {}", "123").as_str())
            .create();

        let configuration = Configuration {
            base_url: mock_url,
            database_token: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
            access_token: None,
            refresh_token: None,
            user: None,
        };
        let baserow = Baserow::with_configuration(configuration);
        let table = baserow.table_by_id(1234);

        let result = table.delete(5678).await;
        assert!(result.is_ok());

        mock.assert();
    }

    #[tokio::test]
    async fn test_upload_file_via_url() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("POST", "/api/user-files/upload-via-url/")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_header(AUTHORIZATION, format!("Token {}", "123").as_str())
            .with_body(r#"{
    "url": "https://files.baserow.io/user_files/VXotniBOVm8tbstZkKsMKbj2Qg7KmPvn_39d354a76abe56baaf569ad87d0333f58ee4bf3eed368e3b9dc736fd18b09dfd.png",
    "thumbnails": {
        "tiny": {
            "url": "https://files.baserow.io/media/thumbnails/tiny/VXotniBOVm8tbstZkKsMKbj2Qg7KmPvn_39d354a76abe56baaf569ad87d0333f58ee4bf3eed368e3b9dc736fd18b09dfd.png",
            "width": 21,
            "height": 21
        },
        "small": {
            "url": "https://files.baserow.io/media/thumbnails/small/VXotniBOVm8tbstZkKsMKbj2Qg7KmPvn_39d354a76abe56baaf569ad87d0333f58ee4bf3eed368e3b9dc736fd18b09dfd.png",
            "width": 48,
            "height": 48
        }
    },
    "name": "VXotniBOVm8tbstZkKsMKbj2Qg7KmPvn_39d354a76abe56baaf569ad87d0333f58ee4bf3eed368e3b9dc736fd18b09dfd.png",
    "size": 229940,
    "mime_type": "image/png",
    "is_image": true,
    "image_width": 1280,
    "image_height": 585,
    "uploaded_at": "2020-11-17T12:16:10.035234+00:00"
}"#)
            .create();

        let configuration = Configuration {
            base_url: mock_url,
            database_token: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
            access_token: None,
            refresh_token: None,
            user: None,
        };
        let baserow = Baserow::with_configuration(configuration);

        let result = baserow
            .upload_file_via_url("https://example.com/test.txt")
            .await;
        assert!(result.is_ok());

        let uploaded_file = result.unwrap();
        assert_eq!(uploaded_file.name, "VXotniBOVm8tbstZkKsMKbj2Qg7KmPvn_39d354a76abe56baaf569ad87d0333f58ee4bf3eed368e3b9dc736fd18b09dfd.png".to_string());

        mock.assert();
    }

    #[tokio::test]
    async fn test_token_auth() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("POST", "/api/user/token-auth/")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"{
  "user": {
    "first_name": "string",
    "username": "user@example.com",
    "language": "string"
  },
  "token": "string",
  "access_token": "string",
  "refresh_token": "string"
}"#,
            )
            .create();

        let configuration = ConfigBuilder::new()
            .base_url(&mock_url)
            .email("test@example.com")
            .password("password")
            .build();
        let baserow = Baserow::with_configuration(configuration);

        let result = baserow.token_auth().await;
        assert!(result.is_ok());

        let logged_in_baserow = result.unwrap();
        assert_eq!(
            logged_in_baserow
                .get_configuration()
                .database_token
                .unwrap(),
            "string"
        );

        mock.assert();
    }

    #[tokio::test]
    async fn test_upload_file() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("POST", "/api/user-files/upload-file/")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_header(AUTHORIZATION, format!("Token {}", "123").as_str())
            .with_body(r#"{
    "url": "https://files.baserow.io/user_files/VXotniBOVm8tbstZkKsMKbj2Qg7KmPvn_39d354a76abe56baaf569ad87d0333f58ee4bf3eed368e3b9dc736fd18b09dfd.png",
    "thumbnails": {
        "tiny": {
            "url": "https://files.baserow.io/media/thumbnails/tiny/VXotniBOVm8tbstZkKsMKbj2Qg7KmPvn_39d354a76abe56baaf569ad87d0333f58ee4bf3eed368e3b9dc736fd18b09dfd.png",
            "width": 21,
            "height": 21
        },
        "small": {
            "url": "https://files.baserow.io/media/thumbnails/small/VXotniBOVm8tbstZkKsMKbj2Qg7KmPvn_39d354a76abe56baaf569ad87d0333f58ee4bf3eed368e3b9dc736fd18b09dfd.png",
            "width": 48,
            "height": 48
        }
    },
    "name": "VXotniBOVm8tbstZkKsMKbj2Qg7KmPvn_39d354a76abe56baaf569ad87d0333f58ee4bf3eed368e3b9dc736fd18b09dfd.png",
    "size": 229940,
    "mime_type": "image/png",
    "is_image": true,
    "image_width": 1280,
    "image_height": 585,
    "uploaded_at": "2020-11-17T12:16:10.035234+00:00"
}"#)
            .create();

        let configuration = Configuration {
            base_url: mock_url,
            database_token: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
            access_token: None,
            refresh_token: None,
            user: None,
        };
        let baserow = Baserow::with_configuration(configuration);

        let file = File::open(".gitignore").unwrap();
        let result = baserow.upload_file(file, "image.png".to_string()).await;
        assert!(result.is_ok());

        let uploaded_file = result.unwrap();
        assert_eq!(uploaded_file.name, "VXotniBOVm8tbstZkKsMKbj2Qg7KmPvn_39d354a76abe56baaf569ad87d0333f58ee4bf3eed368e3b9dc736fd18b09dfd.png".to_string());

        mock.assert();
    }

    #[tokio::test]
    async fn test_view_query() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("GET", "/api/database/rows/table/1234/")
            .match_query(mockito::Matcher::UrlEncoded("view_id".into(), "5678".into()))
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_header(AUTHORIZATION, format!("Token {}", "123").as_str())
            .with_body(r#"{"count": 1, "next": null, "previous": null, "results": [{"id": 1, "field_1": "test"}]}"#)
            .create();

        let configuration = Configuration {
            base_url: mock_url,
            database_token: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
            access_token: None,
            refresh_token: None,
            user: None,
        };
        let baserow = Baserow::with_configuration(configuration);
        let table = baserow.table_by_id(1234);

        let result = table
            .query()
            .view(5678)
            .get::<HashMap<String, Value>>()
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.count, 1);
        assert_eq!(
            response.results[0]["field_1"],
            Value::String("test".to_string())
        );

        mock.assert();
    }

    #[tokio::test]
    async fn test_view_query_with_filters() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("GET", "/api/database/rows/table/1234/")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("view_id".into(), "5678".into()),
                mockito::Matcher::UrlEncoded("filter__field_1__equal".into(), "test".into()),
                mockito::Matcher::UrlEncoded("order_by".into(), "field_1".into()),
            ]))
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_header(AUTHORIZATION, format!("Token {}", "123").as_str())
            .with_body(r#"{"count": 1, "next": null, "previous": null, "results": [{"id": 1, "field_1": "test"}]}"#)
            .create();

        let configuration = Configuration {
            base_url: mock_url,
            database_token: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
            access_token: None,
            refresh_token: None,
            user: None,
        };
        let baserow = Baserow::with_configuration(configuration);
        let table = baserow.table_by_id(1234);

        let result = table
            .query()
            .view(5678)
            .filter_by("field_1", filter::Filter::Equal, "test")
            .order_by("field_1", OrderDirection::Asc)
            .get::<HashMap<String, Value>>()
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.count, 1);
        assert_eq!(
            response.results[0]["field_1"],
            Value::String("test".to_string())
        );

        mock.assert();
    }

    #[tokio::test]
    async fn test_view_query_with_pagination() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("GET", "/api/database/rows/table/1234/")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("view_id".into(), "5678".into()),
                mockito::Matcher::UrlEncoded("size".into(), "2".into()),
                mockito::Matcher::UrlEncoded("offset".into(), "1".into()),
            ]))
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_header(AUTHORIZATION, format!("Token {}", "123").as_str())
            .with_body(r#"{"count": 3, "next": "http://example.com/next", "previous": "http://example.com/prev", "results": [{"id": 2, "field_1": "test2"}, {"id": 3, "field_1": "test3"}]}"#)
            .create();

        let configuration = Configuration {
            base_url: mock_url,
            database_token: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
            access_token: None,
            refresh_token: None,
            user: None,
        };
        let baserow = Baserow::with_configuration(configuration);
        let table = baserow.table_by_id(1234);

        let result = table
            .query()
            .view(5678)
            .page_size(2)
            .offset(1)
            .get::<HashMap<String, Value>>()
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.count, 3);
        assert_eq!(response.next, Some("http://example.com/next".to_string()));
        assert_eq!(
            response.previous,
            Some("http://example.com/prev".to_string())
        );
        assert_eq!(response.results.len(), 2);
        assert_eq!(
            response.results[0]["field_1"],
            Value::String("test2".to_string())
        );
        assert_eq!(
            response.results[1]["field_1"],
            Value::String("test3".to_string())
        );

        mock.assert();
    }

    #[tokio::test]
    async fn test_query_with_user_field_names() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("GET", "/api/database/rows/table/1234/")
            .match_query(mockito::Matcher::UrlEncoded(
                "user_field_names".into(),
                "true".into(),
            ))
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_header(AUTHORIZATION, format!("Token {}", "123").as_str())
            .with_body(r#"{"count": 1, "next": null, "previous": null, "results": [{"User Name": "test"}]}"#)
            .create();

        let configuration = Configuration {
            base_url: mock_url,
            database_token: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
            access_token: None,
            refresh_token: None,
            user: None,
        };
        let baserow = Baserow::with_configuration(configuration);
        let table = baserow.table_by_id(1234);

        let result = table
            .query()
            .user_field_names(true)
            .get::<HashMap<String, Value>>()
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.count, 1);
        assert_eq!(
            response.results[0]["User Name"],
            Value::String("test".to_string())
        );

        mock.assert();
    }

    #[tokio::test]
    async fn test_view_query_invalid_view() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("GET", "/api/database/rows/table/1234/")
            .match_query(mockito::Matcher::UrlEncoded(
                "view_id".into(),
                "9999".into(),
            ))
            .with_status(404)
            .with_header("Content-Type", "application/json")
            .with_header(AUTHORIZATION, format!("Token {}", "123").as_str())
            .with_body(r#"{"error": "View does not exist."}"#)
            .create();

        let configuration = Configuration {
            base_url: mock_url,
            database_token: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
            access_token: None,
            refresh_token: None,
            user: None,
        };
        let baserow = Baserow::with_configuration(configuration);
        let table = baserow.table_by_id(1234);

        let result = table
            .query()
            .view(9999)
            .get::<HashMap<String, Value>>()
            .await;

        assert!(result.is_err());
        mock.assert();
    }

    #[tokio::test]
    async fn test_table_fields() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("GET", "/api/database/fields/table/1234/")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_header(AUTHORIZATION, format!("Token {}", "123").as_str())
            .with_body(
                r#"[
    {
        "id": 1529,
        "table_id": 1234,
        "name": "Name",
        "order": 0,
        "type": "text",
        "primary": true,
        "read_only": false,
        "description": "A sample description"
    },
    {
        "id": 6499,
        "table_id": 1234,
        "name": "Field 2",
        "order": 1,
        "type": "last_modified",
        "primary": false,
        "read_only": true,
        "description": "A sample description"
    },
    {
        "id": 6500,
        "table_id": 1234,
        "name": "Datei",
        "order": 2,
        "type": "file",
        "primary": false,
        "read_only": false,
        "description": "A sample description"
    }
]"#,
            )
            .create();

        let configuration = Configuration {
            base_url: mock_url,
            database_token: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
            access_token: None,
            refresh_token: None,
            user: None,
        };
        let baserow = Baserow::with_configuration(configuration);

        let result = baserow.table_fields(1234).await;

        print!("result: {:#?}", result);

        assert!(result.is_ok());

        let fields = result.unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].id, 1529);
        assert_eq!(fields[0].table_id, 1234);
        assert_eq!(fields[0].name, "Name");
        assert_eq!(fields[1].id, 6499);
        assert_eq!(fields[1].table_id, 1234);
        assert_eq!(fields[1].name, "Field 2");

        mock.assert();
    }
}
