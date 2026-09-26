--! get_table_storage_location
SELECT storage_location
FROM uc_tables
WHERE id = :table_id::text
  AND NULLIF(storage_location, '') IS NOT NULL;

--! get_volume_storage_location
SELECT storage_location
FROM uc_volumes
WHERE id = :volume_id::text
  AND NULLIF(storage_location, '') IS NOT NULL;

--! get_model_version_storage_location
SELECT
    model_versions.url AS storage_location,
    model_versions.status
FROM uc_model_versions model_versions
JOIN uc_registered_models models ON models.id = model_versions.registered_model_id
JOIN uc_schemas schemas ON schemas.id = models.schema_id
JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id
WHERE catalogs.name = :catalog_name::text
  AND schemas.name = :schema_name::text
  AND models.name = :model_name::text
  AND model_versions.version = :version
  AND NULLIF(model_versions.url, '') IS NOT NULL;
