use std::{collections::HashMap, error::Error, fs::File};

use api::{
    authentication::{LoginRequest, TokenResponse, User},
    table::RowRequestBuilder,
};
use error::{FileUploadError, TokenAuthError};
use mapper::TableMapper;
use reqwest::{
    header::AUTHORIZATION,
    multipart::{self, Form},
    Body, Client, StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_util::codec::{BytesCodec, FramedRead};

mod api;

pub mod error;
pub mod filter;
pub mod mapper;

#[derive(Clone)]
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

#[derive(Default)]
pub struct ConfigBuilder {
    base_url: Option<String>,
    api_key: Option<String>,
    email: Option<String>,
    password: Option<String>,
    jwt: Option<String>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            base_url: None,
            api_key: None,
            email: None,
            password: None,
            jwt: None,
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

#[derive(Clone)]
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

    /// Authenticates an existing user based on their email and their password.
    /// If successful, an access token and a refresh token will be returned.
    pub async fn token_auth(&self) -> Result<Baserow, TokenAuthError> {
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
                Ok(self
                    .clone()
                    .with_database_token(token_response.token)
                    .with_access_token(token_response.access_token)
                    .with_refresh_token(token_response.refresh_token)
                    .with_user(token_response.user))
            }
            _ => Err(TokenAuthError::AuthenticationFailed(resp.text().await?)),
        }
    }

    /// Retrieves all fields for a given table.
    ///
    /// This function sends a GET request to the Baserow API's
    /// `/api/database/fields/table/{table_id}/` endpoint and returns a vector
    /// of `TableField`s.
    ///
    /// If the request is successful (200), the JSON response is parsed into a
    /// vector of `TableField`s and returned. Otherwise, the error message is
    /// returned as a `Box<dyn Error>`.
    pub async fn table_fields(&self, table_id: u64) -> Result<Vec<TableField>, Box<dyn Error>> {
        let url = format!(
            "{}/api/database/fields/table/{}/",
            &self.configuration.base_url, table_id
        );

        let mut req = self.client.get(url);

        if let Some(token) = &self.configuration.jwt {
            req = req.header(AUTHORIZATION, format!("JWT {}", token));
        } else if let Some(database_token) = &self.configuration.database_token {
            req = req.header(AUTHORIZATION, format!("Token {}", database_token));
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

    // Returns a table by its ID.
    pub fn table_by_id(&self, id: u64) -> BaserowTable {
        BaserowTable::default()
            .with_id(id)
            .with_baserow(self.clone())
    }

    pub async fn upload_file(
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

    pub async fn upload_file_via_url(&self, url: &str) -> Result<api::file::File, FileUploadError> {
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
    // data_sync: DataSync,
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

    pub async fn auto_map(mut self) -> Result<BaserowTable, Box<dyn Error>> {
        let id = self.id.ok_or("Table ID is missing")?;

        let baserow = self.baserow.clone().ok_or("Baserow instance is missing")?;
        let fields = baserow.table_fields(id).await?;

        let mapper = TableMapper::new();
        mapper.clone().map_fields(fields.clone());
        self.mapper = Some(mapper);

        Ok(self)
    }

    pub fn rows(self) -> RowRequestBuilder {
        RowRequestBuilder::new()
            .with_baserow(self.baserow.clone().unwrap())
            .with_table(self.clone())
    }

    pub async fn create_one(
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

    pub async fn get_one(self, id: u64) -> Result<HashMap<String, Value>, Box<dyn Error>> {
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

    pub async fn update(
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

    pub async fn delete(self, id: u64) -> Result<(), Box<dyn Error>> {
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

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TableField {
    pub id: u64,
    pub table_id: u64,
    pub name: String,
    pub order: u32,
    pub field_type: String,
    pub primary: bool,
    pub read_only: bool,
    pub immutable_type: bool,
    pub immutable_properties: bool,
    pub description: Option<String>,
    pub text_default: Option<String>,
}

pub enum OrderDirection {
    Asc,
    Desc,
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let result = table.create_one(record).await;
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

        let result = table.get_one(5678).await;
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

        let result = table.update(5678, record).await;
        assert!(result.is_ok());

        let updated_record = result.unwrap();
        assert_eq!(
            updated_record["field_1"],
            Value::String("updated".to_string())
        );

        mock.assert();
    }

    /// Tests the `delete` function of the `BaserowTable` struct to ensure it can delete a record successfully.
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
            logged_in_baserow.configuration.database_token.unwrap(),
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
                            "id": 123,
                            "table_id": 1234,
                            "name": "Field 1",
                            "type": "text",
                            "order": 0
                        },
                        {
                            "id": 456,
                            "table_id": 1234,
                            "name": "Field 2",
                            "type": "text",
                            "order": 1
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

        assert!(result.is_ok());

        let fields = result.unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].id, 123);
        assert_eq!(fields[0].table_id, 1234);
        assert_eq!(fields[0].name, "Field 1");
        assert_eq!(fields[1].id, 456);
        assert_eq!(fields[1].table_id, 1234);
        assert_eq!(fields[1].name, "Field 2");

        mock.assert();
    }
}
