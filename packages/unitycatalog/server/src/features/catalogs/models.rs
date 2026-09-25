use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Catalog {
    pub name: String,
    pub comment: String,
    pub properties: BTreeMap<String, String>,
    pub owner: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: Option<i64>,
    pub updated_by: String,
    pub id: String,
    pub storage_root: String,
    pub storage_location: String,
}
