--! find_data_contract
SELECT id, COALESCE(active_version, '') AS active_version
FROM data_contracts
WHERE catalog_name = :catalog_name::text
  AND schema_name = :schema_name::text
  AND table_name = :table_name::text;

--! create_data_contract
INSERT INTO data_contracts (
    id, catalog_name, schema_name, table_name, created_by
)
VALUES (
    :id::text,
    :catalog_name::text,
    :schema_name::text,
    :table_name::text,
    :created_by::text
);

--! get_data_contract_version
SELECT document, document_hash, schema_hash, created_by, created_at::text AS created_at
FROM data_contract_versions
WHERE contract_id = :contract_id::text
  AND version = :version::text;

--! create_data_contract_version
INSERT INTO data_contract_versions (
    contract_id, version, document, document_hash, schema_hash, created_by
)
VALUES (
    :contract_id::text,
    :version::text,
    :document,
    encode(digest((:document)::jsonb::text, 'sha256'), 'hex'),
    encode(digest((:normalized_schema)::jsonb::text, 'sha256'), 'hex'),
    :created_by::text
);

--! activate_data_contract_version
UPDATE data_contracts
SET active_version = :version::text,
    updated_at = NOW()
WHERE id = :contract_id::text;

--! set_pending_data_contract_observation
INSERT INTO data_contract_observations (
    contract_id, contract_version, state
)
VALUES (
    :contract_id::text, :contract_version::text, 'PENDING_DEPLOYMENT'
)
ON CONFLICT (contract_id) DO UPDATE
SET contract_version = EXCLUDED.contract_version,
    unity_table_id = NULL,
    state = EXCLUDED.state,
    observed_columns = '[]'::jsonb,
    observed_properties = '{}'::jsonb,
    observed_schema_hash = NULL,
    differences = '[]'::jsonb,
    checked_at = NOW();

--! get_active_data_contract
SELECT
    contracts.id,
    contracts.catalog_name,
    contracts.schema_name,
    contracts.table_name,
    versions.version,
    versions.document,
    versions.document_hash,
    versions.schema_hash,
    versions.created_by,
    versions.created_at::text AS created_at,
    COALESCE(observations.state, 'PENDING_DEPLOYMENT') AS sync_state,
    COALESCE(observations.unity_table_id, '') AS unity_table_id,
    COALESCE(observations.observed_schema_hash, '') AS observed_schema_hash,
    COALESCE(observations.differences, '[]'::jsonb) AS differences,
    COALESCE(observations.checked_at::text, '') AS checked_at
FROM data_contracts contracts
JOIN data_contract_versions versions
  ON versions.contract_id = contracts.id
 AND versions.version = contracts.active_version
LEFT JOIN data_contract_observations observations
  ON observations.contract_id = contracts.id
WHERE contracts.catalog_name = :catalog_name::text
  AND contracts.schema_name = :schema_name::text
  AND contracts.table_name = :table_name::text;

--! upsert_data_contract_observation
INSERT INTO data_contract_observations (
    contract_id,
    contract_version,
    unity_table_id,
    state,
    observed_columns,
    observed_properties,
    observed_schema_hash,
    differences,
    checked_at
)
VALUES (
    :contract_id::text,
    :contract_version::text,
    NULLIF(:unity_table_id::text, ''),
    :state::text,
    :observed_columns,
    :observed_properties,
    CASE
        WHEN jsonb_typeof((:normalized_schema)::jsonb) = 'array'
        THEN encode(digest((:normalized_schema)::jsonb::text, 'sha256'), 'hex')
        ELSE NULL
    END,
    :differences,
    NOW()
)
ON CONFLICT (contract_id) DO UPDATE
SET contract_version = EXCLUDED.contract_version,
    unity_table_id = EXCLUDED.unity_table_id,
    state = EXCLUDED.state,
    observed_columns = EXCLUDED.observed_columns,
    observed_properties = EXCLUDED.observed_properties,
    observed_schema_hash = EXCLUDED.observed_schema_hash,
    differences = EXCLUDED.differences,
    checked_at = NOW();

--! create_data_quality_run
INSERT INTO data_quality_runs (
    contract_id, contract_version, engine, status, trigger_type, triggered_by
)
VALUES (
    :contract_id::text,
    :contract_version::text,
    'DUCKDB',
    'RUNNING',
    'MANUAL',
    :triggered_by::text
)
RETURNING id, started_at::text AS started_at;

--! create_data_quality_result
INSERT INTO data_quality_results (
    run_id,
    rule_id,
    rule_scope,
    rule_type,
    metric,
    severity,
    status,
    metric_value,
    expectation,
    duration_ms,
    message
)
VALUES (
    :run_id::text,
    :rule_id::text,
    :rule_scope::text,
    :rule_type::text,
    :metric::text,
    :severity::text,
    :status::text,
    :metric_value,
    :expectation,
    :duration_ms,
    NULLIF(:message::text, '')
);

--! complete_data_quality_run
UPDATE data_quality_runs
SET status = :status::text,
    completed_at = NOW(),
    passed_count = :passed_count,
    failed_count = :failed_count,
    error_count = :error_count,
    error = NULLIF(:error::text, '')
WHERE id = :run_id::text;

--! list_data_quality_runs
SELECT
    runs.id,
    runs.contract_id,
    runs.contract_version,
    runs.engine,
    runs.status,
    runs.trigger_type,
    runs.triggered_by,
    runs.started_at::text AS started_at,
    COALESCE(runs.completed_at::text, '') AS completed_at,
    runs.passed_count,
    runs.failed_count,
    runs.error_count,
    COALESCE(runs.error, '') AS error
FROM data_quality_runs runs
JOIN data_contracts contracts ON contracts.id = runs.contract_id
WHERE contracts.catalog_name = :catalog_name::text
  AND contracts.schema_name = :schema_name::text
  AND contracts.table_name = :table_name::text
ORDER BY runs.started_at DESC
LIMIT :result_limit;

--! list_data_quality_results
SELECT
    rule_id,
    rule_scope,
    rule_type,
    metric,
    severity,
    status,
    metric_value,
    expectation,
    duration_ms,
    COALESCE(message, '') AS message
FROM data_quality_results
WHERE run_id = :run_id::text
ORDER BY rule_scope, rule_id;
