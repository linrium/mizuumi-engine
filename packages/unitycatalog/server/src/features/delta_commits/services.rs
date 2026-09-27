use async_trait::async_trait;
use deadpool_postgres::Pool;
use serde_json::Value;
use tokio_postgres::{IsolationLevel, Row};

use crate::error::AppError;

use super::dtos::{
    DeltaCommitInfo, DeltaCommitRequest, DeltaGetCommitsRequest, DeltaGetCommitsResponse,
};

const MAX_UNBACKFILLED_COMMITS: i64 = 10;

#[async_trait]
pub trait DeltaCommitService: Send + Sync {
    async fn get_commits(
        &self,
        request: DeltaGetCommitsRequest,
    ) -> Result<DeltaGetCommitsResponse, AppError>;
    async fn post_commit(&self, request: DeltaCommitRequest) -> Result<(), AppError>;
}

pub struct DefaultDeltaCommitService {
    pool: Pool,
}

impl DefaultDeltaCommitService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeltaCommitService for DefaultDeltaCommitService {
    async fn get_commits(
        &self,
        request: DeltaGetCommitsRequest,
    ) -> Result<DeltaGetCommitsResponse, AppError> {
        if request
            .end_version
            .is_some_and(|end| end < request.start_version)
        {
            return Err(AppError::InvalidParameter(
                "end_version must be greater than or equal to start_version".to_string(),
            ));
        }

        let mut client = self.pool.get().await?;
        let transaction = client
            .build_transaction()
            .isolation_level(IsolationLevel::RepeatableRead)
            .read_only(true)
            .start()
            .await?;
        let table = transaction
            .query_opt(
                "SELECT table_type, COALESCE(data_source_format, '') AS data_source_format, \
                        storage_location FROM uc_tables WHERE id = $1",
                &[&request.table_id],
            )
            .await?;
        validate_table(table, &request.table_id, &request.table_uri)?;

        let state = transaction
            .query_opt(
                "SELECT latest_table_version FROM uc_delta_table_state WHERE table_id = $1",
                &[&request.table_id],
            )
            .await?;
        let latest_table_version = state.map_or(0, |row| row.get("latest_table_version"));
        let rows = transaction
            .query(
                "SELECT commit_info FROM uc_delta_commits \
                 WHERE table_id = $1 AND is_backfilled = FALSE AND version >= $2 \
                   AND ($3::bigint IS NULL OR version <= $3) \
                 ORDER BY version LIMIT 10",
                &[
                    &request.table_id,
                    &request.start_version,
                    &request.end_version,
                ],
            )
            .await?;
        let commits = rows
            .into_iter()
            .map(|row| serde_json::from_value::<DeltaCommitInfo>(row.get::<_, Value>(0)))
            .collect::<Result<Vec<_>, _>>()?;
        transaction.commit().await?;

        Ok(DeltaGetCommitsResponse {
            commits,
            latest_table_version,
        })
    }

    async fn post_commit(&self, request: DeltaCommitRequest) -> Result<(), AppError> {
        if request.commit_info.is_none() && request.latest_backfilled_version.is_none() {
            return Err(AppError::InvalidParameter(
                "commit_info or latest_backfilled_version is required".to_string(),
            ));
        }
        if let Some(info) = &request.commit_info {
            if info.version < 1 || info.file_name.is_empty() || info.file_size < 0 {
                return Err(AppError::InvalidParameter(
                    "commit_info requires version >= 1, a file_name, and nonnegative file_size"
                        .to_string(),
                ));
            }
        }

        let mut client = self.pool.get().await?;
        let transaction = client.transaction().await?;
        let table = transaction
            .query_opt(
                "SELECT table_type, COALESCE(data_source_format, '') AS data_source_format, \
                        storage_location FROM uc_tables WHERE id = $1 FOR UPDATE",
                &[&request.table_id],
            )
            .await?;
        validate_table(table, &request.table_id, &request.table_uri)?;

        transaction
            .execute(
                "INSERT INTO uc_delta_table_state (table_id) VALUES ($1) ON CONFLICT DO NOTHING",
                &[&request.table_id],
            )
            .await?;
        let state = transaction
            .query_one(
                "SELECT latest_table_version, latest_backfilled_version \
                 FROM uc_delta_table_state WHERE table_id = $1 FOR UPDATE",
                &[&request.table_id],
            )
            .await?;
        let mut latest_table_version: i64 = state.get("latest_table_version");
        let mut latest_backfilled_version: i64 = state.get("latest_backfilled_version");
        if let Some(version) = request.latest_backfilled_version {
            let last_possible_version =
                latest_table_version + i64::from(request.commit_info.is_some());
            if version < 0 || version < latest_backfilled_version || version > last_possible_version
            {
                return Err(AppError::InvalidParameter(
                    "latest_backfilled_version must be between the previous backfilled version and latest_table_version"
                        .to_string(),
                ));
            }
        }

        if let Some(info) = &request.commit_info {
            if info.version <= latest_table_version {
                let existing = transaction
                    .query_opt(
                        "SELECT commit_info FROM uc_delta_commits \
                         WHERE table_id = $1 AND version = $2",
                        &[&request.table_id, &info.version],
                    )
                    .await?;
                if existing.is_some_and(|row| {
                    row.get::<_, Value>(0)["file_name"].as_str() == Some(info.file_name.as_str())
                }) {
                    return Ok(());
                }
                return Err(AppError::Conflict(format!(
                    "Delta commit version {} already exists",
                    info.version
                )));
            }
            if info.version != latest_table_version + 1 {
                return Err(AppError::InvalidParameter(format!(
                    "expected Delta commit version {}",
                    latest_table_version + 1
                )));
            }
            let unbackfilled: i64 = transaction
                .query_one(
                    "SELECT count(*) FROM uc_delta_commits \
                     WHERE table_id = $1 AND is_backfilled = FALSE AND version > $2",
                    &[
                        &request.table_id,
                        &request
                            .latest_backfilled_version
                            .unwrap_or(latest_backfilled_version),
                    ],
                )
                .await?
                .get(0);
            if unbackfilled >= MAX_UNBACKFILLED_COMMITS {
                return Err(AppError::TooManyRequests(
                    "backfill Delta commits before committing another version".to_string(),
                ));
            }

            let commit_info = serde_json::to_value(info)?;
            transaction
                .execute(
                    "INSERT INTO uc_delta_commits (table_id, version, commit_info) \
                     VALUES ($1, $2, $3)",
                    &[&request.table_id, &info.version, &commit_info],
                )
                .await?;
            latest_table_version = info.version;
        }

        if let Some(version) = request.latest_backfilled_version {
            latest_backfilled_version = version;
            transaction
                .execute(
                    "UPDATE uc_delta_commits SET is_backfilled = TRUE \
                     WHERE table_id = $1 AND version <= $2",
                    &[&request.table_id, &version],
                )
                .await?;
        }

        if let Some(metadata) = &request.metadata {
            let columns = metadata
                .schema
                .as_ref()
                .and_then(|schema| schema.columns.as_ref())
                .map(serde_json::to_value)
                .transpose()?;
            let properties = metadata
                .properties
                .as_ref()
                .and_then(|properties| properties.properties.as_ref())
                .map(serde_json::to_value)
                .transpose()?;
            transaction
                .execute(
                    "UPDATE uc_tables \
                     SET comment = COALESCE($2, comment), \
                         columns = COALESCE($3::jsonb, columns), \
                         properties = COALESCE($4::jsonb, properties), \
                         updated_at = (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint \
                     WHERE id = $1",
                    &[
                        &request.table_id,
                        &metadata.description,
                        &columns,
                        &properties,
                    ],
                )
                .await?;
        }

        transaction
            .execute(
                "UPDATE uc_delta_table_state \
                 SET latest_table_version = $2, latest_backfilled_version = $3, \
                     uniform = COALESCE($4::jsonb, uniform) \
                 WHERE table_id = $1",
                &[
                    &request.table_id,
                    &latest_table_version,
                    &latest_backfilled_version,
                    &request.uniform,
                ],
            )
            .await?;
        transaction.commit().await?;
        Ok(())
    }
}

fn validate_table(row: Option<Row>, table_id: &str, table_uri: &str) -> Result<(), AppError> {
    let row = row.ok_or_else(|| AppError::NotFound(format!("table {table_id}")))?;
    let table_type: String = row.get("table_type");
    let format: String = row.get("data_source_format");
    if table_type != "MANAGED" || (format != "DELTA" && !format.is_empty()) {
        return Err(AppError::FailedPrecondition(
            "Delta commits require a MANAGED DELTA table".to_string(),
        ));
    }
    let storage_location: Option<String> = row.get("storage_location");
    if storage_location
        .as_deref()
        .map(|value| value.trim_end_matches('/'))
        != Some(table_uri.trim_end_matches('/'))
    {
        return Err(AppError::InvalidParameter(
            "table_uri does not match the table's storage_location".to_string(),
        ));
    }
    Ok(())
}
