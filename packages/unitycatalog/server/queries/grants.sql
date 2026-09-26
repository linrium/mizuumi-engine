--! get_grant_resource_id
SELECT resource.id
FROM (
    SELECT id, 'metastore'::text AS securable_type, :full_name::text AS full_name
    FROM uc_metastore
    UNION ALL
    SELECT id, 'catalog', name
    FROM uc_catalogs
    UNION ALL
    SELECT schemas.id, 'schema', CONCAT(catalogs.name, '.', schemas.name)
    FROM uc_schemas schemas
    JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id
    UNION ALL
    SELECT tables.id, 'table', CONCAT(catalogs.name, '.', schemas.name, '.', tables.name)
    FROM uc_tables tables
    JOIN uc_schemas schemas ON schemas.id = tables.schema_id
    JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id
    UNION ALL
    SELECT id, 'credential', name
    FROM uc_credentials
    UNION ALL
    SELECT id, 'external_location', name
    FROM uc_external_locations
) resource
WHERE resource.securable_type = :securable_type
  AND resource.full_name = :full_name;

--! grant_permission
INSERT INTO uc_permissions (principal, resource_id, securable_type, privilege)
VALUES (:principal, :resource_id, :securable_type, :privilege)
ON CONFLICT DO NOTHING
RETURNING privilege;

--! revoke_permission
DELETE FROM uc_permissions
WHERE principal = :principal
  AND resource_id = :resource_id
  AND privilege = :privilege
RETURNING privilege;

--! list_permissions
SELECT principal, privilege
FROM uc_permissions
WHERE resource_id = :resource_id
  AND (:principal = '' OR principal = :principal)
ORDER BY principal, privilege;
