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
