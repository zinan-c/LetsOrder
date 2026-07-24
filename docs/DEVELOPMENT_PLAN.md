# LetsOrder Software Development Plan

Document version: 1.0

Last updated: 2026-07-25

Planning baseline: `b676970`

Related documents:

- `docs/LETSORDER_SERVICE_DESIGN.md`
- `docs/CODE_REVIEW.md`
- `README.md`

## 1. Objective

This plan advances the current working MVP into a stable and maintainable
small-scale home service. It does not repeat completed code-review fixes. It
starts from the current implementation and focuses on release gates,
operations, permission consistency, security, and future scaling boundaries.

Goals:

- Preserve the stable menu collaboration flow without broad rewrites.
- Keep security, authorization, and data isolation under continuous testing.
- Establish reproducible build, deployment, backup, and recovery procedures.
- Define the limits of the single-machine MVP and prepare for storage or
  multi-instance evolution.

## 2. Current Baseline

### 2.1 Implemented Capabilities

| Workstream | Current status |
| --- | --- |
| Accounts and sessions | Registration, login, logout, 48-hour sessions, Argon2, and legacy upgrade |
| Login protection | Process-local five-failure/60-second username throttling |
| Gatherings | Create, join, lock, reopen, archive, and automatic expiry |
| Host ownership | One-time claim token, atomic binding, and backend Host authorization |
| Menu collaboration | Create, edit, cancel, status, Chef, links, and revision conflicts |
| Ratings and recommendations | Post-lock ratings and access-scoped historical recommendations |
| Photos | Post-lock upload, decoding checks, protected reads, and Admin maintenance |
| Realtime | One-time tickets, event isolation, reconnect, and lag recovery |
| Consistency | Transactions, constraints, and retry for concurrent joins/usernames |
| Deployment boundaries | Production admin checks, explicit CORS, protected uploads |
| Automated tests | 15 backend integration tests and 4 Playwright tests |

### 2.2 Current Technical Debt

- The main body of `CODE_REVIEW.md` still describes an older revision, although
  later sections record completed work. It should be archived or rewritten as a
  current snapshot.
- Parts of README do not clearly distinguish Admin frontend capabilities from
  Host capabilities.
- The main Host Dashboard management controls are still visible only to Admin,
  while the backend authorizes claimed Hosts.
- Login throttling, broadcast, and cleanup jobs are single-process facilities.
- Session tokens are stored in plaintext in the database and in browser
  `localStorage`.
- `/health` reports liveness but not database or filesystem readiness.
- SQLite and upload storage do not yet have a formal backup, recovery, and
  upgrade runbook.
- The repository has no CI configuration; `scripts/check.sh` depends on
  developers running it manually.

## 3. Priority Definitions

| Priority | Definition |
| --- | --- |
| P0 | Release blocker or severe data/authorization defect |
| P1 | Security, reliability, or core experience required this release |
| P2 | Operational, quality, and maintainability improvement for the next stage |
| P3 | Architecture evolution triggered by product or capacity growth |

## 4. Milestones

### Milestone 0 - Documentation and Product Decisions

Priority: P1

Estimate: 1-2 person-days

Goal: Align code, product authorization, and project documentation.

Tasks:

1. Review roles, the gathering state machine, APIs, and deployment assumptions in
   the service design.
2. Decide whether a claimed Host should manage deadlines, locking, and archiving
   directly in the frontend.
3. Confirm whether all participants may upload photos and whether only Admin may
   rename or delete them.
4. Archive the historical review body and generate a concise open-risk snapshot
   for the current HEAD.
5. Update README implementation status, environment variables, and Host claim
   flow.

Deliverables:

- Approved service design.
- Recorded permission decisions.
- README and review status aligned with current code.

Acceptance criteria:

- Every documented API, role, and state maps to current code or tests.
- Completed review findings are not presented as open work.
- Host frontend permissions have an explicit product decision.

### Milestone 1 - Release Quality Gates

Priority: P1

Estimate: 2-4 person-days

Goal: Automatically validate formatting, compilation, static checks, backend
integration tests, and browser workflows for every change.

Tasks:

1. Add CI equivalent to `scripts/check.sh`.
2. Pin Rust, Node, and browser versions and configure dependency caches.
3. Give Playwright isolated ports, a temporary database, and a temporary resource
   directory.
4. Preserve failure logs, Playwright traces, and screenshots as artifacts.
5. Require passing checks before merging to the main branch.
6. Add a migration test that starts from an empty database.

Acceptance criteria:

- A clean environment can run all checks with one command.
- CI does not depend on an existing database, local ports, or upload files.
- Two consecutive runs produce consistent results.
- Failed required checks prevent merging.

### Milestone 2 - Host and Permission Experience

Priority: P1

Estimate: 2-4 person-days

Dependency: Host product decision from Milestone 0.

Goal: Align frontend capabilities with backend authorization and avoid showing
participants controls they cannot use.

Tasks:

1. Derive `canManageGathering` from the authenticated participant role, never a
   display name.
2. If approved, expose deadline, lock, and archive controls to claimed Hosts.
3. Provide role-appropriate loading, access-denied, conflict, and success states.
4. Remove the URL fragment and temporary local claim token after successful
   claiming.
5. Add Admin, Host, and Participant Playwright workflows.
6. Align navigation across `/host/:inviteCode` and `/review/:inviteCode`.

Acceptance criteria:

- Every backend-authorized management role can complete its intended frontend
  workflow.
- Participants do not see management actions and direct API calls still return
  `403`.
- Claim tokens do not appear in query parameters, logs, or the browser URL after
  consumption.

### Milestone 3 - Data Protection and Recoverable Deployment

Priority: P1

Estimate: 3-5 person-days

Goal: Establish persistent storage, backup, recovery, and upgrade procedures for
a single-machine production deployment.

Tasks:

1. Define persistent locations for SQLite, uploads, and configuration secrets.
2. Add a consistent backup process covering both SQLite and uploads.
3. Add a restore process and recovery runbook.
4. Define pre-migration backup, migration failure, and rollback policies.
5. Validate free disk space, database permissions, and resource-directory
   writability during startup/readiness.
6. Provide reverse proxy, TLS, WebSocket, and static frontend examples.
7. Prevent production use of scripts that clear development data.

Acceptance criteria:

- A new machine can restore accounts, gatherings, menus, ratings, and photos from
  backup.
- Every restored photo database row resolves to a file.
- Production startup fails clearly when persistent storage or required security
  configuration is missing.

### Milestone 4 - Security Hardening, Phase Two

Priority: P2

Estimate: 4-7 person-days

Goal: Reduce session exposure and public-network abuse risk.

Tasks:

1. Evaluate storing only session token digests in the database.
2. Evaluate migration from `localStorage` Bearer tokens to Secure, HttpOnly,
   SameSite cookies.
3. If cookies are adopted, add CSRF protection and update WebSocket ticket
   acquisition.
4. Add CSP, `X-Content-Type-Options`, Referrer Policy, and related headers.
5. Extend login throttling to username plus client IP with trusted-proxy rules.
6. Define common length and format limits for captions, menu text, and reference
   URLs.
7. Add dependency vulnerability and secret scanning.

Acceptance criteria:

- A database leak cannot directly expose usable session tokens.
- Browser-side scripts cannot read an HttpOnly session, or the retained Bearer
  design has a documented risk acceptance.
- Authentication, CSRF, CORS, WebSocket, and upload boundaries have regression
  tests.

### Milestone 5 - Observability and Operations

Priority: P2

Estimate: 3-5 person-days

Goal: Distinguish process liveness, dependency readiness, and business
failures, and make background jobs diagnosable.

Tasks:

1. Retain `/health` for liveness and add database/filesystem readiness.
2. Add request IDs and structured technical logs.
3. Add request latency/status, login throttle, socket connection, auto-lock, and
   upload failure metrics.
4. Record last success, processed count, and failures for background work.
5. Add graceful shutdown for HTTP traffic and background jobs.
6. Define log retention, sensitive-field redaction, and basic alert thresholds.

Acceptance criteria:

- Database failure makes readiness fail with clear liveness semantics.
- Logs never include passwords, session tokens, Host claim tokens, or WebSocket
  tickets.
- Operators can determine whether auto-locking and ticket cleanup are healthy.

### Milestone 6 - Product Quality and Accessibility

Priority: P2

Estimate: 4-7 person-days

Goal: Complete critical error handling, mobile behavior, and accessibility.

Tasks:

1. Map `401`, `403`, `404`, `409`, and `429` to useful user-facing messages.
2. Show an actionable wait state for login throttling.
3. Audit every mutation for pending, success, conflict, and failure states.
4. Improve keyboard navigation, focus management, form error association, and
   color contrast.
5. Verify menu editing, Host, Review, and Settings on phone, tablet, and desktop.
6. Remove prototype-only `InviteLandingPage` and mock-data paths, or document
   their supported purpose.
7. Add account switching, session expiry, network interruption, and WebSocket
   reconnect E2E scenarios.

Acceptance criteria:

- Core workflows can be completed without a mouse.
- Access failures and load failures never render as valid empty data.
- Key mobile viewports have no overlap, horizontal overflow, or unreachable
  controls.

### Milestone 7 - Storage and Scale Evolution

Priority: P3

Initial estimate: 8-15 person-days when triggered

Triggers:

- Multi-instance deployment is required.
- SQLite locking or disk capacity becomes a measured bottleneck.
- Uploads need a CDN, thumbnails, or shared cross-machine storage.
- Process-local broadcast or throttling cannot meet availability goals.

Candidate work:

1. Introduce a photo storage interface for S3, R2, or MinIO.
2. Use random object keys, server-controlled content types, and short-lived
   authorized downloads.
3. Migrate SQLite to PostgreSQL and verify time, partial-index, and concurrency
   semantics.
4. Move rate limiting to Redis or the reverse proxy.
5. Move realtime events to Redis Pub/Sub, NATS, or another shared system.
6. Add leader election or an external job runner for scheduled tasks.
7. Provide dual-write, validation, and rollback procedures for data migration.

Acceptance criteria:

- Authorization, ticket use, throttling, and realtime behavior remain consistent
  across instances.
- Files and the database can scale independently while retaining traceable
  references.
- Entity counts, referential integrity, and key business results match before and
  after migration.

## 5. Recommended Order

| Order | Work | Priority | Dependency |
| --- | --- | --- | --- |
| 1 | Milestone 0: Documentation and decisions | P1 | None |
| 2 | Milestone 1: CI and release gates | P1 | None; may run in parallel |
| 3 | Milestone 2: Host permission experience | P1 | Milestone 0 |
| 4 | Milestone 3: Backup, recovery, deployment | P1 | Milestone 1 |
| 5 | Milestone 4: Session and security hardening | P2 | Milestone 1 |
| 6 | Milestone 5: Observability and operations | P2 | Milestone 3 |
| 7 | Milestone 6: Product quality and accessibility | P2 | Milestone 2 |
| 8 | Milestone 7: Storage and scale | P3 | Trigger conditions |

Milestones 0 and 1 can proceed in parallel. Milestone 2 implementation should
wait for the Host product decision, although its tests can be designed earlier.

## 6. Suggested Releases

### Release 0.2 - Stable Single Host

Includes Milestones 0-3.

Release outcomes:

- The single-machine architecture has automated quality gates.
- Admin, Host, and Participant product permissions are consistent.
- Deployment, backup, and recovery are reproducible.

### Release 0.3 - Hardened Home Service

Includes Milestones 4-6.

Release outcomes:

- Browser and session security boundaries are stronger.
- The service has basic readiness, metrics, and alerting support.
- Core workflows have mobile, error-state, and accessibility coverage.

### Release 1.0 - Supported Deployment

Entry criteria:

- Release 0.2 and 0.3 acceptance criteria are complete.
- At least one production-style backup restoration has succeeded.
- No P0 or P1 defect remains open for the release.
- The team has explicitly chosen to retain the single-machine architecture or
  completed the required parts of Milestone 7.

## 7. Test Plan

### 7.1 Every Commit

- `cargo fmt --all --check`
- `cargo check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test`
- `npm run lint`
- `npm run build`

### 7.2 Main-Branch Merge

- Run the complete `scripts/check.sh`, including Playwright.
- Apply all migrations to an empty database.
- Run photo upload and access tests with an isolated resource directory.
- For authentication or authorization changes, test Admin, Host, Participant,
  and anonymous access.

### 7.3 Release Candidate

- Clean installation.
- Upgrade from the previous release database.
- Backup and restore exercise.
- Production configuration failure cases.
- Mobile and desktop Chrome critical flows.
- Session expiry, account switching, network interruption, and socket reconnect.
- Upload boundaries, unwritable disk, and database busy behavior.

## 8. Definition of Done

Work is complete only when:

1. Behavior and authorization are defined in design or feature documentation.
2. Implementation follows the existing Route, Service, and Model boundaries.
3. Database changes use a new ordered migration and do not modify released
   migrations.
4. Success, validation, access denial, and conflict paths have automated tests.
5. Cross-account or cross-gathering features include isolation tests.
6. Concurrent writes have database constraints and concurrency tests.
7. Frontend work includes loading, empty, error, pending, and success states.
8. Logs do not expose passwords, sessions, claim tokens, or socket tickets.
9. `scripts/check.sh` passes.
10. README, service design, development plan, or runbooks are updated as needed.

## 9. Risks and Mitigations

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Unclear Host product rights | Frontend/backend inconsistency | Record the decision in Milestone 0 |
| SQLite write locking | Intermittent write failures | Busy timeout, bounded retry, short transactions |
| Filesystem/database drift | Missing or orphaned photos | Compensation, periodic validation, unified backups |
| Process-local throttling | Protection resets or differs by instance | Keep single-node boundary or move to shared limiter |
| `localStorage` session exposure | XSS can steal a session | CSP and dependency control; evaluate HttpOnly cookies |
| Duplicate scheduled work | Multi-instance jobs run repeatedly | Add leader election or external job runner first |
| Flaky E2E tests | CI loses credibility | Isolated ports/data, pinned dependencies, saved traces |
| Documentation drift | Development and acceptance criteria diverge | Require documentation updates in Definition of Done |

## 10. Plan Maintenance

- Assign an owner, scope, and acceptance criteria when starting each milestone.
- Update the code baseline, test count, and open risks for every release.
- Use `CODE_REVIEW.md` for findings tied to a specific revision, not as the
  long-term design source.
- Use this plan for delivery order and the service design for current behavior.
- Record significant architecture changes in an ADR or in the service design
  with a decision date and migration impact.
