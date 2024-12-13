use std::collections::HashMap;

use crate::TableField;

#[derive(Clone, Default)]
pub struct TableMapper {
    fields: Vec<TableField>,
    ids_to_names: HashMap<u64, String>,
    names_to_ids: HashMap<String, u64>,
}

impl TableMapper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn map_fields(&mut self, fields: Vec<TableField>) {
        fields.iter().for_each(|field| {
            self.ids_to_names.insert(field.id, field.name.clone());
            self.names_to_ids.insert(field.name.clone(), field.id);
        });

        self.fields = fields;
    }
}
