use std::{
    collections::{BTreeSet, HashMap},
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
    sync::Arc,
};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use anyhow::{Context, anyhow};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{Method, header},
    middleware::Next,
    response::Response,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use tokio::sync::RwLock;

use crate::{config::AuthSettings, error::AppError};

#[derive(Clone)]
pub struct AuthService {
    inner: Arc<AuthServiceInner>,
}

struct AuthServiceInner {
    enabled: bool,
    issuer: String,
    audience: String,
    jwks_url: String,
    clock_skew_seconds: u64,
    reader_role: String,
    writer_role: String,
    admin_role: String,
    bootstrap_token: Option<String>,
    client: reqwest::Client,
    keys: RwLock<HashMap<String, DecodingKey>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalKind {
    Human,
    Service,
    Bootstrap,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AuthenticatedPrincipal {
    pub id: String,
    pub subject: String,
    pub kind: PrincipalKind,
    pub roles: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    email_verified: bool,
    #[serde(default)]
    preferred_username: Option<String>,
    #[serde(default)]
    client_id: Option<String>,
    #[serde(default)]
    realm_access: RoleAccess,
    #[serde(default)]
    resource_access: HashMap<String, RoleAccess>,
}

#[derive(Debug, Default, Deserialize)]
struct RoleAccess {
    #[serde(default)]
    roles: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    #[serde(default)]
    alg: Option<String>,
    n: String,
    e: String,
}

impl AuthService {
    pub fn new(settings: &AuthSettings) -> anyhow::Result<Self> {
        if settings.enabled
            && [
                settings.issuer.as_str(),
                settings.audience.as_str(),
                settings.reader_role.as_str(),
                settings.writer_role.as_str(),
                settings.admin_role.as_str(),
            ]
            .iter()
            .any(|value| value.trim().is_empty())
        {
            return Err(anyhow!(
                "auth issuer, audience, and role names must not be empty"
            ));
        }
        let mut client_builder = reqwest::Client::builder();
        if let Some(path) = &settings.ca_certificate {
            let pem = fs::read(path).with_context(|| {
                format!("failed to read Keycloak CA certificate {}", path.display())
            })?;
            let certificate = reqwest::Certificate::from_pem(&pem)
                .context("failed to parse Keycloak CA certificate")?;
            client_builder = client_builder.add_root_certificate(certificate);
        }

        let bootstrap_token = if settings.enabled && settings.bootstrap_enabled {
            load_or_create_bootstrap_token(settings)?
        } else {
            None
        };
        let jwks_url = settings.jwks_url.clone().unwrap_or_else(|| {
            format!(
                "{}/protocol/openid-connect/certs",
                settings.issuer.trim_end_matches('/')
            )
        });

        Ok(Self {
            inner: Arc::new(AuthServiceInner {
                enabled: settings.enabled,
                issuer: settings.issuer.trim_end_matches('/').to_owned(),
                audience: settings.audience.clone(),
                jwks_url,
                clock_skew_seconds: settings.clock_skew_seconds,
                reader_role: settings.reader_role.clone(),
                writer_role: settings.writer_role.clone(),
                admin_role: settings.admin_role.clone(),
                bootstrap_token,
                client: client_builder
                    .build()
                    .context("failed to create Keycloak HTTP client")?,
                keys: RwLock::new(HashMap::new()),
            }),
        })
    }

    async fn authenticate(&self, token: &str) -> Result<AuthenticatedPrincipal, AppError> {
        if !self.inner.enabled {
            return Ok(self.disabled_principal());
        }

        if self.is_bootstrap_token(token) {
            return Ok(self.bootstrap_principal());
        }

        let header = decode_header(token).map_err(|error| {
            tracing::debug!(%error, "bearer token header is invalid");
            AppError::Unauthorized("invalid bearer token".to_owned())
        })?;
        if header.alg != Algorithm::RS256 {
            return Err(AppError::Unauthorized(
                "bearer token must use RS256".to_owned(),
            ));
        }
        let kid = header
            .kid
            .ok_or_else(|| AppError::Unauthorized("bearer token is missing a key id".to_owned()))?;
        let key = self.decoding_key(&kid).await?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.leeway = self.inner.clock_skew_seconds;
        validation.validate_nbf = true;
        validation.set_audience(&[&self.inner.audience]);
        validation.set_issuer(&[&self.inner.issuer]);
        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
        let claims = decode::<Claims>(token, &key, &validation)
            .map_err(|error| {
                tracing::debug!(%error, "bearer token validation failed");
                AppError::Unauthorized("invalid bearer token".to_owned())
            })?
            .claims;

        Ok(self.principal_from_claims(claims))
    }

    async fn decoding_key(&self, kid: &str) -> Result<DecodingKey, AppError> {
        if let Some(key) = self.inner.keys.read().await.get(kid).cloned() {
            return Ok(key);
        }

        self.refresh_keys().await?;
        self.inner
            .keys
            .read()
            .await
            .get(kid)
            .cloned()
            .ok_or_else(|| {
                AppError::Unauthorized("bearer token references an unknown key".to_owned())
            })
    }

    async fn refresh_keys(&self) -> Result<(), AppError> {
        let response = self
            .inner
            .client
            .get(&self.inner.jwks_url)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| {
                tracing::error!(%error, jwks_url = %self.inner.jwks_url, "failed to fetch Keycloak keys");
                AppError::AuthProviderUnavailable
            })?;
        let jwks = response.json::<JwkSet>().await.map_err(|error| {
            tracing::error!(%error, "Keycloak returned an invalid JWKS document");
            AppError::AuthProviderUnavailable
        })?;
        let mut keys = HashMap::new();
        for jwk in jwks.keys {
            if jwk.kty != "RSA" || jwk.alg.as_deref().is_some_and(|alg| alg != "RS256") {
                continue;
            }
            match DecodingKey::from_rsa_components(&jwk.n, &jwk.e) {
                Ok(key) => {
                    keys.insert(jwk.kid, key);
                }
                Err(error) => {
                    tracing::warn!(%error, kid = %jwk.kid, "ignoring invalid Keycloak signing key");
                }
            }
        }
        if keys.is_empty() {
            tracing::error!("Keycloak JWKS contains no usable RS256 keys");
            return Err(AppError::AuthProviderUnavailable);
        }
        *self.inner.keys.write().await = keys;
        Ok(())
    }

    fn principal_from_claims(&self, claims: Claims) -> AuthenticatedPrincipal {
        let service_client = claims
            .preferred_username
            .as_deref()
            .and_then(|username| username.strip_prefix("service-account-"))
            .map(str::to_owned)
            .or_else(|| claims.client_id.clone().filter(|_| claims.email.is_none()));
        let (kind, id) = if let Some(client_id) = service_client {
            (PrincipalKind::Service, client_id)
        } else {
            (
                PrincipalKind::Human,
                claims
                    .email
                    .clone()
                    .filter(|_| claims.email_verified)
                    .or(claims.preferred_username.clone())
                    .unwrap_or_else(|| claims.sub.clone()),
            )
        };

        let mut roles = claims.realm_access.roles;
        if let Some(access) = claims.resource_access.get(&self.inner.audience) {
            roles.extend(access.roles.iter().cloned());
        }
        AuthenticatedPrincipal {
            id,
            subject: claims.sub,
            kind,
            roles,
        }
    }

    fn authorize(
        &self,
        principal: &AuthenticatedPrincipal,
        method: &Method,
        path: &str,
    ) -> Result<(), AppError> {
        if !self.inner.enabled || principal.roles.contains(&self.inner.admin_role) {
            return Ok(());
        }
        if path.starts_with("/api/2.1/unity-catalog/permissions/") {
            return Err(AppError::Forbidden(
                "permission management requires the Unity Catalog admin role".to_owned(),
            ));
        }
        let allowed = matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
            && (principal.roles.contains(&self.inner.reader_role)
                || principal.roles.contains(&self.inner.writer_role))
            || !matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
                && principal.roles.contains(&self.inner.writer_role);
        if allowed {
            Ok(())
        } else {
            Err(AppError::Forbidden(
                "principal does not have the required Unity Catalog role".to_owned(),
            ))
        }
    }

    fn is_bootstrap_token(&self, token: &str) -> bool {
        self.inner
            .bootstrap_token
            .as_deref()
            .is_some_and(|expected| {
                expected.len() == token.len()
                    && bool::from(expected.as_bytes().ct_eq(token.as_bytes()))
            })
    }

    fn bootstrap_principal(&self) -> AuthenticatedPrincipal {
        AuthenticatedPrincipal {
            id: "bootstrap".to_owned(),
            subject: "bootstrap".to_owned(),
            kind: PrincipalKind::Bootstrap,
            roles: BTreeSet::from([self.inner.admin_role.clone()]),
        }
    }

    fn disabled_principal(&self) -> AuthenticatedPrincipal {
        AuthenticatedPrincipal {
            id: "anonymous".to_owned(),
            subject: "anonymous".to_owned(),
            kind: PrincipalKind::Bootstrap,
            roles: BTreeSet::from([self.inner.admin_role.clone()]),
        }
    }
}

pub async fn authorize(
    State(auth): State<AuthService>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let principal = if auth.inner.enabled {
        let header = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing bearer token".to_owned()))?;
        let token = header.strip_prefix("Bearer ").ok_or_else(|| {
            AppError::Unauthorized("authorization must use the Bearer scheme".to_owned())
        })?;
        auth.authenticate(token).await?
    } else {
        auth.disabled_principal()
    };

    auth.authorize(&principal, request.method(), request.uri().path())?;
    tracing::Span::current().record("principal", &principal.id);
    request.extensions_mut().insert(principal);
    Ok(next.run(request).await)
}

fn load_or_create_bootstrap_token(settings: &AuthSettings) -> anyhow::Result<Option<String>> {
    if let Some(token) = settings.bootstrap_token.as_deref() {
        validate_bootstrap_token(token)?;
        tracing::info!("using configured Unity Catalog bootstrap token");
        return Ok(Some(token.to_owned()));
    }
    let Some(path) = settings.bootstrap_token_file.as_deref() else {
        tracing::warn!("authentication is enabled without a bootstrap token");
        return Ok(None);
    };
    let (token, created) = load_or_create_token_file(path)?;
    if created {
        tracing::warn!(path = %path.display(), "generated Unity Catalog bootstrap token; protect this file and replace the token with Keycloak credentials after bootstrap");
    } else {
        tracing::info!(path = %path.display(), "loaded Unity Catalog bootstrap token");
    }
    Ok(Some(token))
}

fn load_or_create_token_file(path: &Path) -> anyhow::Result<(String, bool)> {
    match fs::read_to_string(path) {
        Ok(token) => {
            let token = token.trim().to_owned();
            validate_bootstrap_token(&token)?;
            return Ok((token, false));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", path.display()));
        }
    }

    let token = generate_bootstrap_token()?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    match options.open(path) {
        Ok(mut file) => {
            file.write_all(token.as_bytes())
                .and_then(|()| file.write_all(b"\n"))
                .with_context(|| format!("failed to write {}", path.display()))?;
            Ok((token, true))
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let existing = fs::read_to_string(path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let existing = existing.trim().to_owned();
            validate_bootstrap_token(&existing)?;
            Ok((existing, false))
        }
        Err(error) => Err(error).with_context(|| format!("failed to create {}", path.display())),
    }
}

fn generate_bootstrap_token() -> anyhow::Result<String> {
    let mut bytes = [0_u8; 32];
    ring::rand::SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| anyhow!("operating system random number generator failed"))?;
    Ok(format!("ucb_{}", URL_SAFE_NO_PAD.encode(bytes)))
}

fn validate_bootstrap_token(token: &str) -> anyhow::Result<()> {
    if token.len() < 32 || token.chars().any(char::is_whitespace) {
        return Err(anyhow!(
            "bootstrap token must be at least 32 characters and contain no whitespace"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> AuthSettings {
        AuthSettings {
            bootstrap_token_file: None,
            ..AuthSettings::default()
        }
    }

    #[test]
    fn identifies_human_and_service_principals() {
        let auth = AuthService::new(&settings()).unwrap();
        let human = auth.principal_from_claims(Claims {
            sub: "human-subject".to_owned(),
            email: Some("person@example.com".to_owned()),
            email_verified: true,
            preferred_username: Some("person".to_owned()),
            client_id: None,
            realm_access: RoleAccess::default(),
            resource_access: HashMap::new(),
        });
        let service = auth.principal_from_claims(Claims {
            sub: "service-subject".to_owned(),
            email: None,
            email_verified: false,
            preferred_username: Some("service-account-ingestion".to_owned()),
            client_id: Some("ingestion".to_owned()),
            realm_access: RoleAccess::default(),
            resource_access: HashMap::new(),
        });

        assert_eq!(human.kind, PrincipalKind::Human);
        assert_eq!(human.id, "person@example.com");
        assert_eq!(service.kind, PrincipalKind::Service);
        assert_eq!(service.id, "ingestion");
    }

    #[test]
    fn enforces_role_hierarchy_and_admin_only_permissions() {
        let auth = AuthService::new(&settings()).unwrap();
        let principal = |role: &str| AuthenticatedPrincipal {
            id: "test".to_owned(),
            subject: "test".to_owned(),
            kind: PrincipalKind::Human,
            roles: BTreeSet::from([role.to_owned()]),
        };

        assert!(
            auth.authorize(
                &principal("unitycatalog-reader"),
                &Method::GET,
                "/api/auth/me"
            )
            .is_ok()
        );
        assert!(
            auth.authorize(
                &principal("unitycatalog-reader"),
                &Method::POST,
                "/api/catalogs"
            )
            .is_err()
        );
        assert!(
            auth.authorize(
                &principal("unitycatalog-writer"),
                &Method::GET,
                "/api/2.1/unity-catalog/permissions/catalog/main",
            )
            .is_err()
        );
        assert!(
            auth.authorize(
                &principal("unitycatalog-admin"),
                &Method::PATCH,
                "/api/2.1/unity-catalog/permissions/catalog/main",
            )
            .is_ok()
        );
    }

    #[test]
    fn generated_bootstrap_tokens_have_256_bits_of_random_input() {
        let first = generate_bootstrap_token().unwrap();
        let second = generate_bootstrap_token().unwrap();
        assert!(first.starts_with("ucb_"));
        assert_eq!(first.len(), 47);
        assert_ne!(first, second);
    }
}
