--! create_table
WITH parent_schema AS (
    SELECT
        schemas.id,
        schemas.name AS schema_name,
        catalogs.name AS catalog_name
    FROM uc_schemas schemas
    JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id
    WHERE catalogs.name = :catalog_name
      AND schemas.name = :schema_name
),
next_table AS (
    SELECT
        gen_random_uuid()::text AS id,
        (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint AS now_ms
),
inserted AS (
    INSERT INTO uc_tables (
        id,
        schema_id,
        name,
        table_type,
        data_source_format,
        columns,
        storage_location,
        comment,
        properties,
        owner,
        created_at,
        created_by,
        updated_at,
        updated_by,
        view_definition,
        view_dependencies
    )
    SELECT
        next_table.id,
        parent_schema.id,
        :name,
        :table_type,
        NULLIF(:data_source_format, ''),
        COALESCE(:columns, '[]'::jsonb),
        NULLIF(:storage_location, ''),
        :comment,
        COALESCE(:properties, '{}'::jsonb),
        'system',
        next_table.now_ms,
        'system',
        next_table.now_ms,
        'system',
        NULLIF(:view_definition, ''),
        :view_dependencies
    FROM parent_schema, next_table
    RETURNING *
)
SELECT
    inserted.name,
    parent_schema.catalog_name,
    parent_schema.schema_name,
    inserted.table_type,
    COALESCE(inserted.data_source_format, '') AS data_source_format,
    inserted.columns,
    COALESCE(inserted.storage_location, '') AS storage_location,
    COALESCE(inserted.comment, '') AS comment,
    inserted.properties,
    COALESCE(inserted.owner, '') AS owner,
    inserted.created_at,
    COALESCE(inserted.created_by, '') AS created_by,
    inserted.updated_at,
    COALESCE(inserted.updated_by, '') AS updated_by,
    inserted.id AS table_id,
    COALESCE(inserted.view_definition, '') AS view_definition,
    COALESCE(inserted.view_dependencies, 'null'::jsonb) AS view_dependencies
FROM inserted
JOIN parent_schema ON parent_schema.id = inserted.schema_id;

--! list_tables
SELECT
    tables.name,
    catalogs.name AS catalog_name,
    schemas.name AS schema_name,
    tables.table_type,
    COALESCE(tables.data_source_format, '') AS data_source_format,
    tables.columns,
    COALESCE(tables.storage_location, '') AS storage_location,
    COALESCE(tables.comment, '') AS comment,
    tables.properties,
    COALESCE(tables.owner, '') AS owner,
    tables.created_at,
    COALESCE(tables.created_by, '') AS created_by,
    tables.updated_at,
    COALESCE(tables.updated_by, '') AS updated_by,
    tables.id AS table_id,
    COALESCE(tables.view_definition, '') AS view_definition,
    COALESCE(tables.view_dependencies, 'null'::jsonb) AS view_dependencies
FROM uc_tables tables
JOIN uc_schemas schemas ON schemas.id = tables.schema_id
JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id
WHERE catalogs.name = :catalog_name
  AND schemas.name = :schema_name
  AND (:page_token = '' OR tables.name > :page_token)
ORDER BY tables.name
LIMIT :limit_value;

--! get_table
SELECT
    tables.name,
    catalogs.name AS catalog_name,
    schemas.name AS schema_name,
    tables.table_type,
    COALESCE(tables.data_source_format, '') AS data_source_format,
    tables.columns,
    COALESCE(tables.storage_location, '') AS storage_location,
    COALESCE(tables.comment, '') AS comment,
    tables.properties,
    COALESCE(tables.owner, '') AS owner,
    tables.created_at,
    COALESCE(tables.created_by, '') AS created_by,
    tables.updated_at,
    COALESCE(tables.updated_by, '') AS updated_by,
    tables.id AS table_id,
    COALESCE(tables.view_definition, '') AS view_definition,
    COALESCE(tables.view_dependencies, 'null'::jsonb) AS view_dependencies
FROM uc_tables tables
JOIN uc_schemas schemas ON schemas.id = tables.schema_id
JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id
WHERE catalogs.name = :catalog_name
  AND schemas.name = :schema_name
  AND tables.name = :name;

--! delete_table
DELETE FROM uc_tables tables
USING uc_schemas schemas, uc_catalogs catalogs
WHERE schemas.id = tables.schema_id
  AND catalogs.id = schemas.catalog_id
  AND catalogs.name = :catalog_name
  AND schemas.name = :schema_name
  AND tables.name = :name
RETURNING tables.name;
