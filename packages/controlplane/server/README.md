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
value in whole-hour increments. The duration starts when the request is approved; the resulting
`access_expires_at` timestamp is returned with request details and must be propagated to the target
system during fulfillment. Request reads reconcile elapsed approved or fulfilled requests to the
`EXPIRED` status and append an expiry event.

The request API resolves the published policy from `routed_for_team`, `target_securable_type`,
`target_full_name`, and `requested_privilege`; callers do not supply a template ID. When no policy
matches, the request remains manual and is routed to the `sovicoSecurity` team with one required
approval.

The initial API contract accepts `created_by`, `requested_by`, and `reviewer` explicitly. These
fields must be replaced by authenticated principal and team claims when Keycloak middleware is
added; they are not an authorization boundary by themselves.

