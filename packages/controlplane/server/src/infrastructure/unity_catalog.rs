use async_trait::async_trait;
use reqwest::{Certificate, Client, Url};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;
use thiserror::Error;

use crate::config::UnityCatalogSettings;

#[derive(Debug, Error)]
pub enum UnityCatalogError {
    #[error("invalid Unity Catalog configuration: {0}")]
    Configuration(String),
    #[error("Unity Catalog request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Unity Catalog returned HTTP {status}: {body}")]
    Response { status: u16, body: String },
}

#[async_trait]
pub trait UnityCatalog: Send + Sync {
    async fn get_table(&self, full_name: &str) -> Result<Option<UnityTable>, UnityCatalogError>;
    async fn update_table_properties(
        &self,
        full_name: &str,
        properties: &BTreeMap<String, String>,
    ) -> Result<UnityTable, UnityCatalogError>;

    async fn has_privilege(
        &self,
        securable_type: &str,
        full_name: &str,
        principal: &str,
        privilege: &str,
    ) -> Result<bool, UnityCatalogError>;

    async fn grant(
        &self,
        securable_type: &str,
        full_name: &str,
        principal: &str,
        privilege: &str,
    ) -> Result<(), UnityCatalogError>;

    async fn revoke(
        &self,
        securable_type: &str,
        full_name: &str,
        principal: &str,
        privilege: &str,
    ) -> Result<(), UnityCatalogError>;
}

pub struct HttpUnityCatalog {
    client: Client,
    base_url: Url,
    token_url: Url,
    client_id: String,
    client_secret: String,
}

impl HttpUnityCatalog {
    pub fn new(settings: &UnityCatalogSettings) -> Result<Self, UnityCatalogError> {
        if settings.client_id.trim().is_empty() || settings.client_secret.trim().is_empty() {
            return Err(UnityCatalogError::Configuration(
                "client_id and client_secret are required".to_owned(),
            ));
        }

        let mut client = Client::builder().timeout(Duration::from_secs(10));
        if let Some(path) = &settings.ca_certificate {
            let pem = std::fs::read(path).map_err(|error| {
                UnityCatalogError::Configuration(format!(
                    "failed to read CA certificate {}: {error}",
                    path.display()
                ))
            })?;
            let certificate = Certificate::from_pem(&pem).map_err(|error| {
                UnityCatalogError::Configuration(format!(
                    "failed to parse CA certificate {}: {error}",
                    path.display()
                ))
            })?;
            client = client.add_root_certificate(certificate);
        }

        Ok(Self {
            client: client.build()?,
            base_url: parse_url("url", &settings.url)?,
            token_url: parse_url("token_url", &settings.token_url)?,
            client_id: settings.client_id.clone(),
            client_secret: settings.client_secret.clone(),
        })
    }

    fn permissions_url(
        &self,
        securable_type: &str,
        full_name: &str,
    ) -> Result<Url, UnityCatalogError> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .map_err(|_| UnityCatalogError::Configuration("url cannot be a base URL".to_owned()))?
            .pop_if_empty()
            .extend([
                "api",
                "2.1",
                "unity-catalog",
                "permissions",
                securable_type,
                full_name,
            ]);
        Ok(url)
    }

    fn table_url(&self, full_name: &str) -> Result<Url, UnityCatalogError> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .map_err(|_| UnityCatalogError::Configuration("url cannot be a base URL".to_owned()))?
            .pop_if_empty()
            .extend(["api", "2.1", "unity-catalog", "tables", full_name]);
        Ok(url)
    }

    async fn access_token(&self) -> Result<String, UnityCatalogError> {
        let response = self
            .client
            .post(self.token_url.clone())
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
            ])
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            return Err(UnityCatalogError::Response {
                status: status.as_u16(),
                body: response.text().await.unwrap_or_default(),
            });
        }
        Ok(response.json::<TokenResponse>().await?.access_token)
    }

    async fn update(
        &self,
        securable_type: &str,
        full_name: &str,
        principal: &str,
        add: &[&str],
        remove: &[&str],
    ) -> Result<(), UnityCatalogError> {
        let token = self.access_token().await?;
        let response = self
            .client
            .patch(self.permissions_url(securable_type, full_name)?)
            .bearer_auth(token)
            .json(&PermissionUpdate {
                changes: vec![PermissionChange {
                    principal,
                    add,
                    remove,
                }],
            })
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(UnityCatalogError::Response {
                status: response.status().as_u16(),
                body: response.text().await.unwrap_or_default(),
            });
        }
        Ok(())
    }
}

#[async_trait]
impl UnityCatalog for HttpUnityCatalog {
    async fn get_table(&self, full_name: &str) -> Result<Option<UnityTable>, UnityCatalogError> {
        let token = self.access_token().await?;
        let response = self
            .client
            .get(self.table_url(full_name)?)
            .bearer_auth(token)
            .send()
            .await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(UnityCatalogError::Response {
                status: response.status().as_u16(),
                body: response.text().await.unwrap_or_default(),
            });
        }
        Ok(Some(response.json::<UnityTable>().await?))
    }

    async fn update_table_properties(
        &self,
        full_name: &str,
        properties: &BTreeMap<String, String>,
    ) -> Result<UnityTable, UnityCatalogError> {
        let token = self.access_token().await?;
        let response = self
            .client
            .patch(self.table_url(full_name)?)
            .bearer_auth(token)
            .json(&TablePropertiesUpdate { properties })
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(UnityCatalogError::Response {
                status: response.status().as_u16(),
                body: response.text().await.unwrap_or_default(),
            });
        }
        Ok(response.json::<UnityTable>().await?)
    }

    async fn has_privilege(
        &self,
        securable_type: &str,
        full_name: &str,
        principal: &str,
        privilege: &str,
    ) -> Result<bool, UnityCatalogError> {
        let token = self.access_token().await?;
        let response = self
            .client
            .get(self.permissions_url(securable_type, full_name)?)
            .bearer_auth(token)
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            return Err(UnityCatalogError::Response {
                status: status.as_u16(),
                body: response.text().await.unwrap_or_default(),
            });
        }
        let permissions = response.json::<PermissionsResponse>().await?;
        Ok(permissions.privilege_assignments.iter().any(|assignment| {
            assignment.principal == principal
                && assignment
                    .privileges
                    .iter()
                    .any(|assigned| assigned == privilege)
        }))
    }

    async fn grant(
        &self,
        securable_type: &str,
        full_name: &str,
        principal: &str,
        privilege: &str,
    ) -> Result<(), UnityCatalogError> {
        self.update(securable_type, full_name, principal, &[privilege], &[])
            .await
    }

    async fn revoke(
        &self,
        securable_type: &str,
        full_name: &str,
        principal: &str,
        privilege: &str,
    ) -> Result<(), UnityCatalogError> {
        self.update(securable_type, full_name, principal, &[], &[privilege])
            .await
    }
}

fn parse_url(name: &str, value: &str) -> Result<Url, UnityCatalogError> {
    Url::parse(value).map_err(|error| {
        UnityCatalogError::Configuration(format!("{name} is not a valid URL: {error}"))
    })
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct PermissionsResponse {
    #[serde(default)]
    privilege_assignments: Vec<PrivilegeAssignment>,
}

#[derive(Deserialize)]
struct PrivilegeAssignment {
    principal: String,
    #[serde(default)]
    privileges: Vec<String>,
}

#[derive(Serialize)]
struct PermissionUpdate<'a> {
    changes: Vec<PermissionChange<'a>>,
}

#[derive(Serialize)]
struct PermissionChange<'a> {
    principal: &'a str,
    add: &'a [&'a str],
    remove: &'a [&'a str],
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnityTable {
    pub table_id: String,
    #[serde(default)]
    pub columns: Vec<UnityColumn>,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnityColumn {
    pub name: Option<String>,
    pub type_text: Option<String>,
    pub type_name: Option<String>,
    pub comment: Option<String>,
    pub nullable: Option<bool>,
    pub partition_index: Option<i32>,
    pub position: Option<i32>,
}

#[derive(Serialize)]
struct TablePropertiesUpdate<'a> {
    properties: &'a BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> UnityCatalogSettings {
        UnityCatalogSettings {
            url: "http://unitycatalog:8080".to_owned(),
            token_url: "http://keycloak/token".to_owned(),
            ca_certificate: None,
            client_id: "controlplane".to_owned(),
            client_secret: "secret".to_owned(),
        }
    }

    #[test]
    fn permissions_url_encodes_the_full_name_as_one_path_segment() {
        let client = HttpUnityCatalog::new(&settings()).unwrap();
        let url = client
            .permissions_url("table", "catalog.schema/table")
            .unwrap();

        assert_eq!(
            url.as_str(),
            "http://unitycatalog:8080/api/2.1/unity-catalog/permissions/table/catalog.schema%2Ftable"
        );
    }

    #[test]
    fn table_url_encodes_the_full_name_as_one_path_segment() {
        let client = HttpUnityCatalog::new(&settings()).unwrap();
        let url = client.table_url("catalog.schema/table").unwrap();

        assert_eq!(
            url.as_str(),
            "http://unitycatalog:8080/api/2.1/unity-catalog/tables/catalog.schema%2Ftable"
        );
    }

    #[test]
    fn client_credentials_are_required() {
        let mut settings = settings();
        settings.client_secret.clear();

        assert!(matches!(
            HttpUnityCatalog::new(&settings),
            Err(UnityCatalogError::Configuration(_))
        ));
    }
}
