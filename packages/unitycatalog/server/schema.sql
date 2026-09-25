CREATE EXTENSION IF NOT EXISTS pgcrypto;

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
