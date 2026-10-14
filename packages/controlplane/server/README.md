# Control Plane server

The server exposes health, ODCS data-contract, policy-template, and access-request
workflow APIs.

| Method | Path | Purpose |
| --- | --- | --- |
| `PUT` | `/api/data-contracts/{catalog.schema.table}/versions/{version}` | Validate, store, and activate an immutable ODCS v3.1.0 table contract. |
| `GET` | `/api/data-contracts/{catalog.schema.table}` | Get the active contract and its latest reconciliation state. |
| `POST` | `/api/data-contracts/{catalog.schema.table}/reconcile` | Compare the active contract with the observed Unity Catalog table. |
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

Data contracts are stored as complete JSONB documents with indexed table bindings,
immutable semantic versions, full-document and physical-schema SHA-256 hashes, and the
latest Unity Catalog observation. The initial implementation supports one physical table
object per ODCS document. Reconciliation checks ordered column names, physical types,
nullability, partition positions, and the `mizuumi.odcs.*` Delta table properties.

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

Table requests additionally require `purpose_code`, `consumer_context` (consumer domain,
processing countries, cross-border-transfer and AI-use declarations), and
`acknowledged_contract_terms: true`. Submission resolves the active contract on the server and
freezes its ID, version, document hash, applicable terms, and evaluation evidence. Missing or
inactive contracts, schema drift, prohibited processing locations, and contract duration limits
block submission. Quality state and declared AI use are advisory warnings that disable policy
auto-approval. Approval and fulfillment recheck that the bound version is still active and in
sync so a changed contract cannot silently authorize a Unity Catalog grant.

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

Access-request handlers append audit records to Chronicle when `[chronicle].enabled` is true.
Configure `endpoint`, `service_name`, and `timeout_ms` in `config.toml`, or use the equivalent
`CONTROLPLANE__CHRONICLE__*` environment variables. Audit records include the authenticated
principal, action-specific request fields, timestamps, and the success or error outcome.
Committed access-request lifecycle events are also appended with event-specific actions such as
`access_requests.submitted`, `access_requests.fulfillment_succeeded`, and
`access_requests.expired`, plus the request ID and event actor.

The initial API contract accepts `created_by`, `requested_by`, and `reviewer` explicitly. These
fields must be replaced by authenticated principal and team claims when Keycloak middleware is
added; they are not an authorization boundary by themselves.

