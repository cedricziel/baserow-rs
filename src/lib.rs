use std::{collections::HashMap, error::Error, fs::File};

use api::table::{RowRequestBuilder, RowsResponse};
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod api;

#[derive(Clone)]
pub struct Configuration {
    base_url: String,
    api_key: Option<String>,
    email: Option<String>,
    password: Option<String>,
    jwt: Option<String>,
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
            api_key: self.api_key,

            email: self.email,
            password: self.password,
            jwt: None,
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

    pub fn with_token(self, token: String) -> Self {
        let mut configuration = self.configuration.clone();
        configuration.jwt = Some(token);

        Self { configuration }
    }

    pub async fn login(&self) -> Result<Baserow, Box<dyn Error>> {
        let url = format!("{}/api/auth/token/", &self.configuration.base_url);
        let form = [
            ("email", self.configuration.email.clone().unwrap()),
            ("password", self.configuration.password.clone().unwrap()),
        ];

        let req = reqwest::Client::new().post(url).form(&form);

        let resp = req.send().await?;

        match resp.status() {
            reqwest::StatusCode::OK => {
                let token = resp.text().await?;
                println!("Token: {}", token);
                Ok(self.clone().with_token(token))
            }
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }

    // Returns a table by its ID.
    pub fn table_by_id(&self, id: u64) -> BaserowTable {
        BaserowTable::default()
            .with_id(id)
            .with_baserow(self.clone())
    }

    fn upload_file(&self, file: File) {}

    fn upload_file_via_url(&self, url: &str) {}
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

        let mut req = reqwest::Client::new().post(url);

        if baserow.configuration.jwt.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!("JWT {}", &baserow.configuration.jwt.unwrap()),
            );
        } else if baserow.configuration.api_key.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!("Token {}", &baserow.configuration.api_key.unwrap()),
            );
        }

        let resp = req.json(&data).send().await?;

        match resp.status() {
            reqwest::StatusCode::OK => Ok(resp.json::<HashMap<String, Value>>().await?),
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

        let mut req = reqwest::Client::new().patch(url);

        if baserow.configuration.jwt.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!("JWT {}", &baserow.configuration.jwt.unwrap()),
            );
        } else if baserow.configuration.api_key.is_some() {
            req = req.header(
                AUTHORIZATION,
                format!("Token {}", &baserow.configuration.api_key.unwrap()),
            );
        }

        let resp = req.json(&data).send().await?;

        match resp.status() {
            reqwest::StatusCode::OK => Ok(resp.json::<HashMap<String, Value>>().await?),
            _ => Err(Box::new(resp.error_for_status().unwrap_err())),
        }
    }

    pub async fn delete(self, id: u64) -> Result<RowsResponse, Box<dyn Error>> {
        todo!()
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
    use mockito::Server;

    use super::*;

    #[test]
    fn test() {
        let configuration = Configuration {
            base_url: "https://baserow.io".to_string(),
            api_key: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
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
            api_key: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
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
            api_key: Some("123".to_string()),
            email: None,
            password: None,
            jwt: None,
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
}
