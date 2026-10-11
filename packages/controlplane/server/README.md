# Control Plane server

The server exposes health, policy-template, and access-request workflow APIs.

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/api/policy-templates` | Create and publish version 1 of a policy template. |
| `GET` | `/api/policy-templates?affected_team=<team>` | List published templates, optionally filtered by affected team. |
| `GET` | `/api/policy-templates/{template_id}` | Get a template with grants, teams, and guardrails. |
| `POST` | `/api/access-requests` | Submit a request and snapshot its grants and approval routing. |
| `GET` | `/api/access-requests` | List requests by status, team, requester, or beneficiary. |
| `GET` | `/api/access-requests/{request_id}` | Get request details, reviews, and events. |
| `GET` | `/api/approval-teams/{team}/access-requests` | List the queue routed to an approval team. |
| `POST` | `/api/access-requests/{request_id}/approve` | Record an approval and apply the template quorum. |
| `POST` | `/api/access-requests/{request_id}/reject` | Reject a pending request. |

Template creation publishes the initial version immediately. Access requests skip blast-radius
analysis for now and enter `AWAITING_APPROVAL`. `ANY` templates are approved when one team's
quorum is met; `ALL` templates require every team's quorum.

Every access request must include a nonblank `reason` and a positive
`requested_duration_seconds` value that does not exceed the template maximum.

The initial API contract accepts `created_by`, `requested_by`, and `reviewer` explicitly. These
fields must be replaced by authenticated principal and team claims when Keycloak middleware is
added; they are not an authorization boundary by themselves.

