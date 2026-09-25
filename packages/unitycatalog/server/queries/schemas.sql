--! create_schema
WITH catalog AS (
    SELECT id, name
    FROM uc_catalogs
    WHERE name = :catalog_name
),
next_schema AS (
    SELECT
        gen_random_uuid()::text AS id,
        (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint AS now_ms
),
inserted AS (
    INSERT INTO uc_schemas (
        id,
        catalog_id,
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
        next_schema.id,
        catalog.id,
        :name,
        :comment,
        COALESCE(:properties, '{}'::jsonb),
        'system',
        next_schema.now_ms,
        'system',
        next_schema.now_ms,
        'system',
        NULLIF(:storage_root, ''),
        CASE
            WHEN :storage_root = '' THEN NULL
            ELSE CONCAT(:storage_root, '/__unitystorage/schemas/', next_schema.id)
        END
    FROM catalog, next_schema
    RETURNING *
)
SELECT
    inserted.name,
    catalog.name AS catalog_name,
    COALESCE(inserted.comment, '') AS comment,
    inserted.properties,
    CONCAT(catalog.name, '.', inserted.name) AS full_name,
    COALESCE(inserted.owner, '') AS owner,
    inserted.created_at,
    COALESCE(inserted.created_by, '') AS created_by,
    inserted.updated_at,
    COALESCE(inserted.updated_by, '') AS updated_by,
    inserted.id AS schema_id,
    COALESCE(inserted.storage_root, '') AS storage_root,
    COALESCE(inserted.storage_location, '') AS storage_location
FROM inserted
JOIN catalog ON catalog.id = inserted.catalog_id;

--! list_schemas
SELECT
    schemas.name,
    catalogs.name AS catalog_name,
    COALESCE(schemas.comment, '') AS comment,
    schemas.properties,
    CONCAT(catalogs.name, '.', schemas.name) AS full_name,
    COALESCE(schemas.owner, '') AS owner,
    schemas.created_at,
    COALESCE(schemas.created_by, '') AS created_by,
    schemas.updated_at,
    COALESCE(schemas.updated_by, '') AS updated_by,
    schemas.id AS schema_id,
    COALESCE(schemas.storage_root, '') AS storage_root,
    COALESCE(schemas.storage_location, '') AS storage_location
FROM uc_schemas schemas
JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id
WHERE catalogs.name = :catalog_name
  AND (:page_token = '' OR schemas.name > :page_token)
ORDER BY schemas.name
LIMIT :limit_value;

--! get_schema
SELECT
    schemas.name,
    catalogs.name AS catalog_name,
    COALESCE(schemas.comment, '') AS comment,
    schemas.properties,
    CONCAT(catalogs.name, '.', schemas.name) AS full_name,
    COALESCE(schemas.owner, '') AS owner,
    schemas.created_at,
    COALESCE(schemas.created_by, '') AS created_by,
    schemas.updated_at,
    COALESCE(schemas.updated_by, '') AS updated_by,
    schemas.id AS schema_id,
    COALESCE(schemas.storage_root, '') AS storage_root,
    COALESCE(schemas.storage_location, '') AS storage_location
FROM uc_schemas schemas
JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id
WHERE catalogs.name = :catalog_name
  AND schemas.name = :name;

--! update_schema
UPDATE uc_schemas schemas
SET
    name = COALESCE(:new_name, schemas.name),
    comment = COALESCE(:comment, schemas.comment),
    properties = COALESCE(:properties, schemas.properties),
    updated_at = (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint,
    updated_by = 'system'
FROM uc_catalogs catalogs
WHERE catalogs.id = schemas.catalog_id
  AND catalogs.name = :catalog_name
  AND schemas.name = :name
RETURNING
    schemas.name,
    catalogs.name AS catalog_name,
    COALESCE(schemas.comment, '') AS comment,
    schemas.properties,
    CONCAT(catalogs.name, '.', schemas.name) AS full_name,
    COALESCE(schemas.owner, '') AS owner,
    schemas.created_at,
    COALESCE(schemas.created_by, '') AS created_by,
    schemas.updated_at,
    COALESCE(schemas.updated_by, '') AS updated_by,
    schemas.id AS schema_id,
    COALESCE(schemas.storage_root, '') AS storage_root,
    COALESCE(schemas.storage_location, '') AS storage_location;

--! delete_schema
DELETE FROM uc_schemas schemas
USING uc_catalogs catalogs
WHERE catalogs.id = schemas.catalog_id
  AND catalogs.name = :catalog_name
  AND schemas.name = :name
RETURNING schemas.name;
