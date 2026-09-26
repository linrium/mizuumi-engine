CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS uc_metastore (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    id TEXT NOT NULL UNIQUE DEFAULT gen_random_uuid()::text
);

INSERT INTO uc_metastore (singleton) VALUES (TRUE)
ON CONFLICT DO NOTHING;

CREATE TABLE IF NOT EXISTS uc_catalogs (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    name TEXT NOT NULL UNIQUE,
    comment TEXT,
    properties JSONB NOT NULL DEFAULT '{}'::jsonb,
    owner TEXT,
    created_at BIGINT NOT NULL,
    created_by TEXT,
    updated_at BIGINT,
    updated_by TEXT,
    storage_root TEXT,
    storage_location TEXT
);

CREATE TABLE IF NOT EXISTS uc_schemas (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    catalog_id TEXT NOT NULL REFERENCES uc_catalogs(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    comment TEXT,
    properties JSONB NOT NULL DEFAULT '{}'::jsonb,
    owner TEXT,
    created_at BIGINT NOT NULL,
    created_by TEXT,
    updated_at BIGINT,
    updated_by TEXT,
    storage_root TEXT,
    storage_location TEXT,
    UNIQUE (catalog_id, name)
);

CREATE TABLE IF NOT EXISTS uc_tables (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    schema_id TEXT NOT NULL REFERENCES uc_schemas(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    table_type TEXT NOT NULL,
    data_source_format TEXT,
    columns JSONB NOT NULL DEFAULT '[]'::jsonb,
    storage_location TEXT,
    comment TEXT,
    properties JSONB NOT NULL DEFAULT '{}'::jsonb,
    owner TEXT,
    created_at BIGINT NOT NULL,
    created_by TEXT,
    updated_at BIGINT,
    updated_by TEXT,
    view_definition TEXT,
    view_dependencies JSONB,
    UNIQUE (schema_id, name)
);

CREATE TABLE IF NOT EXISTS uc_credentials (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    name TEXT NOT NULL UNIQUE,
    credential_type TEXT NOT NULL,
    credential JSONB NOT NULL,
    purpose TEXT NOT NULL,
    comment TEXT,
    owner TEXT,
    created_at BIGINT NOT NULL,
    created_by TEXT,
    updated_at BIGINT,
    updated_by TEXT
);

CREATE TABLE IF NOT EXISTS uc_external_locations (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    name TEXT NOT NULL UNIQUE,
    url TEXT NOT NULL UNIQUE,
    credential_id TEXT NOT NULL,
    comment TEXT,
    owner TEXT,
    created_at BIGINT NOT NULL,
    created_by TEXT,
    updated_at BIGINT,
    updated_by TEXT
);

CREATE INDEX IF NOT EXISTS idx_external_locations_credential_id
    ON uc_external_locations (credential_id);

CREATE TABLE IF NOT EXISTS uc_permissions (
    principal TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    securable_type TEXT NOT NULL,
    privilege TEXT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT ((EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint),
    PRIMARY KEY (principal, resource_id, privilege)
);
