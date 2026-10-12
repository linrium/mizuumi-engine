use async_trait::async_trait;
use controlplane_queries::queries::policy_templates as queries;
use deadpool_postgres::{Object, Pool};
use tokio_postgres::error::SqlState;

use crate::error::AppError;

use super::dtos::{
    ApprovalTeam, CreatePolicyTemplateRequest, Guardrail, ListPolicyTemplatesRequest,
    ListPolicyTemplatesResponse, PolicyGrant, PolicyTemplate, PolicyTemplateSummary,
};

const SECURABLE_TYPES: &[&str] = &[
    "metastore",
    "catalog",
    "schema",
    "table",
    "function",
    "volume",
    "registered_model",
    "external_location",
    "credential",
];
fn privileges_for(securable_type: &str) -> &'static [&'static str] {
    match securable_type {
        "metastore" => &["CREATE CATALOG"],
        "catalog" => &["USE CATALOG", "CREATE SCHEMA"],
        "schema" => &[
            "USE SCHEMA",
            "CREATE TABLE",
            "CREATE FUNCTION",
            "CREATE VOLUME",
            "CREATE MODEL",
        ],
        "table" => &["SELECT", "MODIFY"],
        "function" | "registered_model" => &["EXECUTE"],
        "volume" => &["READ VOLUME"],
        "external_location" => &[
            "READ FILES",
            "WRITE FILES",
            "CREATE EXTERNAL TABLE",
            "CREATE EXTERNAL VOLUME",
            "CREATE MANAGED STORAGE",
        ],
        "credential" => &["CREATE EXTERNAL LOCATION", "CREATE STORAGE CREDENTIAL"],
        _ => &[],
    }
}

#[async_trait]
pub trait PolicyTemplateService: Send + Sync {
    async fn create_policy_template(
        &self,
        request: CreatePolicyTemplateRequest,
    ) -> Result<PolicyTemplate, AppError>;
    async fn list_policy_templates(
        &self,
        request: ListPolicyTemplatesRequest,
    ) -> Result<ListPolicyTemplatesResponse, AppError>;
    async fn get_policy_template(&self, template_id: String) -> Result<PolicyTemplate, AppError>;
}

pub struct DefaultPolicyTemplateService {
    pool: Pool,
}

impl DefaultPolicyTemplateService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PolicyTemplateService for DefaultPolicyTemplateService {
    async fn create_policy_template(
        &self,
        request: CreatePolicyTemplateRequest,
    ) -> Result<PolicyTemplate, AppError> {
        validate_create_request(&request)?;

        let mut client = self.pool.get().await?;
        let transaction = client.transaction().await?;
        let template_id = queries::create_policy_template()
            .bind(&transaction, &request.key, &request.created_by)
            .one()
            .await
            .map_err(map_policy_write_error)?;

        queries::create_policy_template_version()
            .bind(
                &transaction,
                &template_id,
                &request.name,
                &request.description.clone().unwrap_or_default(),
                &request.approval_strategy,
                &request.auto_approve,
                &request.created_by,
            )
            .await?;

        for team in &request.affected_teams {
            queries::create_policy_template_affected_team()
                .bind(&transaction, &template_id, team)
                .await
                .map_err(map_policy_write_error)?;
        }
        for team in &request.approval_teams {
            queries::create_policy_template_approval_team()
                .bind(
                    &transaction,
                    &template_id,
                    &team.team,
                    &team.required_approvals,
                )
                .await
                .map_err(map_policy_write_error)?;
        }
        queries::create_policy_template_grant()
            .bind(
                &transaction,
                &template_id,
                &request.grant.securable_type,
                &request.grant.target_full_name,
                &request.grant.privilege,
            )
            .await
            .map_err(map_policy_write_error)?;
        for guardrail in &request.guardrails {
            queries::create_policy_template_guardrail()
                .bind(
                    &transaction,
                    &template_id,
                    &guardrail.guardrail_key,
                    &guardrail.name,
                    &guardrail.description.clone().unwrap_or_default(),
                    &guardrail.enforcement,
                    &guardrail.configuration,
                    &request.created_by,
                )
                .await
                .map_err(map_policy_write_error)?;
        }

        transaction.commit().await?;
        load_policy_template(&client, &template_id).await
    }

    async fn list_policy_templates(
        &self,
        request: ListPolicyTemplatesRequest,
    ) -> Result<ListPolicyTemplatesResponse, AppError> {
        let client = self.pool.get().await?;
        let affected_team = request.affected_team.unwrap_or_default();
        let policy_templates = queries::list_policy_templates()
            .bind(&client, &affected_team)
            .all()
            .await?
            .into_iter()
            .map(|row| PolicyTemplateSummary {
                id: row.id,
                key: row.key,
                version: row.version,
                name: row.name,
                description: non_empty(row.description),
                approval_strategy: row.approval_strategy,
                auto_approve: row.auto_approve,
                created_by: row.created_by,
                created_at: row.created_at,
            })
            .collect();

        Ok(ListPolicyTemplatesResponse { policy_templates })
    }

    async fn get_policy_template(&self, template_id: String) -> Result<PolicyTemplate, AppError> {
        let client = self.pool.get().await?;
        load_policy_template(&client, &template_id).await
    }
}

async fn load_policy_template(
    client: &Object,
    template_id: &str,
) -> Result<PolicyTemplate, AppError> {
    let row = queries::get_policy_template()
        .bind(client, &template_id)
        .opt()
        .await?
        .ok_or_else(|| AppError::NotFound(format!("policy template {template_id}")))?;
    let version = row.version;
    let grant = queries::list_policy_template_grants()
        .bind(client, &template_id, &version)
        .opt()
        .await?
        .map(|grant| PolicyGrant {
            securable_type: grant.securable_type,
            target_full_name: grant.target_full_name,
            privilege: grant.privilege,
        })
        .ok_or_else(|| AppError::Conflict(format!("policy template {template_id} has no grant")))?;
    let affected_teams = queries::list_policy_template_affected_teams()
        .bind(client, &template_id, &version)
        .all()
        .await?;
    let approval_teams = queries::list_policy_template_approval_teams()
        .bind(client, &template_id, &version)
        .all()
        .await?
        .into_iter()
        .map(|team| ApprovalTeam {
            team: team.team,
            required_approvals: team.required_approvals,
        })
        .collect();
    let guardrails = queries::list_policy_template_guardrails()
        .bind(client, &template_id, &version)
        .all()
        .await?
        .into_iter()
        .map(|guardrail| Guardrail {
            guardrail_key: guardrail.guardrail_key,
            name: guardrail.name,
            description: non_empty(guardrail.description),
            enforcement: guardrail.enforcement,
            configuration: guardrail.configuration,
        })
        .collect();

    Ok(PolicyTemplate {
        id: row.id,
        key: row.key,
        version,
        name: row.name,
        description: non_empty(row.description),
        status: row.status,
        approval_strategy: row.approval_strategy,
        auto_approve: row.auto_approve,
        blast_radius_required: row.blast_radius_required,
        guardrail_review_required: row.guardrail_review_required,
        affected_teams,
        approval_teams,
        grant,
        guardrails,
        created_by: row.created_by,
        created_at: row.created_at,
        published_by: non_empty(row.published_by),
        published_at: non_empty(row.published_at),
    })
}

fn validate_create_request(request: &CreatePolicyTemplateRequest) -> Result<(), AppError> {
    require_text("key", &request.key)?;
    require_text("name", &request.name)?;
    require_text("created_by", &request.created_by)?;
    if !matches!(request.approval_strategy.as_str(), "ANY" | "ALL") {
        return Err(AppError::InvalidParameter(
            "approval_strategy must be ANY or ALL".to_owned(),
        ));
    }
    if request.affected_teams.is_empty() {
        return Err(AppError::InvalidParameter(
            "affected_teams must not be empty".to_owned(),
        ));
    }
    if !request.auto_approve && request.approval_teams.is_empty() {
        return Err(AppError::InvalidParameter(
            "approval_teams must not be empty when manual approval is required".to_owned(),
        ));
    }
    for team in &request.affected_teams {
        require_text("affected team", team)?;
    }
    for team in &request.approval_teams {
        require_text("approval team", &team.team)?;
        if team.required_approvals <= 0 {
            return Err(AppError::InvalidParameter(
                "required_approvals must be positive".to_owned(),
            ));
        }
    }
    let grant = &request.grant;
    if !SECURABLE_TYPES.contains(&grant.securable_type.as_str()) {
        return Err(AppError::InvalidParameter(format!(
            "unsupported securable_type: {}",
            grant.securable_type
        )));
    }
    require_text("target_full_name", &grant.target_full_name)?;
    if !privileges_for(&grant.securable_type).contains(&grant.privilege.as_str()) {
        return Err(AppError::InvalidParameter(format!(
            "privilege {} is not valid for securable_type {}",
            grant.privilege, grant.securable_type
        )));
    }
    for guardrail in &request.guardrails {
        require_text("guardrail_key", &guardrail.guardrail_key)?;
        require_text("guardrail name", &guardrail.name)?;
        if !matches!(guardrail.enforcement.as_str(), "REQUIRED" | "ADVISORY") {
            return Err(AppError::InvalidParameter(
                "guardrail enforcement must be REQUIRED or ADVISORY".to_owned(),
            ));
        }
        if !guardrail.configuration.is_object() {
            return Err(AppError::InvalidParameter(
                "guardrail configuration must be a JSON object".to_owned(),
            ));
        }
    }

    Ok(())
}

fn require_text(field: &str, value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        Err(AppError::InvalidParameter(format!(
            "{field} must not be empty"
        )))
    } else {
        Ok(())
    }
}

fn map_policy_write_error(error: tokio_postgres::Error) -> AppError {
    match error.code() {
        Some(code) if code == &SqlState::UNIQUE_VIOLATION => {
            AppError::Conflict("policy template already exists".to_owned())
        }
        Some(code) if code == &SqlState::CHECK_VIOLATION => {
            AppError::InvalidParameter("policy template violates a constraint".to_owned())
        }
        _ => AppError::Postgres(error),
    }
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request(auto_approve: bool) -> CreatePolicyTemplateRequest {
        CreatePolicyTemplateRequest {
            key: "analytics-read".to_owned(),
            name: "Analytics read".to_owned(),
            description: None,
            approval_strategy: "ANY".to_owned(),
            auto_approve,
            affected_teams: vec!["analytics".to_owned()],
            approval_teams: if auto_approve {
                Vec::new()
            } else {
                vec![ApprovalTeam {
                    team: "data-owner".to_owned(),
                    required_approvals: 1,
                }]
            },
            grant: PolicyGrant {
                securable_type: "table".to_owned(),
                target_full_name: "main.analytics.orders".to_owned(),
                privilege: "SELECT".to_owned(),
            },
            guardrails: Vec::new(),
            created_by: "alice".to_owned(),
        }
    }

    #[test]
    fn auto_approval_accepts_fixed_resource_without_approval_teams() {
        assert!(validate_create_request(&valid_request(true)).is_ok());
    }

    #[test]
    fn table_rejects_non_table_privilege() {
        let mut request = valid_request(false);
        request.grant.privilege = "USE CATALOG".to_owned();
        let error = validate_create_request(&request).unwrap_err();
        assert_eq!(
            error.to_string(),
            "invalid parameter: privilege USE CATALOG is not valid for securable_type table"
        );
    }

    #[test]
    fn manual_approval_requires_an_approval_team() {
        let mut request = valid_request(false);
        request.approval_teams.clear();

        let error = validate_create_request(&request).unwrap_err();
        assert_eq!(
            error.to_string(),
            "invalid parameter: approval_teams must not be empty when manual approval is required"
        );
    }
}
