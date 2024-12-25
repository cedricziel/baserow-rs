use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

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
    pub fn deserialize_row<T>(&self, row: HashMap<String, Value>) -> Result<T, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        serde_json::from_value(serde_json::to_value(row)?)
    }
}

impl FieldMapper for TableMapper {
    fn map_fields(&mut self, fields: Vec<TableField>) {
        // Clear existing mappings
        self.ids_to_names.clear();
        self.names_to_ids.clear();

        // Add new mappings
        fields.iter().for_each(|field| {
            self.ids_to_names.insert(field.id, field.name.clone());
            self.names_to_ids.insert(field.name.clone(), field.id);
        });

        self.fields = fields;
    }

    fn get_field_id(&self, name: &str) -> Option<u64> {
        self.names_to_ids.get(name).copied()
    }

    fn get_field_name(&self, id: u64) -> Option<String> {
        self.ids_to_names.get(&id).cloned()
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
