# LetsOrder Code Review

Review date: 2026-07-18

Reviewed revision: `40f0010`

Scope: all backend, frontend, migrations, scripts, API tests, and Playwright
tests currently present in the repository.

## Current Findings

### P0 - A user can claim the Host role by registering with the same display name

Creating a gathering inserts a Host participant without a `user_id`:

- `backend/src/services/gathering_service/gatherings.rs:62`

When a user registers or joins, `ensure_participant_for_user` searches for an
unbound participant with the same editable display name and assigns that
participant to the user:

- `backend/src/services/auth_service.rs:336`

The claimed participant retains its `host` role. Management authorization now
correctly checks immutable user membership, but that check consequently trusts
the role obtained through the unsafe claim:

- `backend/src/services/gathering_service/common.rs:121`

Any user who knows the invitation and Host display name can therefore register
with that name and then lock, reopen, or archive the gathering.

Bind the Host participant to a real user when the gathering is created, or
require a one-time Host claim token. Display-name matching must not transfer an
existing role. Add a regression test that registers with `host_name` and
expects participant-level permissions only.

### P1 - Password handling is still unsuitable for a network deployment

If `LETSORDER_ADMIN_PASSWORD` is missing, the server falls back to the publicly
known `Admin_1234` password:

- `backend/src/services/auth_service.rs:647`

Regular passwords use a single salted SHA-256 operation, which is fast enough
for efficient offline guessing:

- `backend/src/services/auth_service.rs:594`
- `backend/src/services/auth_service.rs:613`

The login endpoint has no attempt throttling or temporary lockout. Password
updates also leave all existing sessions valid for up to 48 hours.

Require the administrator password at startup, use Argon2id or bcrypt with
per-password salts, enforce a password policy, rate-limit login attempts, and
revoke the user's existing sessions after a password reset.

### P1 - Chef recommendations expose history across unrelated gatherings

The recommendation route allows any authenticated user to query any Chef name:

- `backend/src/routes/chefs.rs:27`

The query searches all menu items in the database and does not scope results to
gatherings visible to the caller:

- `backend/src/services/gathering_service/recommendations.rs:16`

Results contain dish names, notes, and reference URLs. A user can query common
display names and retrieve history from unrelated gatherings.

Use an immutable Chef user/participant identifier and pass the authenticated
user into the service. Restrict candidate gatherings to those shared by the
caller, unless the caller is an administrator.

### P1 - Switching accounts does not clear user-scoped query data

Login, registration, account updates, and logout change local authentication
state but never clear the shared React Query client:

- `frontend/src/App.tsx:49`
- `frontend/src/utils/user.ts:35`
- `frontend/src/main.tsx:8`

Permission-sensitive query keys such as `['gatherings']` and
`['gathering', inviteCode]` do not include the current user ID. On a shared
browser, a newly logged-in user can receive the previous user's cached data
while refetching. If the new fetch fails, stale data may remain available to
components that continue reading `query.data`.

Clear or remove permission-sensitive queries whenever the authenticated user
changes. Including the user ID in user-scoped query keys provides an additional
isolation boundary.

### P2 - Archived gatherings can be changed into active or locked gatherings

The deadline update always assigns either `active` or `locked`, regardless of
the current status:

- `backend/src/services/gathering_service/gatherings.rs:278`

The lock operation also overwrites the current status without restricting it to
an active gathering:

- `backend/src/services/gathering_service/locking.rs:22`

An administrator or claimed Host can therefore use a known gathering ID to
accidentally resurrect an archived gathering or convert it to `locked`.

Define allowed state transitions explicitly. Add the expected current status
to mutation SQL `WHERE` clauses and return `409 Conflict` for invalid
transitions. Restoration, if required, should be a separate operation.

### P2 - Participant creation is not concurrency-safe

Participant creation checks for an existing `(gathering_id, user_id)` and then
performs a separate insert:

- `backend/src/services/auth_service.rs:321`
- `backend/src/services/auth_service.rs:364`

The database has an index but no unique constraint on that pair:

- `backend/migrations/0004_auth.sql:22`

Concurrent join or registration requests can create duplicate participants,
duplicate join logs, and incorrect participant counts. The unbound
display-name claim is also a check-then-update operation and can be raced by
multiple users.

Add a unique constraint on `(gathering_id, user_id)` for non-null users and use
an atomic upsert or transaction. Remove display-name role claims as described
in the P0 finding.

### P2 - Multi-step writes do not use transactions

The following operations perform related writes independently:

- Gathering, Host participant, and creation activity:
  `backend/src/services/gathering_service/gatherings.rs:43`
- User, participant, and session creation:
  `backend/src/services/auth_service.rs:85`
- User and participant account updates:
  `backend/src/services/auth_service.rs:170`
- Menu item update, audit logs, and participant activity:
  `backend/src/services/gathering_service/menu_items.rs:165`
- Uploaded file, photo row, and upload activity:
  `backend/src/services/gathering_service/photos.rs:112`

A later failure can leave partial records, missing audit entries, or orphaned
files. Use SQL transactions for related database changes. For uploads, remove
the new file when the database transaction fails.

### P2 - Review and Host pages do not render load failures correctly

`ReviewPage` does not branch on loading or error states. When the gathering
request fails, it renders an apparently valid empty review with photo upload
controls:

- `frontend/src/pages/ReviewPage.tsx:25`
- `frontend/src/pages/ReviewPage.tsx:152`

`HostDashboardPage` similarly renders the normal dashboard and labels a missing
gathering as `Active`:

- `frontend/src/pages/HostDashboardPage.tsx:202`
- `frontend/src/pages/HostDashboardPage.tsx:380`

Add explicit loading, forbidden, not-found, and service-error views before
rendering either operational page.

### P2 - Photo upload policy and image validation remain incomplete

The upload service verifies participant membership but does not require the
gathering to be locked:

- `backend/src/services/gathering_service/photos.rs:40`

Participants can upload photos during active menu editing by calling the API
directly, even though the documented workflow presents photos as a post-event
feature.

Image validation only checks a few signature bytes. The API test's accepted
"PNG" contains a PNG header followed by arbitrary text and cannot be decoded as
an image:

- `backend/src/services/gathering_service/photos.rs:160`
- `backend/tests/api.rs:656`

Enforce the intended gathering state in the backend. Fully decode accepted
images and enforce pixel dimensions in addition to the existing 8 MiB limit.

### P2 - Realtime authentication and event delivery are not isolated

The frontend places the bearer token in the WebSocket URL:

- `frontend/src/hooks/useRealtimeRefresh.ts:16`

The backend authenticates that query token and subscribes every authenticated
socket to the same broadcast receiver:

- `backend/src/routes/realtime.rs:21`
- `backend/src/routes/realtime.rs:38`

Tokens may appear in request, tracing, or proxy logs. Every logged-in user also
receives gathering IDs for unrelated refresh events.

Use an authentication mechanism that does not expose the bearer token in the
URL, and filter subscriptions or outbound events by the gatherings visible to
the authenticated user.

### P3 - The menu list performs redundant N+1 participant requests

The backend already returns only the gatherings available to a regular user.
The frontend then requests the participant list for every returned gathering
and filters again by display name:

- `frontend/src/pages/MenusPage.tsx:28`
- `frontend/src/pages/MenusPage.tsx:34`

This adds one request per gathering and can hide a valid menu when one
participant request fails. Use the backend response directly and remove the
display-name filtering layer.

### P3 - Route-level role and not-found handling is incomplete

The root creation route is protected only by authentication, so a non-admin can
open the creation form even though the backend will reject submission:

- `frontend/src/App.tsx:149`

There is also no catch-all route, leaving unknown paths blank. Add an admin role
guard for the creation page and a not-found route.

## Fixed Since The Previous Review

The current version has addressed these earlier findings:

- Editable `suite-admin` display names no longer grant administrator access.
- Menu revision updates require `expected_revision` and update atomically.
- Runtime mock menu fallbacks were removed from production flows.
- Administrators can open menu workspaces in an explicit read-only mode.
- Cross-Chef recommendation selection no longer fails silently with `403`.
- Uploads now have an 8 MiB limit and basic signature/extension checks.
- `scripts/check.sh` includes formatting, Clippy, tests, lint, build, and E2E.

The Host identity finding above remains open because the participant claim path
still transfers an unbound `host` role based on display-name equality.

## Verification Results

The following checks were run against revision `40f0010`:

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo check` | Passed |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Passed |
| `cargo test` | Passed: 3 API tests |
| `npm run lint` | Passed |

## Resolution Notes

The following findings were addressed in the current working revision:

- Host display names no longer claim an unbound `host` participant. New joins
  always create or reuse a participant bound to the authenticated user, and a
  partial unique index prevents duplicate bound participants.
- Recommendations are scoped to gatherings visible to the authenticated user;
  administrators retain global visibility.
- Archived gatherings cannot be reopened or locked, and locking an already
  locked or archived gathering returns `409 Conflict`.
- Account password changes use Argon2id for newly stored passwords and revoke
  the target user's other sessions. Legacy hashes remain readable for the
  existing local database.
- Photos require a locked gathering and are decoded with dimension limits before
  they are written to disk. The API tests now use a real PNG fixture.
- WebSocket connections use one-minute, one-time tickets instead of bearer
  tokens in the URL, and non-admin events are filtered by participant access.
- Menu list queries use the authenticated backend response directly. React
  Query data is cleared when the authenticated user changes, the create route
  is admin-only, and unknown routes render an explicit not-found page.
- Review and On Track pages now render explicit loading and unavailable states.

Remaining deployment hardening is intentionally tracked separately: require an
administrator password through environment configuration in production, add
login throttling, wrap multi-record writes and upload cleanup in SQL
transactions, and add concurrent stale-revision/realtime isolation tests.
| `npm run build` | Passed |
| `npm run e2e` | Passed: 2 Playwright tests |

All existing automated checks pass. They do not currently cover the P0 Host
claim path or the isolation and concurrency cases listed below.

## Missing Test Coverage

- Registering or joining with the Host display name must not grant `host`.
- Two simultaneous joins by the same user must produce one participant.
- Two simultaneous claims of an unbound participant must not overwrite
  ownership.
- Archived gatherings must reject deadline and lock mutations.
- Account switching must not expose cached data from the previous account.
- Recommendation results must not cross gathering access boundaries.
- Review and Host pages must render 403, 404, and network failures.
- Uploads must reject signature-only files that cannot be decoded.
- Active gatherings must enforce the intended photo upload policy.
- Realtime events must be isolated between unrelated users and gatherings.
- Transaction rollback must be verified for later-step failures.

## Recommended Fix Order

1. Remove display-name Host claims and bind Host ownership securely.
2. Enforce Argon2id/bcrypt, mandatory administrator configuration, login
   throttling, and session revocation on password changes.
3. Scope recommendations to gatherings visible to the authenticated user.
4. Clear or user-scope frontend query caches when authentication changes.
5. Enforce gathering state transitions and participant uniqueness.
6. Add transactions and upload cleanup.
7. Add explicit Review/Host error states.
8. Harden image decoding, upload-state rules, and realtime isolation.
9. Remove redundant menu-list requests and add route guards.
10. Add the missing regression and concurrency tests.
