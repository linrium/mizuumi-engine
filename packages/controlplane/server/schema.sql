CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS controlplane_metadata (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO controlplane_metadata (singleton)
VALUES (TRUE)
ON CONFLICT DO NOTHING;

CREATE TABLE IF NOT EXISTS data_contracts (
    id TEXT PRIMARY KEY CHECK (BTRIM(id) <> ''),
    catalog_name TEXT NOT NULL CHECK (BTRIM(catalog_name) <> ''),
    schema_name TEXT NOT NULL CHECK (BTRIM(schema_name) <> ''),
    table_name TEXT NOT NULL CHECK (BTRIM(table_name) <> ''),
    active_version TEXT,
    created_by TEXT NOT NULL CHECK (BTRIM(created_by) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (catalog_name, schema_name, table_name)
);

CREATE TABLE IF NOT EXISTS data_contract_versions (
    contract_id TEXT NOT NULL REFERENCES data_contracts(id) ON DELETE CASCADE,
    version TEXT NOT NULL CHECK (BTRIM(version) <> ''),
    document JSONB NOT NULL CHECK (jsonb_typeof(document) = 'object'),
    document_hash TEXT NOT NULL CHECK (BTRIM(document_hash) <> ''),
    schema_hash TEXT NOT NULL CHECK (BTRIM(schema_hash) <> ''),
    created_by TEXT NOT NULL CHECK (BTRIM(created_by) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (contract_id, version)
);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'data_contracts_active_version_fkey'
    ) THEN
        ALTER TABLE data_contracts
            ADD CONSTRAINT data_contracts_active_version_fkey
            FOREIGN KEY (id, active_version)
            REFERENCES data_contract_versions(contract_id, version)
            DEFERRABLE INITIALLY DEFERRED;
    END IF;
END
$$;

CREATE TABLE IF NOT EXISTS data_contract_observations (
    contract_id TEXT PRIMARY KEY REFERENCES data_contracts(id) ON DELETE CASCADE,
    contract_version TEXT NOT NULL,
    unity_table_id TEXT,
    state TEXT NOT NULL CHECK (
        state IN ('PENDING_DEPLOYMENT', 'IN_SYNC', 'DRIFTED', 'TABLE_MISSING', 'ERROR')
    ),
    observed_columns JSONB NOT NULL DEFAULT '[]'::jsonb
        CHECK (jsonb_typeof(observed_columns) = 'array'),
    observed_properties JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(observed_properties) = 'object'),
    observed_schema_hash TEXT,
    differences JSONB NOT NULL DEFAULT '[]'::jsonb
        CHECK (jsonb_typeof(differences) = 'array'),
    checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (contract_id, contract_version)
        REFERENCES data_contract_versions(contract_id, version) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_data_contracts_table
    ON data_contracts (catalog_name, schema_name, table_name);

CREATE TABLE IF NOT EXISTS data_quality_runs (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    contract_id TEXT NOT NULL,
    contract_version TEXT NOT NULL,
    engine TEXT NOT NULL CHECK (engine IN ('DUCKDB', 'SPARK')),
    status TEXT NOT NULL CHECK (status IN ('RUNNING', 'PASSED', 'FAILED', 'ERROR')),
    trigger_type TEXT NOT NULL CHECK (trigger_type IN ('MANUAL', 'MATERIALIZATION', 'SCHEDULE')),
    triggered_by TEXT NOT NULL CHECK (BTRIM(triggered_by) <> ''),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    passed_count INTEGER NOT NULL DEFAULT 0 CHECK (passed_count >= 0),
    failed_count INTEGER NOT NULL DEFAULT 0 CHECK (failed_count >= 0),
    error_count INTEGER NOT NULL DEFAULT 0 CHECK (error_count >= 0),
    error TEXT,
    FOREIGN KEY (contract_id, contract_version)
        REFERENCES data_contract_versions(contract_id, version) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_data_quality_runs_contract
    ON data_quality_runs (contract_id, started_at DESC);

CREATE TABLE IF NOT EXISTS data_quality_results (
    run_id TEXT NOT NULL REFERENCES data_quality_runs(id) ON DELETE CASCADE,
    rule_id TEXT NOT NULL CHECK (BTRIM(rule_id) <> ''),
    rule_scope TEXT NOT NULL CHECK (BTRIM(rule_scope) <> ''),
    rule_type TEXT NOT NULL CHECK (BTRIM(rule_type) <> ''),
    metric TEXT NOT NULL CHECK (BTRIM(metric) <> ''),
    severity TEXT NOT NULL CHECK (BTRIM(severity) <> ''),
    status TEXT NOT NULL CHECK (status IN ('PASSED', 'FAILED', 'ERROR')),
    metric_value JSONB,
    expectation JSONB NOT NULL DEFAULT '{}'::jsonb,
    duration_ms BIGINT NOT NULL DEFAULT 0 CHECK (duration_ms >= 0),
    message TEXT,
    PRIMARY KEY (run_id, rule_id)
);

CREATE TABLE IF NOT EXISTS policy_templates (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    key TEXT NOT NULL UNIQUE CHECK (BTRIM(key) <> ''),
    created_by TEXT NOT NULL CHECK (BTRIM(created_by) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS policy_template_versions (
    template_id TEXT NOT NULL REFERENCES policy_templates(id) ON DELETE CASCADE,
    version INTEGER NOT NULL CHECK (version > 0),
    name TEXT NOT NULL CHECK (BTRIM(name) <> ''),
    description TEXT,
    status TEXT NOT NULL DEFAULT 'DRAFT'
        CHECK (status IN ('DRAFT', 'PUBLISHED', 'RETIRED')),
    approval_strategy TEXT NOT NULL DEFAULT 'ANY'
        CHECK (approval_strategy IN ('ANY', 'ALL')),
    auto_approve BOOLEAN NOT NULL DEFAULT FALSE,
    blast_radius_required BOOLEAN NOT NULL DEFAULT TRUE,
    guardrail_review_required BOOLEAN NOT NULL DEFAULT TRUE,
    created_by TEXT NOT NULL CHECK (BTRIM(created_by) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_by TEXT,
    published_at TIMESTAMPTZ,
    PRIMARY KEY (template_id, version),
    CHECK (
        (status = 'PUBLISHED' AND published_by IS NOT NULL AND published_at IS NOT NULL)
        OR status <> 'PUBLISHED'
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_policy_template_published_version
    ON policy_template_versions (template_id)
    WHERE status = 'PUBLISHED';

CREATE TABLE IF NOT EXISTS policy_template_grants (
    template_id TEXT NOT NULL,
    template_version INTEGER NOT NULL,
    securable_type TEXT NOT NULL CHECK (
        securable_type IN (
            'metastore',
            'catalog',
            'schema',
            'table',
            'function',
            'volume',
            'registered_model',
            'external_location',
            'credential'
        )
    ),
    target_full_name TEXT NOT NULL CHECK (BTRIM(target_full_name) <> ''),
    privilege TEXT NOT NULL CHECK (
        privilege IN (
            'OWNER',
            'BROWSE',
            'CREATE CATALOG',
            'USE CATALOG',
            'CREATE SCHEMA',
            'USE SCHEMA',
            'CREATE TABLE',
            'SELECT',
            'MODIFY',
            'CREATE FUNCTION',
            'EXECUTE',
            'CREATE VOLUME',
            'READ VOLUME',
            'CREATE MODEL',
            'CREATE EXTERNAL LOCATION',
            'READ FILES',
            'WRITE FILES',
            'CREATE EXTERNAL TABLE',
            'CREATE EXTERNAL VOLUME',
            'CREATE MANAGED STORAGE',
            'CREATE STORAGE CREDENTIAL'
        )
    ),
    CHECK (
        (securable_type = 'metastore' AND privilege = 'CREATE CATALOG')
        OR (securable_type = 'catalog' AND privilege IN ('BROWSE', 'USE CATALOG', 'CREATE SCHEMA'))
        OR (securable_type = 'schema' AND privilege IN ('BROWSE', 'USE SCHEMA', 'CREATE TABLE', 'CREATE FUNCTION', 'CREATE VOLUME', 'CREATE MODEL'))
        OR (securable_type = 'table' AND privilege IN ('BROWSE', 'SELECT', 'MODIFY'))
        OR (securable_type IN ('function', 'registered_model') AND privilege = 'EXECUTE')
        OR (securable_type = 'volume' AND privilege = 'READ VOLUME')
        OR (securable_type = 'external_location' AND privilege IN ('READ FILES', 'WRITE FILES', 'CREATE EXTERNAL TABLE', 'CREATE EXTERNAL VOLUME', 'CREATE MANAGED STORAGE'))
        OR (securable_type = 'credential' AND privilege IN ('CREATE EXTERNAL LOCATION', 'CREATE STORAGE CREDENTIAL'))
    ),
    PRIMARY KEY (template_id, template_version),
    UNIQUE (securable_type, target_full_name, privilege),
    FOREIGN KEY (template_id, template_version)
        REFERENCES policy_template_versions(template_id, version)
        ON DELETE CASCADE
);

ALTER TABLE policy_template_grants
    DROP CONSTRAINT IF EXISTS policy_template_grants_privilege_check,
    DROP CONSTRAINT IF EXISTS policy_template_grants_check;

ALTER TABLE policy_template_grants
    ADD CONSTRAINT policy_template_grants_privilege_check CHECK (
        privilege IN (
            'OWNER',
            'BROWSE',
            'CREATE CATALOG',
            'USE CATALOG',
            'CREATE SCHEMA',
            'USE SCHEMA',
            'CREATE TABLE',
            'SELECT',
            'MODIFY',
            'CREATE FUNCTION',
            'EXECUTE',
            'CREATE VOLUME',
            'READ VOLUME',
            'CREATE MODEL',
            'CREATE EXTERNAL LOCATION',
            'READ FILES',
            'WRITE FILES',
            'CREATE EXTERNAL TABLE',
            'CREATE EXTERNAL VOLUME',
            'CREATE MANAGED STORAGE',
            'CREATE STORAGE CREDENTIAL'
        )
    ),
    ADD CONSTRAINT policy_template_grants_check CHECK (
        (securable_type = 'metastore' AND privilege = 'CREATE CATALOG')
        OR (securable_type = 'catalog' AND privilege IN ('BROWSE', 'USE CATALOG', 'CREATE SCHEMA'))
        OR (securable_type = 'schema' AND privilege IN ('BROWSE', 'USE SCHEMA', 'CREATE TABLE', 'CREATE FUNCTION', 'CREATE VOLUME', 'CREATE MODEL'))
        OR (securable_type = 'table' AND privilege IN ('BROWSE', 'SELECT', 'MODIFY'))
        OR (securable_type IN ('function', 'registered_model') AND privilege = 'EXECUTE')
        OR (securable_type = 'volume' AND privilege = 'READ VOLUME')
        OR (securable_type = 'external_location' AND privilege IN ('READ FILES', 'WRITE FILES', 'CREATE EXTERNAL TABLE', 'CREATE EXTERNAL VOLUME', 'CREATE MANAGED STORAGE'))
        OR (securable_type = 'credential' AND privilege IN ('CREATE EXTERNAL LOCATION', 'CREATE STORAGE CREDENTIAL'))
    );

CREATE TABLE IF NOT EXISTS policy_template_affected_teams (
    template_id TEXT NOT NULL,
    template_version INTEGER NOT NULL,
    team TEXT NOT NULL CHECK (BTRIM(team) <> ''),
    PRIMARY KEY (template_id, template_version, team),
    FOREIGN KEY (template_id, template_version)
        REFERENCES policy_template_versions(template_id, version)
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS policy_template_approval_teams (
    template_id TEXT NOT NULL,
    template_version INTEGER NOT NULL,
    team TEXT NOT NULL CHECK (BTRIM(team) <> ''),
    required_approvals INTEGER NOT NULL DEFAULT 1 CHECK (required_approvals > 0),
    PRIMARY KEY (template_id, template_version, team),
    FOREIGN KEY (template_id, template_version)
        REFERENCES policy_template_versions(template_id, version)
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS policy_template_guardrails (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    template_id TEXT NOT NULL,
    template_version INTEGER NOT NULL,
    guardrail_key TEXT NOT NULL CHECK (BTRIM(guardrail_key) <> ''),
    name TEXT NOT NULL CHECK (BTRIM(name) <> ''),
    description TEXT,
    enforcement TEXT NOT NULL DEFAULT 'REQUIRED'
        CHECK (enforcement IN ('REQUIRED', 'ADVISORY')),
    configuration JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(configuration) = 'object'),
    created_by TEXT NOT NULL CHECK (BTRIM(created_by) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (template_id, template_version, guardrail_key),
    FOREIGN KEY (template_id, template_version)
        REFERENCES policy_template_versions(template_id, version)
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS access_requests (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    template_id TEXT,
    template_version INTEGER,
    template_name TEXT NOT NULL,
    approval_strategy TEXT NOT NULL DEFAULT 'ANY'
        CHECK (approval_strategy IN ('ANY', 'ALL')),
    requested_by TEXT NOT NULL CHECK (BTRIM(requested_by) <> ''),
    requested_for TEXT NOT NULL CHECK (BTRIM(requested_for) <> ''),
    routed_for_team TEXT CHECK (routed_for_team IS NULL OR BTRIM(routed_for_team) <> ''),
    target_securable_type TEXT NOT NULL CHECK (
        target_securable_type IN (
            'metastore',
            'catalog',
            'schema',
            'table',
            'function',
            'volume',
            'registered_model',
            'external_location',
            'credential'
        )
    ),
    target_full_name TEXT NOT NULL CHECK (BTRIM(target_full_name) <> ''),
    contract_id TEXT,
    contract_version TEXT,
    contract_document_hash TEXT,
    purpose_code TEXT,
    consumer_context JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(consumer_context) = 'object'),
    contract_snapshot JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(contract_snapshot) = 'object'),
    contract_evaluations JSONB NOT NULL DEFAULT '[]'::jsonb
        CHECK (jsonb_typeof(contract_evaluations) = 'array'),
    contract_evaluation_status TEXT NOT NULL DEFAULT 'NOT_APPLICABLE'
        CHECK (contract_evaluation_status IN ('NOT_APPLICABLE', 'PASSED', 'WARNING')),
    contract_evaluated_at TIMESTAMPTZ,
    reason TEXT NOT NULL CONSTRAINT access_requests_reason_required
        CHECK (BTRIM(reason) <> ''),
    status TEXT NOT NULL DEFAULT 'PENDING' CHECK (
        status IN (
            'PENDING',
            'ASSESSING',
            'GUARDRAIL_REVIEW',
            'AWAITING_APPROVAL',
            'APPROVED',
            'REJECTED',
            'CANCELLED',
            'FULFILLING',
            'FULFILLED',
            'FAILED',
            'EXPIRED'
        )
    ),
    requested_duration_seconds BIGINT NOT NULL
        CONSTRAINT access_requests_duration_required
        CHECK (
            requested_duration_seconds > 0
            AND requested_duration_seconds % 3600 = 0
        ),
    idempotency_key TEXT,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    decided_by TEXT,
    decided_at TIMESTAMPTZ,
    decision_comment TEXT,
    fulfilled_at TIMESTAMPTZ,
    access_expires_at TIMESTAMPTZ,
    FOREIGN KEY (template_id, template_version)
        REFERENCES policy_template_versions(template_id, version),
    CONSTRAINT access_requests_contract_version_fkey
        FOREIGN KEY (contract_id, contract_version)
        REFERENCES data_contract_versions(contract_id, version),
    CHECK (
        (template_id IS NULL AND template_version IS NULL)
        OR (template_id IS NOT NULL AND template_version IS NOT NULL)
    ),
    CHECK (idempotency_key IS NULL OR BTRIM(idempotency_key) <> '')
);

ALTER TABLE access_requests
    ADD COLUMN IF NOT EXISTS contract_id TEXT,
    ADD COLUMN IF NOT EXISTS contract_version TEXT,
    ADD COLUMN IF NOT EXISTS contract_document_hash TEXT,
    ADD COLUMN IF NOT EXISTS purpose_code TEXT,
    ADD COLUMN IF NOT EXISTS consumer_context JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS contract_snapshot JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS contract_evaluations JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS contract_evaluation_status TEXT NOT NULL DEFAULT 'NOT_APPLICABLE',
    ADD COLUMN IF NOT EXISTS contract_evaluated_at TIMESTAMPTZ;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'access_requests_contract_version_fkey'
    ) THEN
        ALTER TABLE access_requests
            ADD CONSTRAINT access_requests_contract_version_fkey
            FOREIGN KEY (contract_id, contract_version)
            REFERENCES data_contract_versions(contract_id, version);
    END IF;
END
$$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'access_requests'::regclass
          AND conname = 'access_requests_reason_required'
    ) THEN
        ALTER TABLE access_requests
            ADD CONSTRAINT access_requests_reason_required
            CHECK (reason IS NOT NULL AND BTRIM(reason) <> '') NOT VALID;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'access_requests'::regclass
          AND conname = 'access_requests_duration_required'
    ) THEN
        ALTER TABLE access_requests
            ADD CONSTRAINT access_requests_duration_required
            CHECK (
                requested_duration_seconds IS NOT NULL
                AND requested_duration_seconds > 0
                AND requested_duration_seconds % 3600 = 0
            ) NOT VALID;
    END IF;
END
$$;

CREATE UNIQUE INDEX IF NOT EXISTS uq_access_request_idempotency
    ON access_requests (requested_by, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_access_request_queue
    ON access_requests (status, submitted_at, id);

CREATE INDEX IF NOT EXISTS idx_access_requests_requested_by
    ON access_requests (requested_by, submitted_at DESC);

CREATE INDEX IF NOT EXISTS idx_access_requests_requested_for
    ON access_requests (requested_for, submitted_at DESC);

CREATE INDEX IF NOT EXISTS idx_access_requests_routed_for_team
    ON access_requests (routed_for_team, submitted_at DESC)
    WHERE routed_for_team IS NOT NULL;

CREATE TABLE IF NOT EXISTS access_request_approval_teams (
    request_id TEXT NOT NULL REFERENCES access_requests(id) ON DELETE CASCADE,
    team TEXT NOT NULL CHECK (BTRIM(team) <> ''),
    required_approvals INTEGER NOT NULL DEFAULT 1 CHECK (required_approvals > 0),
    PRIMARY KEY (request_id, team)
);

CREATE TABLE IF NOT EXISTS access_request_reviews (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    request_id TEXT NOT NULL REFERENCES access_requests(id) ON DELETE CASCADE,
    approval_team TEXT NOT NULL,
    reviewer TEXT NOT NULL CHECK (BTRIM(reviewer) <> ''),
    decision TEXT NOT NULL CHECK (decision IN ('APPROVED', 'REJECTED')),
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (request_id, reviewer),
    FOREIGN KEY (request_id, approval_team)
        REFERENCES access_request_approval_teams(request_id, team)
);

CREATE INDEX IF NOT EXISTS idx_access_request_reviews_team
    ON access_request_reviews (approval_team, created_at DESC);

CREATE TABLE IF NOT EXISTS access_request_grants (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    request_id TEXT NOT NULL REFERENCES access_requests(id) ON DELETE CASCADE,
    securable_type TEXT NOT NULL CHECK (
        securable_type IN (
            'metastore',
            'catalog',
            'schema',
            'table',
            'function',
            'volume',
            'registered_model',
            'external_location',
            'credential'
        )
    ),
    full_name TEXT NOT NULL CHECK (BTRIM(full_name) <> ''),
    privilege TEXT NOT NULL CHECK (
        privilege IN (
            'OWNER',
            'BROWSE',
            'CREATE CATALOG',
            'USE CATALOG',
            'CREATE SCHEMA',
            'USE SCHEMA',
            'CREATE TABLE',
            'SELECT',
            'MODIFY',
            'CREATE FUNCTION',
            'EXECUTE',
            'CREATE VOLUME',
            'READ VOLUME',
            'CREATE MODEL',
            'CREATE EXTERNAL LOCATION',
            'READ FILES',
            'WRITE FILES',
            'CREATE EXTERNAL TABLE',
            'CREATE EXTERNAL VOLUME',
            'CREATE MANAGED STORAGE',
            'CREATE STORAGE CREDENTIAL'
        )
    ),
    CHECK (
        (securable_type = 'metastore' AND privilege = 'CREATE CATALOG')
        OR (securable_type = 'catalog' AND privilege IN ('BROWSE', 'USE CATALOG', 'CREATE SCHEMA'))
        OR (securable_type = 'schema' AND privilege IN ('BROWSE', 'USE SCHEMA', 'CREATE TABLE', 'CREATE FUNCTION', 'CREATE VOLUME', 'CREATE MODEL'))
        OR (securable_type = 'table' AND privilege IN ('BROWSE', 'SELECT', 'MODIFY'))
        OR (securable_type IN ('function', 'registered_model') AND privilege = 'EXECUTE')
        OR (securable_type = 'volume' AND privilege = 'READ VOLUME')
        OR (securable_type = 'external_location' AND privilege IN ('READ FILES', 'WRITE FILES', 'CREATE EXTERNAL TABLE', 'CREATE EXTERNAL VOLUME', 'CREATE MANAGED STORAGE'))
        OR (securable_type = 'credential' AND privilege IN ('CREATE EXTERNAL LOCATION', 'CREATE STORAGE CREDENTIAL'))
    ),
    fulfillment_status TEXT NOT NULL DEFAULT 'PENDING'
        CHECK (fulfillment_status IN ('PENDING', 'GRANTING', 'GRANTED', 'UNCHANGED', 'FAILED', 'REVOKED')),
    fulfilled_at TIMESTAMPTZ,
    failure_message TEXT,
    UNIQUE (request_id, securable_type, full_name, privilege)
);

ALTER TABLE access_request_grants
    DROP CONSTRAINT IF EXISTS access_request_grants_privilege_check,
    DROP CONSTRAINT IF EXISTS access_request_grants_check,
    DROP CONSTRAINT IF EXISTS access_request_grants_fulfillment_status_check;

ALTER TABLE access_request_grants
    ADD CONSTRAINT access_request_grants_privilege_check CHECK (
        privilege IN (
            'OWNER',
            'BROWSE',
            'CREATE CATALOG',
            'USE CATALOG',
            'CREATE SCHEMA',
            'USE SCHEMA',
            'CREATE TABLE',
            'SELECT',
            'MODIFY',
            'CREATE FUNCTION',
            'EXECUTE',
            'CREATE VOLUME',
            'READ VOLUME',
            'CREATE MODEL',
            'CREATE EXTERNAL LOCATION',
            'READ FILES',
            'WRITE FILES',
            'CREATE EXTERNAL TABLE',
            'CREATE EXTERNAL VOLUME',
            'CREATE MANAGED STORAGE',
            'CREATE STORAGE CREDENTIAL'
        )
    ),
    ADD CONSTRAINT access_request_grants_check CHECK (
        (securable_type = 'metastore' AND privilege = 'CREATE CATALOG')
        OR (securable_type = 'catalog' AND privilege IN ('BROWSE', 'USE CATALOG', 'CREATE SCHEMA'))
        OR (securable_type = 'schema' AND privilege IN ('BROWSE', 'USE SCHEMA', 'CREATE TABLE', 'CREATE FUNCTION', 'CREATE VOLUME', 'CREATE MODEL'))
        OR (securable_type = 'table' AND privilege IN ('BROWSE', 'SELECT', 'MODIFY'))
        OR (securable_type IN ('function', 'registered_model') AND privilege = 'EXECUTE')
        OR (securable_type = 'volume' AND privilege = 'READ VOLUME')
        OR (securable_type = 'external_location' AND privilege IN ('READ FILES', 'WRITE FILES', 'CREATE EXTERNAL TABLE', 'CREATE EXTERNAL VOLUME', 'CREATE MANAGED STORAGE'))
        OR (securable_type = 'credential' AND privilege IN ('CREATE EXTERNAL LOCATION', 'CREATE STORAGE CREDENTIAL'))
    ),
    ADD CONSTRAINT access_request_grants_fulfillment_status_check CHECK (
        fulfillment_status IN ('PENDING', 'GRANTING', 'GRANTED', 'UNCHANGED', 'FAILED', 'REVOKED')
    );

CREATE INDEX IF NOT EXISTS idx_access_request_grants_status
    ON access_request_grants (request_id, fulfillment_status);

CREATE TABLE IF NOT EXISTS access_request_guardrails (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    request_id TEXT NOT NULL REFERENCES access_requests(id) ON DELETE CASCADE,
    source_template_guardrail_id BIGINT
        REFERENCES policy_template_guardrails(id) ON DELETE SET NULL,
    guardrail_key TEXT NOT NULL CHECK (BTRIM(guardrail_key) <> ''),
    name TEXT NOT NULL CHECK (BTRIM(name) <> ''),
    description TEXT,
    enforcement TEXT NOT NULL DEFAULT 'REQUIRED'
        CHECK (enforcement IN ('REQUIRED', 'ADVISORY')),
    configuration JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(configuration) = 'object'),
    evaluation_status TEXT NOT NULL DEFAULT 'NOT_EVALUATED'
        CHECK (evaluation_status IN ('NOT_EVALUATED', 'PASSED', 'FAILED', 'WAIVED')),
    evaluation_result JSONB
        CHECK (evaluation_result IS NULL OR jsonb_typeof(evaluation_result) = 'object'),
    evaluated_by TEXT,
    evaluated_at TIMESTAMPTZ,
    UNIQUE (request_id, guardrail_key)
);

CREATE INDEX IF NOT EXISTS idx_access_request_guardrails_status
    ON access_request_guardrails (request_id, evaluation_status);

CREATE TABLE IF NOT EXISTS blast_radius_assessments (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    subject_type TEXT NOT NULL
        CHECK (subject_type IN ('TEMPLATE_VERSION', 'ACCESS_REQUEST')),
    template_id TEXT,
    template_version INTEGER,
    request_id TEXT REFERENCES access_requests(id) ON DELETE CASCADE,
    supersedes_assessment_id TEXT
        REFERENCES blast_radius_assessments(id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'QUEUED'
        CHECK (status IN ('QUEUED', 'RUNNING', 'SUCCEEDED', 'FAILED', 'STALE')),
    risk_level TEXT NOT NULL DEFAULT 'UNKNOWN'
        CHECK (risk_level IN ('UNKNOWN', 'LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    risk_score NUMERIC(5, 2) CHECK (risk_score BETWEEN 0 AND 100),
    summary TEXT,
    agent_name TEXT NOT NULL CHECK (BTRIM(agent_name) <> ''),
    agent_version TEXT,
    model_provider TEXT NOT NULL CHECK (BTRIM(model_provider) <> ''),
    model_name TEXT NOT NULL CHECK (BTRIM(model_name) <> ''),
    prompt_version TEXT,
    input_snapshot JSONB NOT NULL CHECK (jsonb_typeof(input_snapshot) = 'object'),
    raw_output JSONB CHECK (raw_output IS NULL OR jsonb_typeof(raw_output) = 'object'),
    error_message TEXT,
    requested_by TEXT NOT NULL CHECK (BTRIM(requested_by) <> ''),
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    FOREIGN KEY (template_id, template_version)
        REFERENCES policy_template_versions(template_id, version)
        ON DELETE CASCADE,
    CHECK (
        (
            subject_type = 'TEMPLATE_VERSION'
            AND template_id IS NOT NULL
            AND template_version IS NOT NULL
            AND request_id IS NULL
        )
        OR (
            subject_type = 'ACCESS_REQUEST'
            AND template_id IS NULL
            AND template_version IS NULL
            AND request_id IS NOT NULL
        )
    )
);

CREATE INDEX IF NOT EXISTS idx_blast_radius_assessments_request
    ON blast_radius_assessments (request_id, requested_at DESC)
    WHERE request_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_blast_radius_assessments_template
    ON blast_radius_assessments (template_id, template_version, requested_at DESC)
    WHERE template_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_blast_radius_assessment_queue
    ON blast_radius_assessments (status, requested_at, id);

CREATE TABLE IF NOT EXISTS blast_radius_impacts (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    assessment_id TEXT NOT NULL
        REFERENCES blast_radius_assessments(id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    category TEXT NOT NULL CHECK (
        category IN (
            'DATA_EXPOSURE',
            'PRIVILEGE_ESCALATION',
            'COMPLIANCE',
            'OPERATIONAL',
            'FINANCIAL',
            'DEPENDENCY',
            'OTHER'
        )
    ),
    severity TEXT NOT NULL
        CHECK (severity IN ('INFO', 'LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    scope_type TEXT NOT NULL
        CHECK (scope_type IN ('TEAM', 'SECURABLE', 'PRINCIPAL', 'SYSTEM', 'OTHER')),
    scope_identifier TEXT,
    title TEXT NOT NULL CHECK (BTRIM(title) <> ''),
    description TEXT NOT NULL CHECK (BTRIM(description) <> ''),
    evidence JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(evidence) = 'object'),
    confidence NUMERIC(4, 3) CHECK (confidence BETWEEN 0 AND 1),
    UNIQUE (assessment_id, ordinal)
);

CREATE INDEX IF NOT EXISTS idx_blast_radius_impacts_assessment
    ON blast_radius_impacts (assessment_id, severity, ordinal);

CREATE TABLE IF NOT EXISTS blast_radius_guardrail_recommendations (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    assessment_id TEXT NOT NULL
        REFERENCES blast_radius_assessments(id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    guardrail_key TEXT NOT NULL CHECK (BTRIM(guardrail_key) <> ''),
    title TEXT NOT NULL CHECK (BTRIM(title) <> ''),
    description TEXT NOT NULL CHECK (BTRIM(description) <> ''),
    rationale TEXT,
    enforcement TEXT NOT NULL DEFAULT 'ADVISORY'
        CHECK (enforcement IN ('REQUIRED', 'ADVISORY')),
    configuration JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(configuration) = 'object'),
    priority TEXT NOT NULL DEFAULT 'MEDIUM'
        CHECK (priority IN ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    confidence NUMERIC(4, 3) CHECK (confidence BETWEEN 0 AND 1),
    disposition TEXT NOT NULL DEFAULT 'PROPOSED'
        CHECK (disposition IN ('PROPOSED', 'ACCEPTED', 'REJECTED')),
    reviewed_by TEXT,
    reviewed_at TIMESTAMPTZ,
    review_comment TEXT,
    UNIQUE (assessment_id, ordinal),
    UNIQUE (assessment_id, guardrail_key)
);

CREATE INDEX IF NOT EXISTS idx_blast_radius_guardrails_assessment
    ON blast_radius_guardrail_recommendations (assessment_id, disposition, priority);

CREATE TABLE IF NOT EXISTS access_request_events (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    request_id TEXT NOT NULL REFERENCES access_requests(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL CHECK (
        event_type IN (
            'SUBMITTED',
            'UPDATED',
            'APPROVAL_RECORDED',
            'APPROVED',
            'REJECTED',
            'CANCELLED',
            'FULFILLMENT_STARTED',
            'FULFILLMENT_SUCCEEDED',
            'FULFILLMENT_FAILED',
            'EXPIRED',
            'ACCESS_REVOKED'
        )
    ),
    actor TEXT NOT NULL CHECK (BTRIM(actor) <> ''),
    comment TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(metadata) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_access_request_events_request
    ON access_request_events (request_id, created_at, id);

COMMENT ON TABLE policy_template_versions IS
    'Immutable policy definitions; publish a new version instead of editing a published version.';
COMMENT ON TABLE policy_template_grants IS
    'Exactly one fixed securable and privilege belongs to each policy template version.';
COMMENT ON TABLE policy_template_affected_teams IS
    'External team identifiers whose members are eligible to discover and request a template.';
COMMENT ON TABLE policy_template_approval_teams IS
    'External team identifiers and approval quorum used to route request reviews.';
COMMENT ON TABLE access_request_grants IS
    'Expanded grant snapshot captured when a request is submitted, insulated from later template changes.';
COMMENT ON TABLE access_request_approval_teams IS
    'Approval routing snapshot captured when a request is submitted.';
COMMENT ON TABLE access_request_guardrails IS
    'Template or AI-suggested guardrails materialized onto a request for evaluation or waiver.';
COMMENT ON TABLE blast_radius_assessments IS
    'Versioned AI or agent runs with reproducibility inputs, model provenance, and aggregate risk.';
COMMENT ON TABLE blast_radius_impacts IS
    'Structured impacts generated by a blast-radius assessment.';
COMMENT ON TABLE blast_radius_guardrail_recommendations IS
    'AI-generated guardrails that require explicit acceptance or rejection.';
COMMENT ON TABLE access_request_events IS
    'Append-only audit history for request decisions, fulfillment, expiry, and revocation.';

