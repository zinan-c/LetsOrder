# LetsOrder Code Review

Review date: 2026-07-21

Reviewed revision: `a8dd53e`

Scope: backend, frontend, migrations, scripts, API tests, and Playwright tests
currently present in the repository.

## Current Findings

### P0 - Non-admin Host management is no longer usable

Gathering creation still accepts a `host_name` and returns a Host participant and
access token, but the participant is now stored with the `participant` role:

- `backend/src/services/gathering_service/gatherings.rs:62`

Registration and joining always create another participant with the
`participant` role:

- `backend/src/services/auth_service.rs:377`
- `backend/src/services/auth_service.rs:402`

Management authorization, however, only permits administrators or a participant
bound to the authenticated user with the `host` role:

- `backend/src/services/gathering_service/common.rs:121`

No current code path creates such a bound Host. The browser stores the returned
participant ID and access token, but backend management authorization does not
consume either value. Consequently, a non-admin user can open the Host page but
cannot change the deadline, lock, or archive the gathering.

Restore Host ownership without reintroducing display-name claims. Prefer binding
the Host to an existing user ID at creation, or issue a single-use Host claim
token that atomically binds the unbound Host participant. The resulting
participant must have `role = 'host'` and an immutable `user_id`.

Required regression tests:

- The intended Host can claim and manage the gathering.
- A different user with the same display name cannot claim Host privileges.
- A Host claim token can only be consumed once.
- Ordinary participants receive `403` from Host mutations.

### P1 - Administrator login is not ready for a network deployment

If `LETSORDER_ADMIN_PASSWORD` is absent, authentication falls back to the known
`Admin_1234` password:

- `backend/src/services/auth_service.rs:21`
- `backend/src/services/auth_service.rs:33`
- `backend/src/services/auth_service.rs:707`

The login endpoint also has no attempt throttling or temporary lockout. Argon2 is
used for new passwords, but successful login with a legacy SHA-256 or FNV hash
does not upgrade the stored hash:

- `backend/src/services/auth_service.rs:25`
- `backend/src/services/auth_service.rs:651`

Require the administrator password through environment configuration and fail
startup when it is missing or still set to a known development value. Add
account/IP-based login throttling with bounded lockout. Rehash legacy passwords
with Argon2 after successful verification.

Required regression tests:

- Production startup fails without an administrator password.
- Repeated failed login attempts are throttled.
- A valid login still succeeds after the throttling window.
- A successful legacy-password login replaces the stored hash with Argon2.

### P1 - WebSocket tickets are not consumed atomically

Ticket consumption performs a `SELECT` followed by a separate `DELETE`:

- `backend/src/services/auth_service.rs:161`

Two concurrent WebSocket handshakes can read the same ticket before either
request deletes it, allowing a nominally one-time credential to authenticate
more than once. Expired tickets that are never presented also remain in the
database indefinitely.

Consume tickets in one atomic database operation, such as
`DELETE ... RETURNING`, or use a transaction with the appropriate SQLite write
locking behavior. Reject expired tickets within the same operation and add
periodic expired-ticket cleanup.

Required regression tests:

- Two concurrent consumers of one ticket produce exactly one success.
- An expired ticket is rejected.
- A consumed ticket cannot be reused.
- Expired unconsumed tickets are removed by cleanup.

### P1 - Related writes still lack transactions and file compensation

Several operations perform related writes independently:

- Gathering, initial participant, and activity creation:
  `backend/src/services/gathering_service/gatherings.rs:43`
- User, participant, and session registration:
  `backend/src/services/auth_service.rs:77`
- User, sessions, and participant account updates:
  `backend/src/services/auth_service.rs:180`
- Menu item update, change logs, and participant activity:
  `backend/src/services/gathering_service/menu_items.rs:165`
- Uploaded file, photo row, and upload activity:
  `backend/src/services/gathering_service/photos.rs:128`

A later failure can leave incomplete records, missing audit entries, inconsistent
display names, or orphaned files. Wrap related database changes in SQL
transactions. For uploads, remove the newly written file if the transaction
fails. Define explicit compensation behavior when deleting a database row
succeeds but deleting its file fails.

Required regression tests should inject a failure into each later step and
verify database rollback and upload-file cleanup.

### P2 - Realtime refresh does not recover from failures

The frontend creates one ticket and one WebSocket. Ticket request failures,
connection failures, and later socket closures are ignored, with no reconnect:

- `frontend/src/hooks/useRealtimeRefresh.ts:17`

The backend loop exits on any broadcast receive error. In particular, a
`Lagged` result caused by a burst larger than the broadcast buffer closes the
connection:

- `backend/src/routes/realtime.rs:41`
- `backend/src/main.rs:21`

Add bounded exponential-backoff reconnection that requests a fresh ticket for
each attempt. Handle broadcast lag by continuing after recording the skipped
event count, and reserve connection termination for closed channels or socket
errors.

Required tests should cover initial ticket failure, socket closure, reconnect
with a fresh ticket, broadcast lag, and event isolation between unrelated users.

### P2 - Security and concurrency fixes lack focused regression coverage

The current suite contains three backend API tests and two Playwright tests. It
does not directly cover several recently changed boundaries:

- Host ownership and Host display-name impersonation.
- Concurrent participant joins.
- WebSocket ticket single-use behavior and gathering isolation.
- Archived gathering mutation rejection.
- Recommendation isolation across unrelated gatherings.
- React Query cache isolation after switching accounts.
- Review and Host page behavior for `403`, `404`, and network failures.
- Transaction rollback and upload-file cleanup.

Add focused backend integration tests first, then browser tests for the
user-visible account-switching, Host, and realtime-reconnect flows.

### P3 - Username allocation has a check-then-insert race

Registration searches for an available username and inserts the user in a later
operation:

- `backend/src/services/auth_service.rs:84`
- `backend/src/services/auth_service.rs:594`

Concurrent registrations with the same display name can choose the same
candidate. One request then fails on the unique username constraint instead of
retrying with the next suffix. Treat the database constraint as authoritative
and retry allocation on a username conflict, preferably within the registration
transaction.

### P3 - Deployment boundaries remain permissive

The server applies permissive CORS to every route:

- `backend/src/main.rs:24`

Restrict allowed origins, headers, and methods through deployment configuration.
Also decide whether `/resources/uploads/*` is intentionally public. If photos
are private gathering data, serving the directory directly bypasses gathering
authorization:

- `backend/src/routes/mod.rs:40`

## Recommended Fix Order

1. Restore secure Host ownership and management authorization.
2. Require administrator configuration and add login throttling.
3. Make WebSocket ticket consumption atomic and clean up expired tickets.
4. Add transactions and upload-file compensation.
5. Add focused tests for items 1-4 before further refactoring.
6. Add realtime reconnect and broadcast-lag recovery.
7. Add the remaining isolation, failure-state, and browser regression tests.
8. Make username allocation concurrency-safe.
9. Restrict production CORS and decide upload-resource visibility.

## Resolved Since Revision `40f0010`

The following previous findings are implemented in revision `a8dd53e` and should
remain protected by regression tests:

- Display-name matching no longer transfers an unbound Host role.
- A partial unique index prevents duplicate bound participants for one user and
  gathering.
- Chef recommendations are scoped to gatherings visible to the caller.
- Archived gatherings cannot be reopened or locked.
- New and changed passwords use Argon2, and password changes revoke other
  sessions.
- Photo uploads require a locked gathering and fully decode images with dimension
  limits.
- WebSocket URLs use short-lived tickets instead of bearer session tokens, and
  non-admin events are filtered by gathering membership.
- Account changes clear React Query data, menu-list N+1 requests were removed,
  the creation route is admin-only, and unknown routes render a not-found page.
- Review and Host pages render explicit loading and unavailable states.

## Verification Results

The following checks were run against revision `a8dd53e` on 2026-07-21:

| Check | Result |
| --- | --- |
| `cargo test` | Passed: 3 API tests |
| `npm run lint` | Passed |
| `npm run build` | Passed |

These passing checks do not cover the P0 Host regression or the concurrency,
rollback, and realtime recovery cases listed above.

## Execution Status

### Host ownership — implemented, pending local commit

Host creation now stores a hashed one-time claim token and retains the Host
role. An authenticated user can consume the token through
`POST /api/gatherings/:id/host/claim`; if registration already created a
participant, the claim atomically upgrades that participant and consumes the
unbound Host record without transferring the role by display name. Reuse and
same-name impersonation are rejected. Frontend Host claim URLs use a URL
fragment so the token is not sent as an HTTP request parameter.

### Administrator authentication — implemented, pending local commit

Production startup now requires `LETSORDER_ADMIN_PASSWORD` through
`LETSORDER_ENV=production`, rejects the known development password and short
values, applies username-based temporary login throttling, and upgrades legacy
password hashes to Argon2 after successful login. A focused rate-limit API test
is included; a production startup test and legacy-hash migration assertion are
still candidates for the broader test pass.

### WebSocket ticket consumption — implemented, pending local commit

Ticket consumption now uses SQLite `DELETE ... RETURNING` with an expiry guard,
so only one concurrent consumer can succeed. The ten-minute background job also
removes expired unconsumed tickets, and an integration test verifies concurrent
single-use behavior.

### Related writes and upload compensation — implemented, pending local commit

Gathering creation, participant registration/join, account/member updates,
menu item creation/update, and photo row plus activity-log writes now use SQL
transactions. Photo writes remove the newly created file when the database
transaction or file write fails.

### Realtime recovery — implemented, pending local commit

The frontend now requests a fresh ticket for each connection attempt and uses
bounded exponential backoff after ticket or socket failures. The backend
continues after `broadcast` lag while logging the skipped event count, and only
terminates on a closed channel or socket error.

### Focused regression coverage — implemented, pending local commit

The backend suite now covers Host claim ownership, one-time WebSocket tickets,
login throttling, archived mutation rejection, concurrent participant joins,
and the existing recommendation/photo/menu permission flows. Concurrent join
handling includes SQLite busy timeout and bounded retry behavior.

### Username allocation — implemented, pending local commit

Registration now uses `ON CONFLICT(username) DO NOTHING` inside the registration
transaction and retries username allocation when another request wins the same
candidate. A concurrent registration test verifies both requests succeed with
distinct usernames.
