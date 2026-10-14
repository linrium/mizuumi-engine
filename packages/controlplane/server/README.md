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
| `GET` | `/api/duck-server/metrics` | Fetch DuckDB query-profiler time series from SigNoz. |
| `GET` | `/api/computes` | List Duck Server pods and SparkApplication jobs from Kubernetes. |
| `GET` | `/api/computes/{compute_id}` | Fetch one compute by Kubernetes UID. |
| `GET` | `/api/computes/{compute_id}/metrics` | Fetch DuckDB or Spark Flare metrics scoped to one compute. |

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

`GET /api/duck-server/metrics` queries the fixed set of `duckdb.query.*`
histograms through SigNoz's `/api/v5/query_range` API. It accepts optional
`start` and `end` Unix timestamps in milliseconds, `step` in seconds, and
`aggregation` (`p50`, `p90`, `p95`, or `p99`; default `p95`). The maximum range
is seven days. Create a read-only SigNoz service-account API key, save its
one-time value to the ignored, mode-600 file
`packages/controlplane/signoz-api-key`, and bootstrap the Kubernetes Secret
without exposing the key to clients:

```sh
./scripts/controlplane/bootstrap.sh
```

`SIGNOZ_API_KEY_FILE` can override the default path, and `SIGNOZ_API_KEY` is
also accepted for local automation. `scripts/controlplane/bootstrap.sh` creates
the Secret before deploying the control plane and reuses a populated Secret on
later runs, so the one-time key is only required on the first bootstrap or when
intentionally rotating it.

The chart reads that optional Secret into `CONTROLPLANE__SIGNOZ__API_KEY`. When
the key is absent, the endpoint returns `503` while the rest of the control
plane remains available.

Compute discovery reads Duck Server pods in the configured `[compute]` namespace
and SparkApplication custom resources in the namespaces allowed by the chart's
`rbac.sparkApplications` settings. The Helm chart grants the control-plane service
account read-only access to both resource types. Duck metrics use the pod UID
exported as the `k8s.pod.uid` OpenTelemetry resource attribute.

Spark compute metrics query the nine `flare.task.*` and `flare.stage.*`
instruments emitted by the application's driver and executor services. Histogram
series use the requested percentile; byte counters use per-interval increases.
The SparkApplication name must match the Flare service-name convention
`<application>-driver` and `<application>-executor`, which the repository's Spark
deployment scripts configure automatically.

