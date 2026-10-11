CREATE TABLE IF NOT EXISTS controlplane_metadata (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO controlplane_metadata (singleton)
VALUES (TRUE)
ON CONFLICT DO NOTHING;

