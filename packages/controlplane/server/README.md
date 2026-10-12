# Control Plane server

The server exposes health, policy-template, and access-request workflow APIs.

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/api/policy-templates` | Create and publish version 1 of a policy template. |
| `GET` | `/api/policy-templates?affected_team=<team>` | List published templates, optionally filtered by affected team. |
| `GET` | `/api/policy-templates/{template_id}` | Get a template with its fixed grant, teams, and guardrails. |
| `POST` | `/api/access-requests` | Submit a request, resolve its policy, and snapshot grants and approval routing. |
| `GET` | `/api/access-requests` | List requests by status, team, requester, or beneficiary. |
| `GET` | `/api/access-requests/{request_id}` | Get request details, reviews, and events. |
| `PATCH` | `/api/access-requests/{request_id}` | Edit an awaiting request and clear partial reviews. |
| `GET` | `/api/approval-teams/{team}/access-requests` | List the queue routed to an approval team. |
| `POST` | `/api/access-requests/{request_id}/approve` | Record an approval and apply the template quorum. |
| `POST` | `/api/access-requests/{request_id}/reject` | Reject a pending request. |
| `POST` | `/api/access-requests/{request_id}/cancel` | Cancel an awaiting request owned by the requester. |

Template creation publishes the initial version immediately. Manual templates route matching
requests to `AWAITING_APPROVAL`: `ANY` templates are approved when one team's quorum is met and
`ALL` templates require every team's quorum. Each template has exactly one fixed securable and
privilege and may set `auto_approve`; matching requests then enter `APPROVED` immediately without
approval teams. The database prevents duplicate templates for the same securable type, full name,
and privilege.

Every access request must include a nonblank `reason` and a positive `requested_duration_seconds`
value in whole-hour increments. The duration starts when the request is approved. Approval and
auto-approval synchronously apply the snapshotted grants to Unity Catalog for `requested_for`, then
move the request through `FULFILLING` to `FULFILLED`. Failed API calls set the request and grants to
`FAILED`. A durable `GRANTING` marker lets reconciliation revoke a permission even when the remote
mutation succeeds but its response is lost. Request reads reconcile elapsed grants, revoke
control-plane-managed Unity Catalog permissions, and move the request to `EXPIRED`. Permissions
that existed before the request are recorded as `UNCHANGED` and are never revoked; overlapping
active requests also retain access until the last managed grant expires.

Unity Catalog synchronization uses a Keycloak client-credentials service account with the
`unitycatalog-admin` role. Configure `[unity_catalog]` (`url`, `token_url`, `client_id`, and
`client_secret`) or the equivalent `CONTROLPLANE__UNITY_CATALOG__*` environment variables. The
local bootstrap stores the generated client credentials in the `unitycatalog-credentials` secret.

The request API resolves the published policy from `routed_for_team`, `target_securable_type`,
`target_full_name`, and `requested_privilege`; callers do not supply a template ID. When no policy
matches, the request remains manual and is routed to the `sovicoSecurity` team with one required
approval.

`BROWSE` is a valid requested or policy-template privilege for catalogs, schemas, and tables. It
only exposes metadata in Unity Catalog; data access still requires `SELECT` and the normal catalog
and schema usage privileges.

The initial API contract accepts `created_by`, `requested_by`, and `reviewer` explicitly. These
fields must be replaced by authenticated principal and team claims when Keycloak middleware is
added; they are not an authorization boundary by themselves.

