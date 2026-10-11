--! create_policy_template
INSERT INTO policy_templates (key, created_by)
VALUES (:key::text, :created_by::text)
RETURNING id;

--! create_policy_template_version
INSERT INTO policy_template_versions (
    template_id,
    version,
    name,
    description,
    status,
    approval_strategy,
    blast_radius_required,
    guardrail_review_required,
    default_duration_seconds,
    max_duration_seconds,
    created_by,
    published_by,
    published_at
)
VALUES (
    :template_id::text,
    1,
    :name::text,
    NULLIF(:description::text, ''),
    'PUBLISHED',
    :approval_strategy::text,
    FALSE,
    FALSE,
    NULLIF(:default_duration_seconds::bigint, 0),
    NULLIF(:max_duration_seconds::bigint, 0),
    :created_by::text,
    :created_by::text,
    NOW()
);

--! create_policy_template_grant
INSERT INTO policy_template_grants (
    template_id,
    template_version,
    securable_type,
    target_kind,
    target_full_name,
    privilege
)
VALUES (
    :template_id::text,
    1,
    :securable_type::text,
    :target_kind::text,
    :target_full_name::text,
    :privilege::text
);

--! create_policy_template_affected_team
INSERT INTO policy_template_affected_teams (template_id, template_version, team)
VALUES (:template_id::text, 1, :team::text);

--! create_policy_template_approval_team
INSERT INTO policy_template_approval_teams (
    template_id,
    template_version,
    team,
    required_approvals
)
VALUES (:template_id::text, 1, :team::text, :required_approvals);

--! create_policy_template_guardrail
INSERT INTO policy_template_guardrails (
    template_id,
    template_version,
    guardrail_key,
    name,
    description,
    enforcement,
    configuration,
    created_by
)
VALUES (
    :template_id::text,
    1,
    :guardrail_key::text,
    :name::text,
    NULLIF(:description::text, ''),
    :enforcement::text,
    :configuration,
    :created_by::text
);

--! list_policy_templates
SELECT
    templates.id,
    templates.key,
    versions.version,
    versions.name,
    COALESCE(versions.description, '') AS description,
    versions.approval_strategy,
    COALESCE(versions.default_duration_seconds, 0) AS default_duration_seconds,
    COALESCE(versions.max_duration_seconds, 0) AS max_duration_seconds,
    templates.created_by,
    templates.created_at::text AS created_at
FROM policy_templates templates
JOIN policy_template_versions versions
  ON versions.template_id = templates.id
 AND versions.status = 'PUBLISHED'
WHERE (
    :affected_team::text = ''
    OR EXISTS (
        SELECT 1
        FROM policy_template_affected_teams affected
        WHERE affected.template_id = versions.template_id
          AND affected.template_version = versions.version
          AND affected.team = :affected_team::text
    )
)
ORDER BY templates.key;

--! get_policy_template
SELECT
    templates.id,
    templates.key,
    versions.version,
    versions.name,
    COALESCE(versions.description, '') AS description,
    versions.status,
    versions.approval_strategy,
    versions.blast_radius_required,
    versions.guardrail_review_required,
    COALESCE(versions.default_duration_seconds, 0) AS default_duration_seconds,
    COALESCE(versions.max_duration_seconds, 0) AS max_duration_seconds,
    templates.created_by,
    templates.created_at::text AS created_at,
    COALESCE(versions.published_by, '') AS published_by,
    COALESCE(versions.published_at::text, '') AS published_at
FROM policy_templates templates
JOIN policy_template_versions versions ON versions.template_id = templates.id
WHERE templates.id = :template_id::text
ORDER BY versions.version DESC
LIMIT 1;

--! list_policy_template_grants
SELECT securable_type, target_kind, target_full_name, privilege
FROM policy_template_grants
WHERE template_id = :template_id::text
  AND template_version = :template_version
ORDER BY securable_type, target_kind, target_full_name, privilege;

--! list_policy_template_affected_teams
SELECT team
FROM policy_template_affected_teams
WHERE template_id = :template_id::text
  AND template_version = :template_version
ORDER BY team;

--! list_policy_template_approval_teams
SELECT team, required_approvals
FROM policy_template_approval_teams
WHERE template_id = :template_id::text
  AND template_version = :template_version
ORDER BY team;

--! list_policy_template_guardrails
SELECT
    guardrail_key,
    name,
    COALESCE(description, '') AS description,
    enforcement,
    configuration
FROM policy_template_guardrails
WHERE template_id = :template_id::text
  AND template_version = :template_version
ORDER BY guardrail_key;

