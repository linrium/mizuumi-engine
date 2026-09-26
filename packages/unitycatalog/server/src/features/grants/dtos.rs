use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurableType {
    Metastore,
    Catalog,
    Schema,
    Table,
    Function,
    Volume,
    RegisteredModel,
    ExternalLocation,
    Credential,
}

impl SecurableType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Metastore => "metastore",
            Self::Catalog => "catalog",
            Self::Schema => "schema",
            Self::Table => "table",
            Self::Function => "function",
            Self::Volume => "volume",
            Self::RegisteredModel => "registered_model",
            Self::ExternalLocation => "external_location",
            Self::Credential => "credential",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Privilege {
    #[serde(rename = "CREATE CATALOG")]
    CreateCatalog,
    #[serde(rename = "USE CATALOG")]
    UseCatalog,
    #[serde(rename = "CREATE SCHEMA")]
    CreateSchema,
    #[serde(rename = "USE SCHEMA")]
    UseSchema,
    #[serde(rename = "CREATE TABLE")]
    CreateTable,
    #[serde(rename = "SELECT")]
    Select,
    #[serde(rename = "MODIFY")]
    Modify,
    #[serde(rename = "CREATE FUNCTION")]
    CreateFunction,
    #[serde(rename = "EXECUTE")]
    Execute,
    #[serde(rename = "CREATE VOLUME")]
    CreateVolume,
    #[serde(rename = "READ VOLUME")]
    ReadVolume,
    #[serde(rename = "CREATE MODEL")]
    CreateModel,
    #[serde(rename = "CREATE EXTERNAL LOCATION")]
    CreateExternalLocation,
    #[serde(rename = "READ FILES")]
    ReadFiles,
    #[serde(rename = "WRITE FILES")]
    WriteFiles,
    #[serde(rename = "CREATE EXTERNAL TABLE")]
    CreateExternalTable,
    #[serde(rename = "CREATE EXTERNAL VOLUME")]
    CreateExternalVolume,
    #[serde(rename = "CREATE MANAGED STORAGE")]
    CreateManagedStorage,
    #[serde(rename = "CREATE STORAGE CREDENTIAL")]
    CreateStorageCredential,
}

impl Privilege {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CreateCatalog => "CREATE CATALOG",
            Self::UseCatalog => "USE CATALOG",
            Self::CreateSchema => "CREATE SCHEMA",
            Self::UseSchema => "USE SCHEMA",
            Self::CreateTable => "CREATE TABLE",
            Self::Select => "SELECT",
            Self::Modify => "MODIFY",
            Self::CreateFunction => "CREATE FUNCTION",
            Self::Execute => "EXECUTE",
            Self::CreateVolume => "CREATE VOLUME",
            Self::ReadVolume => "READ VOLUME",
            Self::CreateModel => "CREATE MODEL",
            Self::CreateExternalLocation => "CREATE EXTERNAL LOCATION",
            Self::ReadFiles => "READ FILES",
            Self::WriteFiles => "WRITE FILES",
            Self::CreateExternalTable => "CREATE EXTERNAL TABLE",
            Self::CreateExternalVolume => "CREATE EXTERNAL VOLUME",
            Self::CreateManagedStorage => "CREATE MANAGED STORAGE",
            Self::CreateStorageCredential => "CREATE STORAGE CREDENTIAL",
        }
    }
}

impl TryFrom<String> for Privilege {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "CREATE CATALOG" => Ok(Self::CreateCatalog),
            "USE CATALOG" => Ok(Self::UseCatalog),
            "CREATE SCHEMA" => Ok(Self::CreateSchema),
            "USE SCHEMA" => Ok(Self::UseSchema),
            "CREATE TABLE" => Ok(Self::CreateTable),
            "SELECT" => Ok(Self::Select),
            "MODIFY" => Ok(Self::Modify),
            "CREATE FUNCTION" => Ok(Self::CreateFunction),
            "EXECUTE" => Ok(Self::Execute),
            "CREATE VOLUME" => Ok(Self::CreateVolume),
            "READ VOLUME" => Ok(Self::ReadVolume),
            "CREATE MODEL" => Ok(Self::CreateModel),
            "CREATE EXTERNAL LOCATION" => Ok(Self::CreateExternalLocation),
            "READ FILES" => Ok(Self::ReadFiles),
            "WRITE FILES" => Ok(Self::WriteFiles),
            "CREATE EXTERNAL TABLE" => Ok(Self::CreateExternalTable),
            "CREATE EXTERNAL VOLUME" => Ok(Self::CreateExternalVolume),
            "CREATE MANAGED STORAGE" => Ok(Self::CreateManagedStorage),
            "CREATE STORAGE CREDENTIAL" => Ok(Self::CreateStorageCredential),
            _ => Err(value),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct GetPermissionsRequest {
    #[validate(length(min = 1))]
    pub principal: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePermissions {
    #[validate(nested)]
    pub changes: Vec<PermissionsChange>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PermissionsChange {
    #[validate(length(min = 1))]
    pub principal: String,
    pub add: Vec<Privilege>,
    pub remove: Vec<Privilege>,
}

#[derive(Debug, Serialize)]
pub struct PermissionsList {
    pub privilege_assignments: Vec<PrivilegeAssignment>,
}

#[derive(Debug, Serialize)]
pub struct PrivilegeAssignment {
    pub principal: String,
    pub privileges: Vec<Privilege>,
}

#[cfg(test)]
mod tests {
    use super::Privilege;

    #[test]
    fn privilege_values_match_openapi() {
        let cases = [
            (Privilege::CreateCatalog, "CREATE CATALOG"),
            (Privilege::UseCatalog, "USE CATALOG"),
            (Privilege::CreateSchema, "CREATE SCHEMA"),
            (Privilege::UseSchema, "USE SCHEMA"),
            (Privilege::CreateTable, "CREATE TABLE"),
            (Privilege::Select, "SELECT"),
            (Privilege::Modify, "MODIFY"),
            (Privilege::CreateFunction, "CREATE FUNCTION"),
            (Privilege::Execute, "EXECUTE"),
            (Privilege::CreateVolume, "CREATE VOLUME"),
            (Privilege::ReadVolume, "READ VOLUME"),
            (Privilege::CreateModel, "CREATE MODEL"),
            (
                Privilege::CreateExternalLocation,
                "CREATE EXTERNAL LOCATION",
            ),
            (Privilege::ReadFiles, "READ FILES"),
            (Privilege::WriteFiles, "WRITE FILES"),
            (Privilege::CreateExternalTable, "CREATE EXTERNAL TABLE"),
            (Privilege::CreateExternalVolume, "CREATE EXTERNAL VOLUME"),
            (Privilege::CreateManagedStorage, "CREATE MANAGED STORAGE"),
            (
                Privilege::CreateStorageCredential,
                "CREATE STORAGE CREDENTIAL",
            ),
        ];

        for (privilege, value) in cases {
            assert_eq!(privilege.as_str(), value);
            assert_eq!(Privilege::try_from(value.to_string()), Ok(privilege));
            assert_eq!(
                serde_json::to_string(&privilege).unwrap(),
                format!("\"{value}\"")
            );
            assert_eq!(
                serde_json::from_str::<Privilege>(&format!("\"{value}\"")).unwrap(),
                privilege
            );
        }
    }
}
