use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, instrument, warn};

use crate::TableField;

/// A trait for mapping between Baserow field IDs and their human-readable names
///
/// This trait provides functionality to map between numeric field IDs used by Baserow
/// and their corresponding human-readable names. This is useful when working with the API
/// where field IDs are required, but you want to reference fields by their names in your code.
pub trait FieldMapper: Clone {
    /// Maps a collection of table fields, building the internal mapping structures
    ///
    /// # Arguments
    /// * `fields` - A vector of TableField structs containing field metadata
    fn map_fields(&mut self, fields: Vec<TableField>);

    /// Gets the field ID corresponding to a field name
    ///
    /// # Arguments
    /// * `name` - The human-readable field name
    ///
    /// # Returns
    /// * `Option<u64>` - The field ID if found, None otherwise
    fn get_field_id(&self, name: &str) -> Option<u64>;

    /// Gets the field name corresponding to a field ID
    ///
    /// # Arguments
    /// * `id` - The numeric field ID
    ///
    /// # Returns
    /// * `Option<String>` - The field name if found, None otherwise
    fn get_field_name(&self, id: u64) -> Option<String>;

    /// Gets all mapped fields
    ///
    /// # Returns
    /// * `Vec<TableField>` - A vector of all mapped table fields
    fn get_fields(&self) -> Vec<TableField>;
}

/// Default implementation of the FieldMapper trait
///
/// Provides bidirectional mapping between field IDs and names using HashMaps
/// for efficient lookups.
#[derive(Clone, Default)]
pub struct TableMapper {
    fields: Vec<TableField>,
    ids_to_names: HashMap<u64, String>,
    names_to_ids: HashMap<String, u64>,
}

impl TableMapper {
    /// Creates a new empty TableMapper
    pub fn new() -> Self {
        Self::default()
    }

    /// Deserializes a row into a user-defined type
    ///
    /// # Type Parameters
    /// * `T` - The type to deserialize into. Must implement DeserializeOwned.
    ///
    /// # Arguments
    /// * `row` - The row data as a HashMap
    ///
    /// # Returns
    /// * `Result<T, serde_json::Error>` - The deserialized struct or an error
    #[instrument(skip(self, row), fields(row_keys = ?row.keys().collect::<Vec<_>>()), err)]
    pub fn deserialize_row<T>(&self, row: HashMap<String, Value>) -> Result<T, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        // First convert field IDs to names
        let converted = self.convert_to_field_names(row);
        // Then deserialize
        serde_json::from_value(serde_json::to_value(converted)?)
    }

    /// Converts field IDs to field names in a row
    ///
    /// # Arguments
    /// * `row` - The row data with field IDs as keys
    ///
    /// # Returns
    /// * HashMap with field names as keys
    #[instrument(skip(self, row), fields(row_keys = ?row.keys().collect::<Vec<_>>()))]
    pub fn convert_to_field_names(&self, row: HashMap<String, Value>) -> HashMap<String, Value> {
        let mut converted = HashMap::new();
        for (key, value) in row {
            // Try to parse as a raw field ID first
            if let Ok(field_id) = key.parse::<u64>() {
                if let Some(name) = self.get_field_name(field_id) {
                    debug!(field_id = field_id, field_name = ?name, "Converted raw field ID to name");
                    converted.insert(name, value);
                    continue;
                }
            }
            // Then try with field_ prefix
            if let Some(field_id) = key
                .strip_prefix("field_")
                .and_then(|id| id.parse::<u64>().ok())
            {
                if let Some(name) = self.get_field_name(field_id) {
                    debug!(field_id = field_id, field_name = ?name, "Converted prefixed field ID to name");
                    converted.insert(name, value);
                    continue;
                }
                warn!(field_id = field_id, "No name mapping found for field ID");
            }
            debug!(key = ?key, "Keeping original key");
            converted.insert(key, value);
        }
        debug!(
            field_count = converted.len(),
            "Completed field name conversion"
        );
        converted
    }

    /// Converts field names to field IDs in a row
    ///
    /// # Arguments
    /// * `row` - The row data with field names as keys
    ///
    /// # Returns
    /// * HashMap with field IDs as keys
    #[instrument(skip(self, row), fields(row_keys = ?row.keys().collect::<Vec<_>>()))]
    pub fn convert_to_field_ids(&self, row: HashMap<String, Value>) -> HashMap<String, Value> {
        let mut converted = HashMap::new();
        for (key, value) in row {
            if let Some(id) = self.get_field_id(&key) {
                let field_key = format!("field_{}", id);
                debug!(field_name = ?key, field_id = id, "Converted field name to ID");
                converted.insert(field_key, value);
                continue;
            }
            debug!(key = ?key, "Keeping original key");
            converted.insert(key, value);
        }
        debug!(
            field_count = converted.len(),
            "Completed field ID conversion"
        );
        converted
    }
}

impl FieldMapper for TableMapper {
    #[instrument(skip(self, fields), fields(field_count = fields.len()))]
    fn map_fields(&mut self, fields: Vec<TableField>) {
        // Clear existing mappings
        self.ids_to_names.clear();
        self.names_to_ids.clear();

        // Add new mappings
        fields.iter().for_each(|field| {
            debug!(field_id = field.id, field_name = ?field.name, "Mapping field");
            self.ids_to_names.insert(field.id, field.name.clone());
            self.names_to_ids.insert(field.name.clone(), field.id);
        });

        self.fields = fields;
    }

    #[instrument(skip(self))]
    fn get_field_id(&self, name: &str) -> Option<u64> {
        let id = self.names_to_ids.get(name).copied();
        if id.is_none() {
            warn!(field_name = ?name, "Field name not found in mapping");
        }
        id
    }

    #[instrument(skip(self))]
    fn get_field_name(&self, id: u64) -> Option<String> {
        let name = self.ids_to_names.get(&id).cloned();
        if name.is_none() {
            warn!(field_id = id, "Field ID not found in mapping");
        }
        name
    }

    fn get_fields(&self) -> Vec<TableField> {
        self.fields.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_field(id: u64, name: &str) -> TableField {
        TableField {
            id,
            table_id: 1,
            name: name.to_string(),
            order: 0,
            r#type: "text".to_string(),
            primary: false,
            read_only: false,
            description: None,
        }
    }

    #[test]
    fn test_mapping_fields() {
        let mut mapper = TableMapper::new();
        let fields = vec![
            create_test_field(1, "Name"),
            create_test_field(2, "Email"),
            create_test_field(3, "Age"),
        ];

        mapper.map_fields(fields.clone());

        // Test field storage
        assert_eq!(mapper.get_fields().len(), 3);

        // Test ID to name mapping
        assert_eq!(mapper.get_field_name(1), Some("Name".to_string()));
        assert_eq!(mapper.get_field_name(2), Some("Email".to_string()));
        assert_eq!(mapper.get_field_name(3), Some("Age".to_string()));
        assert_eq!(mapper.get_field_name(4), None);

        // Test name to ID mapping
        assert_eq!(mapper.get_field_id("Name"), Some(1));
        assert_eq!(mapper.get_field_id("Email"), Some(2));
        assert_eq!(mapper.get_field_id("Age"), Some(3));
        assert_eq!(mapper.get_field_id("Unknown"), None);
    }

    #[test]
    fn test_remapping_fields() {
        let mut mapper = TableMapper::new();

        // Initial mapping
        let initial_fields = vec![create_test_field(1, "Name"), create_test_field(2, "Email")];
        mapper.map_fields(initial_fields);

        // Remap with updated fields
        let updated_fields = vec![
            create_test_field(1, "FullName"), // Changed name
            create_test_field(2, "Email"),
            create_test_field(3, "Phone"), // New field
        ];
        mapper.map_fields(updated_fields);

        // Verify updated mappings
        assert_eq!(mapper.get_field_name(1), Some("FullName".to_string()));
        assert_eq!(mapper.get_field_id("FullName"), Some(1));
        assert_eq!(mapper.get_field_name(3), Some("Phone".to_string()));
        assert_eq!(mapper.get_field_id("Phone"), Some(3));

        // Old name should no longer exist
        assert_eq!(mapper.get_field_id("Name"), None);
    }
}
