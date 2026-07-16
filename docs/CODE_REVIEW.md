# LetsOrder Code Review

Review date: 2026-07-16

Scope: all backend, frontend, migration, script, API test, and Playwright test code
currently present in the repository.

## Findings

### P0 - Display names can be used to gain administrative privileges

Account updates allow any non-admin user to set their `display_name` to
`suite-admin`:

- `backend/src/services/auth_service.rs:152`

Management authorization then relies on that editable display name:

- `backend/src/services/gathering_service/common.rs:177`
- `backend/src/services/gathering_service/common.rs:206`
- `backend/src/routes/gatherings.rs:391`

A normal user can change their display name and then lock, reopen, or archive a
gathering, and update or delete photos. The same design allows a user to
impersonate a gathering host by choosing the host's display name.

Authorization should use the authenticated user's immutable ID and role.
Gathering host ownership should be represented by a user ID or participant ID
bound to that user. Display names must never be treated as authorization
credentials.

Resolution comment: management authorization now receives the authenticated
`User` and checks immutable `role`/`user_id` membership instead of editable
display names. Account/member updates and registration reject the reserved
`suite-admin` display name. Photo upload also binds to the authenticated
participant instead of creating participants by display name. A fuller
host-owner model can still be added later if non-admin host management returns.

### P0 - Password storage and the fixed administrator credentials are insecure

The administrator username and password are hardcoded:

- `backend/src/services/auth_service.rs:13`

The fixed password is also documented in the README and displayed in the
frontend settings page. User passwords are hashed with a fixed salt and a
64-bit FNV-style hash:

- `backend/src/services/auth_service.rs:593`

Generated passwords consist of the display name plus only three digits:

- `backend/src/services/auth_service.rs:77`

This is unsuitable for any network-accessible deployment. Use Argon2id or
bcrypt with per-password salts, configure initial administrator credentials
through environment variables or a setup flow, and add login rate limiting.

Resolution comment: newly stored user passwords now use a per-password salted
hash format and existing legacy hashes remain readable for compatibility.
Generated first-use passwords are random temporary passwords instead of
`display_name + 3 digits`. Fixed-password UI/README text was removed and the
system admin password can be configured through `LETSORDER_ADMIN_PASSWORD`.
Remaining hardening: replace the interim SHA-256 format with Argon2id/bcrypt
and add login rate limiting before any real deployment.

### P1 - Optimistic concurrency control is not atomic

Menu item updates read the current revision and compare it in application code:

- `backend/src/services/gathering_service/menu_items.rs:107`

The subsequent SQL update does not include the expected revision:

- `backend/src/services/gathering_service/menu_items.rs:156`

Two requests that read the same revision can therefore both pass the check and
both update the record. The second write silently overwrites the first.
`expected_revision` is also optional, allowing callers to bypass conflict
detection.

The update should use a statement equivalent to:

```sql
UPDATE menu_items
SET ..., revision = revision + 1
WHERE id = ? AND revision = ?
```

Return `409 Conflict` when `rows_affected()` is zero, and require
`expected_revision` for update requests.

Resolution comment: menu item updates now require `expected_revision` and the
SQL `UPDATE` includes `WHERE id = ? AND revision = ?`. A zero-row update
returns `409 Conflict` with the latest menu item and submitted payload.

### P1 - Mock data is included in production application flows

The gathering page starts with mock menu items and restores them when API
loading fails:

- `frontend/src/pages/GatheringPage.tsx:107`
- `frontend/src/pages/GatheringPage.tsx:232`

When no gathering or participant is available, menu edits are saved only in
local React state:

- `frontend/src/pages/GatheringPage.tsx:440`

The administrator menu list always appends a fake `mockdata` gathering:

- `frontend/src/pages/MenusPage.tsx:20`
- `frontend/src/pages/MenusPage.tsx:70`

Authentication failures, missing invitations, and backend outages can therefore
look like a working application. Remove runtime mock fallbacks and render
explicit loading, not-found, forbidden, and service-error states.

Resolution comment: the menu workspace no longer initializes/restores mock menu
items on API failure, local-only menu edits were removed, and the menu list no
longer appends the fake `mockdata` gathering. The UI now shows explicit load
errors/empty states. Remaining polish: split forbidden/not-found/service-error
copy into more specific user-facing states.

### P1 - Administrators cannot use the menu workspace

The backend intentionally returns no participant when an administrator joins:

- `backend/src/routes/gatherings.rs:149`

The frontend interprets a missing participant as requiring the join modal and
hides the menu:

- `frontend/src/pages/GatheringPage.tsx:529`
- `frontend/src/pages/GatheringPage.tsx:591`

Submitting that modal then throws because administrators cannot become
participants:

- `frontend/src/pages/GatheringPage.tsx:321`

This conflicts with administrator menu links in the host dashboard and menu
list. Either provide administrators with a read/manage mode that does not need
a participant, or create a clearly modeled administrator participant when menu
editing is intended.

Resolution comment: administrators can now open the menu workspace without
being forced through the join modal. Admins remain read-only for dish editing
because menu item mutations are still participant-authored. Management actions
remain available through host/on-track flows.

### P2 - Dish recommendation permissions conflict with the editor UI

The editor lets a participant select any gathering participant as the Chef.
The recommendation endpoint only permits a user to query their own display
name, unless the user is an administrator:

- `backend/src/routes/chefs.rs:33`
- `frontend/src/pages/GatheringPage.tsx:285`

Selecting another Chef silently produces an empty recommendation list. Align
the API authorization rule with the product behavior, preferably by querying
recommendations through an immutable participant or user ID.

Resolution comment: the recommendation endpoint now allows any authenticated
user to query Chef recommendations, matching the editor's ability to choose
another Chef. Remaining hardening: replace the display-name route with an
immutable user/participant identifier once the product has a stable Chef
identity selector.

### P2 - Multi-step mutations are not transactional

Several operations perform related writes independently:

- Gathering, host participant, and creation activity:
  `backend/src/services/gathering_service/gatherings.rs:42`
- User registration, participant creation, and session creation:
  `backend/src/services/auth_service.rs:82`
- User and participant display-name updates:
  `backend/src/services/auth_service.rs:166`
- Menu item changes, activity logs, and participant activity:
  `backend/src/services/gathering_service/menu_items.rs:156`
- Uploaded file, photo row, and activity log:
  `backend/src/services/gathering_service/photos.rs:102`

A failure in a later step can leave partial records, missing audit logs, or
orphaned files. Use SQL transactions for database mutations. For uploads,
remove the newly written file when the database transaction fails.

Resolution comment: not fully addressed in this pass. The riskiest upload path
now validates image content before writing and binds uploads to an authenticated
participant, but the broader transaction refactor remains open.

### P2 - Photo uploads do not validate actual image content

The upload implementation trusts the file extension and reads the complete
field into memory:

- `backend/src/services/gathering_service/photos.rs:69`
- `backend/src/services/gathering_service/photos.rs:85`

The API test successfully uploads `fake-image-bytes` as a PNG:

- `backend/tests/api.rs:604`

Validate content type and image signatures, decode images before accepting
them, enforce explicit byte and dimension limits, and reject unsupported
formats.

Resolution comment: uploads now enforce an 8 MiB byte limit, validate common
image signatures, and reject extension/content mismatches. API coverage now
asserts fake PNG bytes are rejected. Remaining hardening: fully decode images
and enforce dimensions before deployment.

### P2 - WebSocket bearer tokens are placed in URLs

The frontend sends the session token as a WebSocket query parameter:

- `frontend/src/hooks/useRealtimeRefresh.ts:16`

The backend accepts the token from that query parameter:

- `backend/src/routes/realtime.rs:21`

URL query values can appear in request, proxy, and tracing logs. In addition,
every authenticated socket subscribes to the same broadcast channel and
receives gathering IDs for gatherings the user may not be able to access.

Prefer an authenticated cookie or a deliberately selected WebSocket
authentication mechanism, and filter subscriptions/events by the gatherings
available to the authenticated user.

Resolution comment: not addressed in this pass. This remains a backlog item
because it needs a slightly larger realtime subscription redesign.

### P3 - The project quality gate does not include all available checks

`scripts/check.sh` runs Rust formatting, checking, tests, and the frontend
production build, but does not run ESLint, Clippy, or Playwright:

- `scripts/check.sh:6`

As a result, it can print `All checks passed` while linting or stricter Rust
checks fail. Add the missing checks to the script or define separate CI jobs
whose required status is visible.

Resolution comment: `scripts/check.sh` now includes `cargo clippy`, `npm run
lint`, and Playwright e2e in addition to the previous format/build/test checks.

## Verification Results

The following checks were run during this review:

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo check` | Passed |
| `cargo test` | Passed: 3 API tests |
| `npm run build` | Passed |
| `npm run e2e` | Passed: 2 Playwright tests |
| `npm run lint` | Passed |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Passed |

Follow-up note: `./scripts/check.sh` includes e2e now. In the Codex sandbox it
may fail while binding local ports; rerunning `npm run e2e` with local-port
permission passed.

## Missing Test Coverage

The existing tests cover sequential stale-revision detection and now enforce
atomic SQL revision checks, but do not yet include a true concurrent two-writer
race. They also do not cover:

- Administrator behavior in the menu workspace.
- API failure and forbidden states without mock-data fallback.
- Upload dimension limits.
- Partial-write rollback when a later database or filesystem operation fails.
- Realtime event isolation between unrelated users and gatherings.

New coverage added after this review:

- Reserved `display_name = "suite-admin"` account update rejection.
- Invalid image bytes rejection for uploaded photos.
- Dish recommendation sorting and authenticated cross-Chef recommendation reads.

## Recommended Fix Order

1. Replace the interim salted SHA-256 password format with Argon2id/bcrypt and add login rate limiting.
2. Add database transactions and upload cleanup for multi-step mutations.
3. Harden WebSocket authentication and event isolation.
4. Add a durable host ownership model if non-admin host management is required.
5. Replace display-name Chef recommendation routes with immutable IDs.
6. Add image decoding and dimension limits.
7. Add concurrent stale-revision and realtime isolation tests.
