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
