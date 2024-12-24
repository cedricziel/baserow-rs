/// Filter operations available for querying Baserow tables
///
/// This enum provides all the possible filter operations that can be used
/// when querying table rows. The filters are grouped by type (text, date, number, etc.)
/// and provide rich comparison capabilities.
///
/// # Example
/// ```no_run
/// use baserow_rs::{ConfigBuilder, Baserow, BaserowTableOperations, filter::Filter, api::client::BaserowClient};
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
///     // Query with multiple filters
///     let results = table.rows()
///         .filter_by("Status", Filter::Equal, "Active")
///         .filter_by("Age", Filter::HigherThan, "18")
///         .filter_by("Name", Filter::Contains, "John")
///         .get()
///         .await
///         .unwrap();
/// }
/// ```
#[derive(Debug)]
pub enum Filter {
    /// Exact match comparison
    /// Field value must exactly match the provided value
    /// Example: `.filter_by("Status", Filter::Equal, "Active")`
    Equal,
    /// Inverse of Equal
    /// Field value must not match the provided value
    NotEqual,

    // Date Filters
    /// Date field matches exact date
    DateIs,
    /// Date field does not match exact date
    DateIsNot,
    /// Date field is before specified date
    DateIsBefore,
    /// Date field is on or before specified date
    DateIsOnOrBefore,
    /// Date field is after specified date
    DateIsAfter,
    /// Date field is on or after specified date
    DateIsOnOrAfter,
    /// Date field is within specified range
    DateIsWithin,
    /// Legacy exact date match
    DateEqual,
    /// Legacy date not equal
    DateNotEqual,
    /// Date field is today
    DateEqualsToday,
    /// Date field is before today
    DateBeforeToday,
    /// Date field is after today
    DateAfterToday,
    /// Date is within specified number of days
    DateWithinDays,
    /// Date is within specified number of weeks
    DateWithinWeeks,
    /// Date is within specified number of months
    DateWithinMonths,
    /// Date is exactly specified number of days ago
    DateEqualsDaysAgo,
    /// Date is exactly specified number of months ago
    DateEqualsMonthsAgo,
    /// Date is exactly specified number of years ago
    DateEqualsYearsAgo,
    /// Date is in specified week
    DateEqualsWeek,
    /// Date is in specified month
    DateEqualsMonth,
    /// Date is in specified year
    DateEqualsYear,
    /// Date matches specific day of month
    DateEqualsDayOfMonth,
    /// Legacy before date
    DateBefore,
    /// Legacy before or equal date
    DateBeforeOrEqual,
    /// Legacy after date
    DateAfter,
    /// Legacy after or equal date
    DateAfterOrEqual,
    /// Date is after specified number of days ago
    DateAfterDaysAgo,

    // Value Presence Filters
    /// Field has empty value
    HasEmptyValue,
    /// Field has non-empty value
    HasNotEmptyValue,
    /// Field has specific value
    HasValueEqual,
    /// Field does not have specific value
    HasNotValueEqual,
    /// Field value contains substring
    HasValueContains,
    /// Field value does not contain substring
    HasNotValueContains,
    /// Field value contains specific word
    HasValueContainsWord,
    /// Field value does not contain specific word
    HasNotValueContainsWord,
    /// Field value length is less than specified
    HasValueLengthIsLowerThan,
    /// All values in field equal specified value
    HasAllValuesEqual,
    /// Any select option matches value
    HasAnySelectOptionEqual,
    /// No select option matches value
    HasNoneSelectOptionEqual,

    // Text Filters
    /// Text contains substring
    /// Example: `.filter_by("Name", Filter::Contains, "John")`
    Contains,
    /// Text does not contain substring
    ContainsNot,
    /// Text contains whole word
    ContainsWord,
    /// Text does not contain whole word
    DoesntContainWord,

    // File Filters
    /// Filename contains substring
    FilenameContains,
    /// File is of specified type
    HasFileType,
    /// Number of files is less than specified
    FilesLowerThan,
    /// Text length is less than specified
    LengthIsLowerThan,

    // Numeric Filters
    /// Number is greater than value
    /// Example: `.filter_by("Age", Filter::HigherThan, "18")`
    HigherThan,
    /// Number is greater than or equal to value
    HigherThanOrEqual,
    /// Number is less than value
    LowerThan,
    /// Number is less than or equal to value
    LowerThanOrEqual,
    /// Number is even and whole
    IsEvenAndWhole,

    // Select Filters
    /// Single select matches value
    SingleSelectEqual,
    /// Single select does not match value
    SingleSelectNotEqual,
    /// Single select matches any of values
    SingleSelectIsAnyOf,
    /// Single select matches none of values
    SingleSelectIsNoneOf,
    /// Boolean field matches value
    Boolean,

    // Link Row Filters
    /// Linked row exists
    LinkRowHas,
    /// Linked row does not exist
    LinkRowHasNot,
    /// Linked row contains value
    LinkRowContains,
    /// Linked row does not contain value
    LinkRowNotContains,

    // Multiple Select Filters
    /// Multiple select includes value
    MultipleSelectHas,
    /// Multiple select does not include value
    MultipleSelectHasNot,
    /// Multiple collaborators includes user
    MultipleCollaboratorsHas,
    /// Multiple collaborators does not include user
    MultipleCollaboratorsHasNot,

    // Null Filters
    /// Field is empty/null
    Empty,
    /// Field is not empty/null
    NotEmpty,

    // User Filters
    /// Field matches specific user
    UserIs,
    /// Field does not match specific user
    UserIsNot,
}

impl Filter {
    /// Converts the filter to its string representation for API requests
    pub fn as_str(&self) -> &'static str {
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

/// Internal structure for representing a filter condition
///
/// Combines a field name, filter operation, and value into a single filter condition
/// that can be applied to a table query.
pub struct FilterTriple {
    /// The name of the field to filter on
    pub field: String,
    /// The filter operation to apply
    pub filter: Filter,
    /// The value to compare against
    pub value: String,
}
