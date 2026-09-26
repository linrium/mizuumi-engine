--! create_external_location
WITH next_location AS (
    SELECT
        gen_random_uuid()::text AS id,
        (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint AS now_ms
),
inserted AS (
    INSERT INTO uc_external_locations (
        id,
        name,
        url,
        credential_id,
        comment,
        owner,
        created_at,
        created_by,
        updated_at,
        updated_by
    )
    SELECT
        next_location.id,
        :name::text,
        :url::text,
        credentials.id,
        :comment::text,
        'system',
        next_location.now_ms,
        'system',
        next_location.now_ms,
        'system'
    FROM next_location
    JOIN uc_credentials credentials ON credentials.name = :credential_name::text
    RETURNING *
)
SELECT
    inserted.name,
    inserted.id,
    inserted.url,
    credentials.name AS credential_name,
    COALESCE(inserted.comment, '') AS comment,
    COALESCE(inserted.owner, '') AS owner,
    inserted.credential_id,
    inserted.created_at,
    COALESCE(inserted.created_by, '') AS created_by,
    inserted.updated_at,
    COALESCE(inserted.updated_by, '') AS updated_by
FROM inserted
JOIN uc_credentials credentials ON credentials.id = inserted.credential_id;

--! list_external_locations
SELECT
    locations.name,
    locations.id,
    locations.url,
    COALESCE(credentials.name, '') AS credential_name,
    COALESCE(locations.comment, '') AS comment,
    COALESCE(locations.owner, '') AS owner,
    locations.credential_id,
    locations.created_at,
    COALESCE(locations.created_by, '') AS created_by,
    locations.updated_at,
    COALESCE(locations.updated_by, '') AS updated_by
FROM uc_external_locations locations
LEFT JOIN uc_credentials credentials ON credentials.id = locations.credential_id
WHERE (:page_token::text = '' OR locations.name > :page_token::text)
ORDER BY locations.name
LIMIT :limit_value;

--! get_external_location
SELECT
    locations.name,
    locations.id,
    locations.url,
    COALESCE(credentials.name, '') AS credential_name,
    COALESCE(locations.comment, '') AS comment,
    COALESCE(locations.owner, '') AS owner,
    locations.credential_id,
    locations.created_at,
    COALESCE(locations.created_by, '') AS created_by,
    locations.updated_at,
    COALESCE(locations.updated_by, '') AS updated_by
FROM uc_external_locations locations
LEFT JOIN uc_credentials credentials ON credentials.id = locations.credential_id
WHERE locations.name = :name::text;

--! update_external_location
WITH updated AS (
    UPDATE uc_external_locations locations
    SET
        name = :new_name::text,
        url = :url::text,
        credential_id = credentials.id,
        comment = :comment::text,
        updated_at = (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint,
        updated_by = 'system'
    FROM uc_credentials credentials
    WHERE locations.name = :name::text
      AND credentials.name = :credential_name::text
    RETURNING locations.*
)
SELECT
    updated.name,
    updated.id,
    updated.url,
    credentials.name AS credential_name,
    COALESCE(updated.comment, '') AS comment,
    COALESCE(updated.owner, '') AS owner,
    updated.credential_id,
    updated.created_at,
    COALESCE(updated.created_by, '') AS created_by,
    updated.updated_at,
    COALESCE(updated.updated_by, '') AS updated_by
FROM updated
JOIN uc_credentials credentials ON credentials.id = updated.credential_id;

--! find_overlapping_external_location
SELECT name
FROM uc_external_locations
WHERE (:exclude_id::text = '' OR id <> :exclude_id::text)
  AND (
      url = :url::text
      OR starts_with(:url::text, url || CASE WHEN right(url, 1) = '/' THEN '' ELSE '/' END)
      OR starts_with(url, :url::text || CASE WHEN right(:url::text, 1) = '/' THEN '' ELSE '/' END)
  )
LIMIT 1;

--! find_external_table_using_location
SELECT tables.id
FROM uc_tables tables
WHERE tables.storage_location = :url::text
   OR starts_with(
       tables.storage_location,
       :url::text || CASE WHEN right(:url::text, 1) = '/' THEN '' ELSE '/' END
   )
LIMIT 1;

--! delete_external_location
WITH deleted AS (
    DELETE FROM uc_external_locations
    WHERE name = :name::text
    RETURNING id
),
deleted_permissions AS (
    DELETE FROM uc_permissions
    WHERE resource_id IN (SELECT id FROM deleted)
)
SELECT id FROM deleted;

--! find_external_location_using_credential
SELECT locations.name
FROM uc_external_locations locations
JOIN uc_credentials credentials ON credentials.id = locations.credential_id
WHERE credentials.name = :credential_name::text
LIMIT 1;

--! find_external_location_for_path
SELECT url
FROM uc_external_locations
WHERE url = :url::text
   OR starts_with(
       :url::text,
       url || CASE WHEN right(url, 1) = '/' THEN '' ELSE '/' END
   )
ORDER BY length(url) DESC
LIMIT 1;
