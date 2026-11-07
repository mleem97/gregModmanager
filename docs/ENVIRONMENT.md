# Environment configuration

[`.env.example`](../.env.example) inventories the active application settings,
CI inputs and proposed integration settings. It contains no working credentials.
`ACTIVE` means the named component reads the variable today; `RESERVED` means
implementation is still required. The template does not enable a synchronization
service or new Discord notifications.

The Rust app and CLI do not automatically read `.env`. Export only the settings
you need before launching them. Empty optional overrides can suppress defaults;
leave commented overrides unset unless you supply a real value. Store CI secrets
in Jenkins/Forgejo credentials and explicitly bind them to jobs. GitHub remains
a read-only code archive; its Issues API is a separate reporting destination.
Never embed service administration or sync credentials in distributed binaries.

## Telemetry and Discord

The active sender in `crates/greg-platform/src/telemetry.rs` reads `GREG_LOKI_URL`,
`GREG_LOKI_USER`, `GREG_LOKI_PASS` and `GREG_LOKI_TENANT`. To use the requested
canonical deployment names today, explicitly export the following mappings:

| Deployment setting | Current Rust setting | Meaning |
| --- | --- | --- |
| `TELEMETRY_ENDPOINT` | `GREG_LOKI_URL` | Full Loki push URL ending in `/loki/api/v1/push` |
| `TELEMETRY_USER` | `GREG_LOKI_USER` | Basic-auth user |
| `TELEMETRY_API_KEY` | `GREG_LOKI_PASS` | Basic-auth token/password |
| `TELEMETRY_TENANT` | `GREG_LOKI_TENANT` | `X-Scope-OrgID` header |

Sending remains opt-in through `GREG_TELEMETRY_ENABLED` and application
preferences. Grafana dashboards visualize Loki data; their browser URL is not a
Loki ingestion endpoint. Local build mode forces the localhost Loki push URL.

The former release workflow used `DISCORD_WEBHOOK_URL`. The current Jenkins
quality/build matrix has no notification dispatcher. Separate reserved
URLs cover public full releases, internal workflow status, build metrics, quality,
errors, telemetry summaries and sync health. A future notification dispatcher must
route each embed explicitly, exclude prereleases from the public destination,
redact secrets and personal data, respect Discord rate limits, and aggregate
metrics rather than sending an embed for every event. Filling these values alone
does not activate a notification workflow.

## MantisNext, Plane, GitHub and Bugsink

The target was inspected at
`/mnt/win/Repositories/Webapps/MantisNext/mantis-next/`, including its environment
template, deployment documentation, REST issues route and integration worker.
No secrets were read from its local `.env`, and no target files were modified.

| Component | Configuration ownership | Current behavior |
| --- | --- | --- |
| Report Assistant | Desktop: `GREG_GITHUB_*` | Creates a GitHub issue with configured token/repository; API origin has a default |
| MantisNext client | Proposed bridge: `MANTIS_NEXT_*` | Target supports Bearer API tokens and project-scoped `GET`/`POST /api/v1/issues`; this app has no client yet |
| Plane mirror | MantisNext web + separate worker: `PLANE_*` | Existing adapter mirrors public issue title, description and priority; project UUID mapping is configured in `/integrations` |
| Forgejo events | MantisNext: `FORGEJO_WEBHOOK_SECRET` | Signed receiver `/api/webhooks/forgejo`; repository mapping in `/integrations` |
| Jenkins events | Sender + MantisNext: `JENKINS_WEBHOOK_TOKEN` | Bearer receiver `/api/webhooks/jenkins`; payload needs `repository`, `build_number`, `result`, `build_url`, optional `commit_sha` |
| GitHub mirror | Proposed bridge: `GITHUB_*` | No bidirectional connector or webhook receiver exists here |
| Bugsink | Proposed ingestion/bridge: `BUGSINK_*`, optional `SENTRY_DSN` | No active Rust SDK/connector; DSN fallback and API/webhook handling must be implemented and checked against the deployed server |

Provision a MantisNext account token with `issues:read` and `issues:write` and
appropriate project access. The target token alone does not bypass membership
checks. Record project UUIDs and optional category/version UUIDs. Configure Plane
and Forgejo mappings through MantisNext's Integrations screen; `PLANE_PROJECT_ID`
is a proposed provisioning input, not a setting read by MantisNext today.

Run the MantisNext integration worker as a separate persistent process with its
own deployment monitoring. `INTEGRATION_WORKER_ENABLED=true` is only an operator
indicator; it does not start the worker. MantisNext database/session credentials
(`DATABASE_URL`, `BETTER_AUTH_SECRET`, `BETTER_AUTH_URL`) belong to that server's
own environment template and are not client integration credentials.

### Requirements for a full 1:1 mirror

The current systems cannot deliver a complete mirror through configuration alone.
Before enabling `ISSUE_SYNC_ENABLED`, implement and verify:

- A bridge with authenticated provider clients and verified inbound webhooks or
  durable polling; validate Bugsink's deployed API and supported event types.
- Durable project/issue mappings across all providers, delivery deduplication,
  origin tracking to prevent loops, checkpoints, retries and failed-delivery review.
- Explicit mappings for states, users, priorities, labels, comments, attachments,
  visibility and deletion behavior. Unsupported fields need a documented policy;
  no provider should silently lose data or leak private reports.
- Conflict review and initial backfill/reconciliation. The target currently lacks
  REST update/delete issue routes, complete pagination for backfills, and Plane
  state/assignee/comment mappings. Extend these before claiming a complete mirror.
- Tests across deployed service versions for create/update, duplicate delivery,
  outage/retry, simultaneous edits and private-issue handling, followed by dry-run
  reconciliation before live writes.

No new dependency or external service is activated by this documentation change.
Relevant documentation is tracked under `docs/`; no separate wiki update is
needed. Future implementation should update this page, the target's integration
architecture and its environment-variable documentation together.
