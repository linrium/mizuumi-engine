--! create_credential
WITH next_credential AS (
    SELECT
        gen_random_uuid()::text AS id,
        gen_random_uuid()::text AS external_id,
        (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint AS now_ms
),
inserted AS (
    INSERT INTO uc_credentials (
        id,
        name,
        credential_type,
        credential,
        purpose,
        comment,
        owner,
        created_at,
        created_by,
        updated_at,
        updated_by
    )
    SELECT
        next_credential.id,
        :name::text,
        'AWS_IAM_ROLE',
        jsonb_build_object('role_arn', :role_arn::text, 'external_id', next_credential.external_id),
        :purpose::text,
        :comment::text,
        'system',
        next_credential.now_ms,
        'system',
        next_credential.now_ms,
        'system'
    FROM next_credential
    RETURNING *
)
SELECT
    name,
    credential,
    COALESCE(comment, '') AS comment,
    COALESCE(owner, '') AS owner,
    name AS full_name,
    id,
    created_at,
    COALESCE(created_by, '') AS created_by,
    updated_at,
    COALESCE(updated_by, '') AS updated_by,
    purpose
FROM inserted;

--! list_credentials
SELECT
    name,
    credential,
    COALESCE(comment, '') AS comment,
    COALESCE(owner, '') AS owner,
    name AS full_name,
    id,
    created_at,
    COALESCE(created_by, '') AS created_by,
    updated_at,
    COALESCE(updated_by, '') AS updated_by,
    purpose
FROM uc_credentials
WHERE (:page_token::text = '' OR name > :page_token::text)
  AND (:purpose::text = '' OR purpose = :purpose::text)
ORDER BY name
LIMIT :limit_value;

--! get_credential
SELECT
    name,
    credential,
    COALESCE(comment, '') AS comment,
    COALESCE(owner, '') AS owner,
    name AS full_name,
    id,
    created_at,
    COALESCE(created_by, '') AS created_by,
    updated_at,
    COALESCE(updated_by, '') AS updated_by,
    purpose
FROM uc_credentials
WHERE name = :name::text;

--! update_credential
UPDATE uc_credentials
SET
    name = COALESCE(:new_name::text, name),
    credential = CASE
        WHEN :role_arn::text = '' THEN credential
        ELSE jsonb_build_object('role_arn', :role_arn::text, 'external_id', gen_random_uuid()::text)
    END,
    comment = COALESCE(:comment::text, comment),
    updated_at = (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint,
    updated_by = 'system'
WHERE name = :name::text
RETURNING
    name,
    credential,
    COALESCE(comment, '') AS comment,
    COALESCE(owner, '') AS owner,
    name AS full_name,
    id,
    created_at,
    COALESCE(created_by, '') AS created_by,
    updated_at,
    COALESCE(updated_by, '') AS updated_by,
    purpose;

--! delete_credential
DELETE FROM uc_credentials
WHERE name = :name::text
RETURNING id;
