--! create_catalog
WITH next_catalog AS (
    SELECT
        gen_random_uuid()::text AS id,
        (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint AS now_ms
)
INSERT INTO uc_catalogs (
    id,
    name,
    comment,
    properties,
    owner,
    created_at,
    created_by,
    updated_at,
    updated_by,
    storage_root,
    storage_location
)
SELECT
    next_catalog.id,
    :name,
    :comment,
    COALESCE(:properties, '{}'::jsonb),
    'system',
    next_catalog.now_ms,
    'system',
    next_catalog.now_ms,
    'system',
    NULLIF(:storage_root, ''),
    CASE
        WHEN :storage_root = '' THEN NULL
        ELSE CONCAT(:storage_root, '/__unitystorage/catalogs/', next_catalog.id)
    END
FROM next_catalog
RETURNING
    name,
    COALESCE(comment, '') AS comment,
    properties,
    COALESCE(owner, '') AS owner,
    created_at,
    COALESCE(created_by, '') AS created_by,
    updated_at,
    COALESCE(updated_by, '') AS updated_by,
    id,
    COALESCE(storage_root, '') AS storage_root,
    COALESCE(storage_location, '') AS storage_location;

--! list_catalogs
SELECT
    name,
    COALESCE(comment, '') AS comment,
    properties,
    COALESCE(owner, '') AS owner,
    created_at,
    COALESCE(created_by, '') AS created_by,
    updated_at,
    COALESCE(updated_by, '') AS updated_by,
    id,
    COALESCE(storage_root, '') AS storage_root,
    COALESCE(storage_location, '') AS storage_location
FROM uc_catalogs
WHERE (:page_token = '' OR name > :page_token)
ORDER BY name
LIMIT :limit_value;

--! get_catalog
SELECT
    name,
    COALESCE(comment, '') AS comment,
    properties,
    COALESCE(owner, '') AS owner,
    created_at,
    COALESCE(created_by, '') AS created_by,
    updated_at,
    COALESCE(updated_by, '') AS updated_by,
    id,
    COALESCE(storage_root, '') AS storage_root,
    COALESCE(storage_location, '') AS storage_location
FROM uc_catalogs
WHERE name = :name;

--! update_catalog
UPDATE uc_catalogs
SET
    name = COALESCE(:new_name, name),
    comment = COALESCE(:comment, comment),
    properties = COALESCE(:properties, properties),
    updated_at = (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint,
    updated_by = 'system'
WHERE name = :name
RETURNING
    name,
    COALESCE(comment, '') AS comment,
    properties,
    COALESCE(owner, '') AS owner,
    created_at,
    COALESCE(created_by, '') AS created_by,
    updated_at,
    COALESCE(updated_by, '') AS updated_by,
    id,
    COALESCE(storage_root, '') AS storage_root,
    COALESCE(storage_location, '') AS storage_location;

--! delete_catalog
DELETE FROM uc_catalogs
WHERE name = :name
RETURNING name;
