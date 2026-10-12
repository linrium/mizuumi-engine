--! resolve_request_policy
SELECT
    versions.template_id,
    versions.version,
    versions.name,
    versions.approval_strategy,
    versions.auto_approve
FROM policy_template_versions versions
JOIN policy_template_affected_teams affected
  ON affected.template_id = versions.template_id
 AND affected.template_version = versions.version
JOIN policy_template_grants policy_grant
  ON policy_grant.template_id = versions.template_id
 AND policy_grant.template_version = versions.version
WHERE versions.status = 'PUBLISHED'
  AND affected.team = :team::text
  AND policy_grant.securable_type = :target_securable_type::text
  AND policy_grant.target_full_name = :target_full_name::text
  AND policy_grant.privilege = :privilege::text;

--! create_access_request
INSERT INTO access_requests (
    template_id,
    template_version,
    template_name,
    approval_strategy,
    requested_by,
    requested_for,
    routed_for_team,
    target_securable_type,
    target_full_name,
    reason,
    status,
    requested_duration_seconds,
    idempotency_key,
    decided_by,
    decided_at,
    decision_comment,
    access_expires_at
)
VALUES (
    NULLIF(:template_id::text, ''),
    CASE WHEN :template_id::text = '' THEN NULL ELSE :template_version::integer END,
    :template_name::text,
    :approval_strategy::text,
    :requested_by::text,
    :requested_for::text,
    :routed_for_team::text,
    :target_securable_type::text,
    :target_full_name::text,
    :reason::text,
    CASE WHEN :auto_approve::boolean THEN 'APPROVED' ELSE 'AWAITING_APPROVAL' END,
    :requested_duration_seconds::bigint,
    NULLIF(:idempotency_key::text, ''),
    CASE WHEN :auto_approve::boolean THEN 'system:auto-approve' END,
    CASE WHEN :auto_approve::boolean THEN NOW() END,
    CASE WHEN :auto_approve::boolean THEN 'Automatically approved by policy template' END,
    CASE
        WHEN :auto_approve::boolean
        THEN NOW() + (:requested_duration_seconds::bigint * INTERVAL '1 second')
    END
)
RETURNING id;

--! copy_access_request_approval_teams
INSERT INTO access_request_approval_teams (request_id, team, required_approvals)
SELECT :request_id::text, team, required_approvals
FROM policy_template_approval_teams
WHERE template_id = :template_id::text
  AND template_version = :template_version;

--! copy_access_request_grants
INSERT INTO access_request_grants (request_id, securable_type, full_name, privilege)
SELECT
    :request_id::text,
    securable_type,
    target_full_name,
    privilege
FROM policy_template_grants
WHERE template_id = :template_id::text
  AND template_version = :template_version;

--! create_access_request_approval_team
INSERT INTO access_request_approval_teams (request_id, team, required_approvals)
VALUES (:request_id::text, :team::text, 1);

--! create_access_request_grant
INSERT INTO access_request_grants (request_id, securable_type, full_name, privilege)
VALUES (
    :request_id::text,
    :securable_type::text,
    :full_name::text,
    :privilege::text
);

--! copy_access_request_guardrails
INSERT INTO access_request_guardrails (
    request_id,
    source_template_guardrail_id,
    guardrail_key,
    name,
    description,
    enforcement,
    configuration
)
SELECT
    :request_id::text,
    id,
    guardrail_key,
    name,
    description,
    enforcement,
    configuration
FROM policy_template_guardrails
WHERE template_id = :template_id::text
  AND template_version = :template_version;

--! count_access_request_grants
SELECT COUNT(*)::bigint AS count
FROM access_request_grants
WHERE request_id = :request_id::text;

--! count_access_request_approval_teams
SELECT COUNT(*)::bigint AS count
FROM access_request_approval_teams
WHERE request_id = :request_id::text;

--! create_access_request_event
INSERT INTO access_request_events (request_id, event_type, actor, comment)
VALUES (
    :request_id::text,
    :event_type::text,
    :actor::text,
    NULLIF(:comment::text, '')
);

--! list_access_requests
SELECT
    requests.id,
    COALESCE(requests.template_id, '') AS template_id,
    COALESCE(requests.template_version, 0) AS template_version,
    requests.template_name,
    requests.requested_by,
    requests.requested_for,
    COALESCE(requests.routed_for_team, '') AS routed_for_team,
    requests.target_securable_type,
    requests.target_full_name,
    requests.reason,
    requests.status,
    requests.requested_duration_seconds,
    requests.submitted_at::text AS submitted_at,
    COALESCE(requests.decided_by, '') AS decided_by,
    COALESCE(requests.decided_at::text, '') AS decided_at,
    COALESCE(requests.decision_comment, '') AS decision_comment
FROM access_requests requests
WHERE (:status::text = '' OR requests.status = :status::text)
  AND (:requested_by::text = '' OR requests.requested_by = :requested_by::text)
  AND (:requested_for::text = '' OR requests.requested_for = :requested_for::text)
  AND (
      :approval_team::text = ''
      OR EXISTS (
          SELECT 1
          FROM access_request_approval_teams approval
          WHERE approval.request_id = requests.id
            AND approval.team = :approval_team::text
      )
  )
ORDER BY requests.submitted_at DESC, requests.id;

--! expire_access_requests
WITH expired AS (
    UPDATE access_requests
    SET status = 'EXPIRED'
    WHERE status IN ('APPROVED', 'FULFILLED')
      AND access_expires_at <= NOW()
    RETURNING id
)
INSERT INTO access_request_events (request_id, event_type, actor, comment)
SELECT id, 'EXPIRED', 'system:expiry', 'Requested access duration elapsed'
FROM expired;

--! get_access_request
SELECT
    requests.id,
    COALESCE(requests.template_id, '') AS template_id,
    COALESCE(requests.template_version, 0) AS template_version,
    requests.template_name,
    requests.approval_strategy,
    requests.requested_by,
    requests.requested_for,
    COALESCE(requests.routed_for_team, '') AS routed_for_team,
    requests.target_securable_type,
    requests.target_full_name,
    requests.reason,
    requests.status,
    requests.requested_duration_seconds,
    requests.submitted_at::text AS submitted_at,
    COALESCE(requests.decided_by, '') AS decided_by,
    COALESCE(requests.decided_at::text, '') AS decided_at,
    COALESCE(requests.decision_comment, '') AS decision_comment,
    COALESCE(requests.fulfilled_at::text, '') AS fulfilled_at,
    COALESCE(requests.access_expires_at::text, '') AS access_expires_at
FROM access_requests requests
WHERE requests.id = :request_id::text;

--! lock_access_request
SELECT requests.status, requests.approval_strategy
FROM access_requests requests
WHERE requests.id = :request_id::text
FOR UPDATE OF requests;

--! lock_owned_access_request
SELECT status
FROM access_requests
WHERE id = :request_id::text
  AND requested_by = :requested_by::text
FOR UPDATE;

--! clear_access_request_reviews
DELETE FROM access_request_reviews
WHERE request_id = :request_id::text;

--! update_access_request
UPDATE access_requests
SET reason = :reason::text,
    requested_duration_seconds = :requested_duration_seconds::bigint
WHERE id = :request_id::text;

--! revoke_access_request
UPDATE access_requests
SET status = 'CANCELLED',
    decided_by = :requested_by::text,
    decided_at = NOW(),
    decision_comment = 'Access revoked by requester',
    access_expires_at = CASE
        WHEN access_expires_at IS NULL THEN NOW()
        ELSE LEAST(access_expires_at, NOW())
    END
WHERE id = :request_id::text;

--! list_access_request_approval_teams
SELECT
    teams.team,
    teams.required_approvals,
    (
        SELECT COUNT(*)::int4
        FROM access_request_reviews reviews
        WHERE reviews.request_id = teams.request_id
          AND reviews.approval_team = teams.team
          AND reviews.decision = 'APPROVED'
    ) AS approved_count,
    (
        SELECT COUNT(*)::int4
        FROM access_request_reviews reviews
        WHERE reviews.request_id = teams.request_id
          AND reviews.approval_team = teams.team
          AND reviews.decision = 'REJECTED'
    ) AS rejected_count
FROM access_request_approval_teams teams
WHERE teams.request_id = :request_id::text
ORDER BY team;

--! list_access_request_grants
SELECT securable_type, full_name, privilege, fulfillment_status
FROM access_request_grants
WHERE request_id = :request_id::text
ORDER BY securable_type, full_name, privilege;

--! list_access_request_guardrails
SELECT
    guardrail_key,
    name,
    COALESCE(description, '') AS description,
    enforcement,
    configuration,
    evaluation_status
FROM access_request_guardrails
WHERE request_id = :request_id::text
ORDER BY guardrail_key;

--! list_access_request_reviews
SELECT
    approval_team,
    reviewer,
    decision,
    COALESCE(comment, '') AS comment,
    created_at::text AS created_at
FROM access_request_reviews
WHERE request_id = :request_id::text
ORDER BY created_at, id;

--! list_access_request_events
SELECT
    event_type,
    actor,
    COALESCE(comment, '') AS comment,
    created_at::text AS created_at
FROM access_request_events
WHERE request_id = :request_id::text
ORDER BY created_at, id;

--! is_access_request_approval_team
SELECT EXISTS (
    SELECT 1
    FROM access_request_approval_teams
    WHERE request_id = :request_id::text
      AND team = :approval_team::text
) AS assigned;

--! create_access_request_review
INSERT INTO access_request_reviews (
    request_id,
    approval_team,
    reviewer,
    decision,
    comment
)
VALUES (
    :request_id::text,
    :approval_team::text,
    :reviewer::text,
    :decision::text,
    NULLIF(:comment::text, '')
);

--! is_access_request_fully_approved
SELECT CASE requests.approval_strategy
    WHEN 'ALL' THEN NOT EXISTS (
        SELECT 1
        FROM access_request_approval_teams teams
        WHERE teams.request_id = requests.id
          AND (
              SELECT COUNT(*)
              FROM access_request_reviews reviews
              WHERE reviews.request_id = requests.id
                AND reviews.approval_team = teams.team
                AND reviews.decision = 'APPROVED'
          ) < teams.required_approvals
    )
    ELSE EXISTS (
        SELECT 1
        FROM access_request_approval_teams teams
        WHERE teams.request_id = requests.id
          AND (
              SELECT COUNT(*)
              FROM access_request_reviews reviews
              WHERE reviews.request_id = requests.id
                AND reviews.approval_team = teams.team
                AND reviews.decision = 'APPROVED'
          ) >= teams.required_approvals
    )
END AS approved
FROM access_requests requests
WHERE requests.id = :request_id::text;

--! approve_access_request
UPDATE access_requests
SET status = 'APPROVED',
    decided_by = :reviewer::text,
    decided_at = NOW(),
    decision_comment = NULLIF(:comment::text, ''),
    access_expires_at = NOW() + (requested_duration_seconds * INTERVAL '1 second')
WHERE id = :request_id::text;

--! reject_access_request
UPDATE access_requests
SET status = 'REJECTED',
    decided_by = :reviewer::text,
    decided_at = NOW(),
    decision_comment = NULLIF(:comment::text, '')
WHERE id = :request_id::text;

