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
const PRIVILEGES: &[&str] = &[
    "OWNER",
    "CREATE CATALOG",
    "USE CATALOG",
    "CREATE SCHEMA",
    "USE SCHEMA",
    "CREATE TABLE",
    "SELECT",
    "MODIFY",
    "CREATE FUNCTION",
    "EXECUTE",
    "CREATE VOLUME",
    "READ VOLUME",
    "CREATE MODEL",
    "CREATE EXTERNAL LOCATION",
    "READ FILES",
    "WRITE FILES",
    "CREATE EXTERNAL TABLE",
    "CREATE EXTERNAL VOLUME",
    "CREATE MANAGED STORAGE",
    "CREATE STORAGE CREDENTIAL",
];

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
                &request.default_duration_seconds.unwrap_or_default(),
                &request.max_duration_seconds.unwrap_or_default(),
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
        for grant in &request.grants {
            queries::create_policy_template_grant()
                .bind(
                    &transaction,
                    &template_id,
                    &grant.securable_type,
                    &grant.target_kind,
                    &grant.target_full_name,
                    &grant.privilege,
                )
                .await
                .map_err(map_policy_write_error)?;
        }
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
                default_duration_seconds: non_zero(row.default_duration_seconds),
                max_duration_seconds: non_zero(row.max_duration_seconds),
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
    let grants = queries::list_policy_template_grants()
        .bind(client, &template_id, &version)
        .all()
        .await?
        .into_iter()
        .map(|grant| PolicyGrant {
            securable_type: grant.securable_type,
            target_kind: grant.target_kind,
            target_full_name: grant.target_full_name,
            privilege: grant.privilege,
        })
        .collect();
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
        blast_radius_required: row.blast_radius_required,
        guardrail_review_required: row.guardrail_review_required,
        default_duration_seconds: non_zero(row.default_duration_seconds),
        max_duration_seconds: non_zero(row.max_duration_seconds),
        affected_teams,
        approval_teams,
        grants,
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
    if request.approval_teams.is_empty() {
        return Err(AppError::InvalidParameter(
            "approval_teams must not be empty".to_owned(),
        ));
    }
    if request.grants.is_empty() {
        return Err(AppError::InvalidParameter(
            "grants must not be empty".to_owned(),
        ));
    }
    if let Some(duration) = request.default_duration_seconds
        && duration <= 0
    {
        return Err(AppError::InvalidParameter(
            "default_duration_seconds must be positive".to_owned(),
        ));
    }
    if let Some(duration) = request.max_duration_seconds
        && duration <= 0
    {
        return Err(AppError::InvalidParameter(
            "max_duration_seconds must be positive".to_owned(),
        ));
    }
    if let (Some(default), Some(max)) = (
        request.default_duration_seconds,
        request.max_duration_seconds,
    ) && default > max
    {
        return Err(AppError::InvalidParameter(
            "default_duration_seconds must not exceed max_duration_seconds".to_owned(),
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
    for grant in &request.grants {
        if !SECURABLE_TYPES.contains(&grant.securable_type.as_str()) {
            return Err(AppError::InvalidParameter(format!(
                "unsupported securable_type: {}",
                grant.securable_type
            )));
        }
        if !PRIVILEGES.contains(&grant.privilege.as_str()) {
            return Err(AppError::InvalidParameter(format!(
                "unsupported privilege: {}",
                grant.privilege
            )));
        }
        match grant.target_kind.as_str() {
            "REQUEST_RESOURCE" if grant.target_full_name.is_empty() => {}
            "FIXED_RESOURCE" if !grant.target_full_name.trim().is_empty() => {}
            _ => {
                return Err(AppError::InvalidParameter(
                    "REQUEST_RESOURCE grants require an empty target_full_name; FIXED_RESOURCE grants require one"
                        .to_owned(),
                ));
            }
        }
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

fn non_zero(value: i64) -> Option<i64> {
    (value != 0).then_some(value)
}
