use std::{collections::HashMap, error::Error, fs::File};

use api::{
    authentication::{LoginRequest, TokenAuthErrorResponse, TokenResponse, User},
    table::RowRequestBuilder,
};
use reqwest::{header::AUTHORIZATION, multipart, Body, Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_util::codec::{BytesCodec, FramedRead};

mod api;

pub mod error;

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
}

impl Baserow {
    pub fn with_configuration(configuration: Configuration) -> Self {
        Self { configuration }
    }

    pub fn with_database_token(self, token: String) -> Self {
        let mut configuration = self.configuration.clone();
        configuration.database_token = Some(token);

        Self { configuration }
    }

    fn with_access_token(&self, access_token: String) -> Self {
        let mut configuration = self.configuration.clone();
        configuration.access_token = Some(access_token);

        Self { configuration }
    }

    fn with_refresh_token(&self, refresh_token: String) -> Self {
        let mut configuration = self.configuration.clone();
        configuration.refresh_token = Some(refresh_token);

        Self { configuration }
    }

    fn with_user(&self, user: User) -> Self {
        let mut configuration = self.configuration.clone();
        configuration.user = Some(user);

        Self { configuration }
    }

    /// Authenticates an existing user based on their email and their password.
    /// If successful, an access token and a refresh token will be returned.
    pub async fn token_auth(&self) -> Result<Baserow, Box<dyn Error>> {
        let url = format!("{}/api/user/token-auth/", &self.configuration.base_url);

        let auth_request = LoginRequest {
            email: self.configuration.email.clone().unwrap(),
            password: self.configuration.password.clone().unwrap(),
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
            StatusCode::UNAUTHORIZED => Err(Box::new(resp.error_for_status().unwrap_err())),
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }

    // Returns a table by its ID.
    pub fn table_by_id(&self, id: u64) -> BaserowTable {
        BaserowTable::default()
            .with_id(id)
            .with_baserow(self.clone())
    }

    pub async fn upload_file(&self, file: File) -> Result<api::file::File, Box<dyn Error>> {
        let url = format!(
            "{}/api/user-files/upload-file/",
            &self.configuration.base_url
        );

        let file = tokio::fs::File::from_std(file);
        let stream = FramedRead::new(file, BytesCodec::new());
        let file_body = Body::wrap_stream(stream);

        let file_part = multipart::Part::stream(file_body)
            .file_name("gitignore.txt")
            .mime_str("text/plain")?;

        let form = reqwest::multipart::Form::new().part("file", file_part);

        let mut req = Client::new().post(url);

        if let Some(token) = &self.configuration.jwt {
            req = req.header(AUTHORIZATION, format!("JWT {}", token));
        } else if let Some(api_key) = &self.configuration.database_token {
            req = req.header(AUTHORIZATION, format!("Token {}", api_key));
        }

        let resp = req.multipart(form).send().await?;

        match resp.status() {
            StatusCode::OK => {
                let json: api::file::File = resp.json().await?;
                Ok(json)
            }
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }

    pub async fn upload_file_via_url(&self, url: &str) -> Result<api::file::File, Box<dyn Error>> {
        let file_url = url.to_string();

        let upload_request = api::file::UploadFileViaUrlRequest {
            url: file_url.clone(),
        };

        let url = format!(
            "{}/api/user-files/upload-via-url/",
            &self.configuration.base_url
        );

        let mut req = Client::new().post(url).json(&upload_request);

        if let Some(token) = &self.configuration.jwt {
            req = req.header(AUTHORIZATION, format!("JWT {}", token));
        } else if let Some(api_key) = &self.configuration.database_token {
            req = req.header(AUTHORIZATION, format!("Token {}", api_key));
        }

        let resp = req.send().await?;

        match resp.status() {
            StatusCode::OK => {
                let json: api::file::File = resp.json().await?;
                Ok(json)
            }
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct BaserowTable {
    #[serde(skip)]
    baserow: Option<Baserow>,

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

        let mut req = Client::new().post(url);

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

        let mut req = Client::new().get(url);

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

        let mut req = Client::new().patch(url);

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

        let mut req = Client::new().delete(url);

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

pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Debug)]
pub enum Filter {
    Equal,
    NotEqual,
    DateIs,
    DateIsNot,
    DateIsBefore,
    DateIsOnOrBefore,
    DateIsAfter,
    DateIsOnOrAfter,
    DateIsWithin,
    DateEqual,
    DateNotEqual,
    DateEqualsToday,
    DateBeforeToday,
    DateAfterToday,
    DateWithinDays,
    DateWithinWeeks,
    DateWithinMonths,
    DateEqualsDaysAgo,
    DateEqualsMonthsAgo,
    DateEqualsYearsAgo,
    DateEqualsWeek,
    DateEqualsMonth,
    DateEqualsYear,
    DateEqualsDayOfMonth,
    DateBefore,
    DateBeforeOrEqual,
    DateAfter,
    DateAfterOrEqual,
    DateAfterDaysAgo,
    HasEmptyValue,
    HasNotEmptyValue,
    HasValueEqual,
    HasNotValueEqual,
    HasValueContains,
    HasNotValueContains,
    HasValueContainsWord,
    HasNotValueContainsWord,
    HasValueLengthIsLowerThan,
    HasAllValuesEqual,
    HasAnySelectOptionEqual,
    HasNoneSelectOptionEqual,
    Contains,
    ContainsNot,
    ContainsWord,
    DoesntContainWord,
    FilenameContains,
    HasFileType,
    FilesLowerThan,
    LengthIsLowerThan,
    HigherThan,
    HigherThanOrEqual,
    LowerThan,
    LowerThanOrEqual,
    IsEvenAndWhole,
    SingleSelectEqual,
    SingleSelectNotEqual,
    SingleSelectIsAnyOf,
    SingleSelectIsNoneOf,
    Boolean,
    LinkRowHas,
    LinkRowHasNot,
    LinkRowContains,
    LinkRowNotContains,
    MultipleSelectHas,
    MultipleSelectHasNot,
    MultipleCollaboratorsHas,
    MultipleCollaboratorsHasNot,
    Empty,
    NotEmpty,
    UserIs,
    UserIsNot,
}

impl Filter {
    fn as_str(&self) -> &'static str {
        match self {
            Filter::Equal => "equal",
            Filter::NotEqual => "not_equal",
            Filter::DateIs => "date_is",
            Filter::DateIsNot => "date_is_not",
            Filter::DateIsBefore => "date_is_before",
            Filter::DateIsOnOrBefore => "date_is_on_or_before",
            Filter::DateIsAfter => "date_is_after",
            Filter::DateIsOnOrAfter => "date_is_on_or_after",
            Filter::DateIsWithin => "date_is_within",
            Filter::DateEqual => "date_equal",
            Filter::DateNotEqual => "date_not_equal",
            Filter::DateEqualsToday => "date_equals_today",
            Filter::DateBeforeToday => "date_before_today",
            Filter::DateAfterToday => "date_after_today",
            Filter::DateWithinDays => "date_within_days",
            Filter::DateWithinWeeks => "date_within_weeks",
            Filter::DateWithinMonths => "date_within_months",
            Filter::DateEqualsDaysAgo => "date_equals_days_ago",
            Filter::DateEqualsMonthsAgo => "date_equals_months_ago",
            Filter::DateEqualsYearsAgo => "date_equals_years_ago",
            Filter::DateEqualsWeek => "date_equals_week",
            Filter::DateEqualsMonth => "date_equals_month",
            Filter::DateEqualsYear => "date_equals_year",
            Filter::DateEqualsDayOfMonth => "date_equals_day_of_month",
            Filter::DateBefore => "date_before",
            Filter::DateBeforeOrEqual => "date_before_or_equal",
            Filter::DateAfter => "date_after",
            Filter::DateAfterOrEqual => "date_after_or_equal",
            Filter::DateAfterDaysAgo => "date_after_days_ago",
            Filter::HasEmptyValue => "has_empty_value",
            Filter::HasNotEmptyValue => "has_not_empty_value",
            Filter::HasValueEqual => "has_value_equal",
            Filter::HasNotValueEqual => "has_not_value_equal",
            Filter::HasValueContains => "has_value_contains",
            Filter::HasNotValueContains => "has_not_value_contains",
            Filter::HasValueContainsWord => "has_value_contains_word",
            Filter::HasNotValueContainsWord => "has_not_value_contains_word",
            Filter::HasValueLengthIsLowerThan => "has_value_length_is_lower_than",
            Filter::HasAllValuesEqual => "has_all_values_equal",
            Filter::HasAnySelectOptionEqual => "has_any_select_option_equal",
            Filter::HasNoneSelectOptionEqual => "has_none_select_option_equal",
            Filter::Contains => "contains",
            Filter::ContainsNot => "contains_not",
            Filter::ContainsWord => "contains_word",
            Filter::DoesntContainWord => "doesnt_contain_word",
            Filter::FilenameContains => "filename_contains",
            Filter::HasFileType => "has_file_type",
            Filter::FilesLowerThan => "files_lower_than",
            Filter::LengthIsLowerThan => "length_is_lower_than",
            Filter::HigherThan => "higher_than",
            Filter::HigherThanOrEqual => "higher_than_or_equal",
            Filter::LowerThan => "lower_than",
            Filter::LowerThanOrEqual => "lower_than_or_equal",
            Filter::IsEvenAndWhole => "is_even_and_whole",
            Filter::SingleSelectEqual => "single_select_equal",
            Filter::SingleSelectNotEqual => "single_select_not_equal",
            Filter::SingleSelectIsAnyOf => "single_select_is_any_of",
            Filter::SingleSelectIsNoneOf => "single_select_is_none_of",
            Filter::Boolean => "boolean",
            Filter::LinkRowHas => "link_row_has",
            Filter::LinkRowHasNot => "link_row_has_not",
            Filter::LinkRowContains => "link_row_contains",
            Filter::LinkRowNotContains => "link_row_not_contains",
            Filter::MultipleSelectHas => "multiple_select_has",
            Filter::MultipleSelectHasNot => "multiple_select_has_not",
            Filter::MultipleCollaboratorsHas => "multiple_collaborators_has",
            Filter::MultipleCollaboratorsHasNot => "multiple_collaborators_has_not",
            Filter::Empty => "empty",
            Filter::NotEmpty => "not_empty",
            Filter::UserIs => "user_is",
            Filter::UserIsNot => "user_is_not",
        }
    }
}

pub struct FilterTriple {
    field: String,
    filter: Filter,
    value: String,
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
        let result = baserow.upload_file(file).await;
        assert!(result.is_ok());

        let uploaded_file = result.unwrap();
        assert_eq!(uploaded_file.name, "VXotniBOVm8tbstZkKsMKbj2Qg7KmPvn_39d354a76abe56baaf569ad87d0333f58ee4bf3eed368e3b9dc736fd18b09dfd.png".to_string());

        mock.assert();
    }
}
