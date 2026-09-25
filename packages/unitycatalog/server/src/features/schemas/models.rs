use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Schema {
    pub name: String,
    pub catalog_name: String,
    pub comment: String,
    pub properties: BTreeMap<String, String>,
    pub full_name: String,
    pub owner: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: Option<i64>,
    pub updated_by: String,
    pub schema_id: String,
    pub storage_root: String,
    pub storage_location: String,
}
