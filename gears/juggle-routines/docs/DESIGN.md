---
description: "How juggle-routines and juggle-routines-sdk are built: crew state as one document, frozen days, photos, reminders."
---

# Technical Design — juggle-routines

## 1. Architecture Overview

### 1.1 Architectural Vision

| Crate | Kind | Job |
|---|---|---|
| `juggle-routines` | gear (`db`, `rest`, `stateful`) | REST API, crew state and photos in PostgreSQL, the reminder worker. Registers `RoutinesClientV1`. |
| `juggle-routines-sdk` | library | `RoutinesClientV1`, `CrewPerson`, `Notification`, `NotificationKind`, `DeliveryOutcome`, `RoutinesError`. Plain types only. |

All of a crew's routines data is one JSON document in `jr_crew_state` with a revision. Photos sit in `jr_avatars`. The gear declares `deps = [juggle_crews, authz_resolver]`.

### 1.2 Architecture Drivers

#### Functional Drivers

| Requirement | Design response |
|---|---|
| `cpt-cj-routines-fr-people` | `State::sync_accounts` adds a person per member account on every load. `edit_person` keeps the stored `account`. |
| `cpt-cj-routines-fr-schedule` | Edits append a `Version` with `effective_from`; `routines_on(day)` picks the version in effect. |
| `cpt-cj-routines-fr-frozen-days` | `State::ensure_day` stores a day's occurrences; a past day that exists is never rebuilt. |
| `cpt-cj-routines-fr-check-off`, `cpt-cj-routines-fr-confirm-undo` | `State::mark`. |
| `cpt-cj-routines-fr-reminders` | `domain/reminders.rs`, run by the gear's `serve` loop. |
| `cpt-cj-routines-fr-crew-push` | `Application` implements `RoutinesClientV1`. |

#### NFR Allocation

| NFR | Allocated to | Design response |
|---|---|---|
| Crew isolation | `Service::scope`, `PgStore` | `CrewPolicy::scope` asks `authz-resolver` on every operation. The repo refuses writes unless the scope admits exactly one crew (`crew_of`). |
| Idempotent commands | `State::record` / `replay` | Journal `operations: operation_id → (actor, command, response)` inside the same document. |
| No lost updates | `Store::compare_and_swap`, item revisions | Row update `WHERE revision = r`; `Service::mutate` reloads and retries up to 8 times, then `409`. Items carry their own revisions. |
| Bounded uploads | routes, `PngNormalizer` | `DefaultBodyLimit` 768 KiB; PNG decode limits. |

#### Key ADRs

- [0001: canonical Gears layout, plugins, typed config](../../../docs/ADR/0001-cpt-cj-adr-canonical-gears.md)

### 1.3 Architecture Layers

| Layer | Holds |
|---|---|
| `api/rest` | DTOs (`dto.rs`, the wire shape the web app has read since v0.1), handlers, routes, problem mapping |
| `domain` | `State` and its rules (`mod.rs`), `Service`, `Application`, journal types, reminders, subscription checks, ports `Store`, `AvatarStore`, `PushTransport`, `AvatarNormalizer` |
| `infra` | `PgStore` (entities, migration, `codec.rs` for the stored JSON, `journal.rs` for stored command fingerprints), `Push` (Web Push), `PngNormalizer` |

## 2. Principles & Constraints

### 2.1 Design Principles

- **This gear is a policy enforcement point.** Each operation gets its scope from `authz-resolver`; the crews authz plugin answers `owner_tenant_id = crew`.
- **The crew owns its zone.** `State.zone` is filled from `CrewsClientV1::crew` on every load and never stored.
- **Stored shape is separate from domain types.** `codec.rs` mirrors the JSON written since v0.1, so old rows and journal fingerprints stay valid.
- **The worker is a crew caller too.** It runs under `worker_context(crew)` and goes through the same policy and scope.

### 2.2 Constraints

- PostgreSQL only. The migration name `storage` is pinned by `tests/migration_names.rs`.
- At most 20 people per crew. Routine name ≤ 120 characters, note ≤ 1000, group ≤ 60; time `HH:MM`; days 1 (Monday) to 7.
- Person tones: `blue`, `lilac`, `green`, `peach`. Moods: `happy`, `neutral`, `grumpy`.

## 3. Technical Architecture

### 3.1 Domain Model

**Location**: [`juggle-routines/src/domain/mod.rs`](../juggle-routines/src/domain/mod.rs)

`State` (one per crew):

| Field | Holds |
|---|---|
| `created_date` | first crew date with data |
| `people` | `Person`: id, name, tone, optional account, created date, revision |
| `versions` | every `Version` of every routine, oldest first |
| `days` | date → `Occurrence` list (routine snapshot, status, revision, who and when) |
| `events` | check-off and undo trail |
| `operations` | idempotency journal |
| `preferences`, `appearances` | per-account display zone; per-routine mood |
| `subscriptions`, `deliveries` | push devices; reminder attempts with status `pending`, `sending`, `sent`, `failed`, `cancelled` |

A routine may name one prerequisite. A first step has at most one second step. The pair shares person, time, days, group, dates and `enabled`; editing one moves both. The prerequisite cannot change after the routine exists.

### 3.2 Component Model

| ID | Where | Does |
|---|---|---|
| `cpt-cj-routines-component-rest` | `api/rest` | 16 routes; multipart parsing for photos; maps `Error` to problems. |
| `cpt-cj-routines-component-service` | `domain::Service` | `scope`, `load` (create state on first use, sync accounts), `mutate` (CAS retry loop), `day`, `schedule`. |
| `cpt-cj-routines-component-application` | `domain::application` | Membership check (`actor`), subscriptions, test push, photos, `RoutinesClientV1`. |
| `cpt-cj-routines-component-reminders` | `domain::reminders` | `tick` over all crews; per crew: prepare deliveries, claim per device, send, record outcome. |
| `cpt-cj-routines-component-store` | `infra::storage::PgStore` | `Store` and `AvatarStore` on the secure ORM. |
| `cpt-cj-routines-component-push` | `infra::push::Push` | Web Push with VAPID; 25 s timeout per send in the worker. |

### 3.3 API Contracts

**Technology**: REST. Every route is authenticated and needs a member of the crew in the caller's context. Errors are problems with `error_domain = "juggle-routines.v1"` and, for older clients, `message` and `code` fields.

| Method | Path | Does |
|---|---|---|
| `GET` | `/juggle-routines/v1/me` | account id, crew zone, today, VAPID public key |
| `GET` / `PUT` | `/juggle-routines/v1/me/preferences` | read everyone's display zones / change yours |
| `GET` | `/juggle-routines/v1/day?date=YYYY-MM-DD` | one day, default today; future dates `400` |
| `PUT` | `/juggle-routines/v1/occurrences/{date}/{id}` | check off or undo |
| `GET` / `PUT` | `/juggle-routines/v1/schedule` | all versions and people / add or change a routine |
| `PUT` | `/juggle-routines/v1/schedule/{id}/appearance` | set a routine's mood |
| `GET` / `PUT` | `/juggle-routines/v1/people` | list / add or change a person |
| `GET` | `/juggle-routines/v1/people/{id}/avatar` | PNG, `no-store` |
| `PUT` | `/juggle-routines/v1/people/{id}/avatar` | multipart `operation_id`, `expected_revision`, `file` (`image/png`) |
| `DELETE` | `/juggle-routines/v1/people/{id}/avatar` | JSON `operation_id`, `expected_revision` |
| `POST` | `/juggle-routines/v1/push/subscriptions` | register this browser |
| `DELETE` | `/juggle-routines/v1/push/subscriptions/{id}` | remove one of your devices |
| `POST` | `/juggle-routines/v1/push/test` | test push to one of your devices |

| Error | Status | `code` |
|---|---|---|
| `Forbidden` | 403 | `access_denied` |
| `Invalid` | 400 | `request_failed` |
| `Conflict` | 409 | `revision_conflict` |
| `ConfirmationRequired` | 409 | `confirmation_required` |
| `NotFound` | 404 | `request_failed` |
| `Storage` | 503 | `request_failed` |

**`RoutinesClientV1`** (ClientHub):

| Method | Who may call | Rules |
|---|---|---|
| `people(ctx)` | a crew member or the worker | the crew's people with optional account id |
| `notify_crew(ctx, message, skip_account)` | the worker only | URL must be `/today` or `/shopping` (plus query) to match the kind; title ≤ 200, body ≤ 2000 chars; TTL 1 s to 1 day. Returns how many devices accepted. Removes gone subscriptions. |

### 3.4 Internal Dependencies

| Dependency | Interface | Purpose |
|---|---|---|
| juggle-crews | `CrewsClientV1` | crew zone, members, crew ids for the worker |
| `authz-resolver` | `PolicyEnforcer` via `CrewPolicy` | scope per operation |
| `toolkit-db` | secure ORM | scoped queries, transactions |

### 3.5 External Dependencies

| System | Use |
|---|---|
| PostgreSQL | `jr_crew_state`, `jr_avatars` |
| Browser push services | Subscriptions are accepted only for HTTPS endpoints without port or credentials on the FCM, Mozilla, Apple and Windows push hosts. `p256dh` must be a 65-byte uncompressed key, `auth` 16 bytes. |

### 3.6 Interactions & Sequences

#### A write (check-off, edit, person, preferences, mood)

1. `Service::load`: scope from the policy; crew and members from juggle-crews; read the row; create it if missing; add missing account people (CAS).
2. Apply the change to the in-memory `State`. The rule checks the item revision and replays a known `operation_id`.
3. `compare_and_swap(scope, revision, state)`. On a lost race, reload and retry (up to 8 times, 5 ms × attempt back-off), then `409`.

#### A photo change

One transaction: lock the `jr_crew_state` row (`FOR UPDATE`), decide with `State::apply_avatar_change` (journal, revision), upsert `jr_avatars` (image or NULL), write the state with revision + 1. Uploads are PNG, ≤ 512 KiB, exactly 256 × 256, re-encoded as RGBA without metadata.

#### Reminders (every 30 s)

1. `crew_ids(worker_context(nil))`, then for each crew under `worker_context(crew)`:
2. Make sure today exists. Add a delivery per (device, person, time) for pending tasks whose time has come, if the device's account is concerned.
3. Per device: claim its deliveries (status `sending`, 2-minute lease, attempts + 1). Cancel those whose tasks are no longer pending.
4. Send one combined message, times shown in the member's display zone. Mark `sent`; or retry after 60 s × attempts; or `failed` after 3 attempts or when the subscription is gone (then it is removed).

A failing crew is logged and skipped.

### 3.7 Database schemas & tables

Migration `storage` (`infra/storage/migrations/initial_001.rs`). Both entities are `Scopable` with `tenant_col = "tenant_id"`.

#### Table: jr_crew_state

| Column | Type | Notes |
|---|---|---|
| `tenant_id` | UUID | PK; the crew |
| `revision` | BIGINT | CHECK > 0; +1 per accepted change |
| `payload` | JSONB | `State` in the `codec.rs` shape |

#### Table: jr_avatars

| Column | Type | Notes |
|---|---|---|
| `tenant_id` | UUID | PK part; the crew |
| `person_id` | VARCHAR(100) | PK part |
| `revision` | BIGINT | CHECK > 0 |
| `image` | BYTEA | NULL after removal |

### 3.8 Deployment Topology

Runs inside `juggle-server`. Config section `juggle-routines`:

| Key | Default | Set from |
|---|---|---|
| `demo` | `false` | `true` in `demo.yaml` |
| `production` | `false` | `true` in `production.yaml` |
| `vapid_private_key`, `vapid_public_key` | empty | `VAPID_PRIVATE_KEY`, `VAPID_PUBLIC_KEY` (base64url) |
| `vapid_subject` | empty | `VAPID_SUBJECT`; must start with `mailto:` or `https://`, else a built-in project URL is used |

In demo mode, start-up replaces the demo crews' state and photos through `AccessScope::for_tenant(demo crew)`, because the policy plugins are not reachable yet. Mochi the cat is a person without an account. Faces for Pip, Zed, Mochi, Nova and Ike ship in `demo-avatars/`.

## 4. Additional context

- Tests: `src/tests/` covers rules, journal replay, reminders and photo checks; `infra/pg_tests.rs` (feature `integration`) runs the store contract on PostgreSQL.
- The document is never pruned: `operations`, `events`, `days` and `deliveries` grow with use.

## 5. Traceability

- **PRD**: [PRD.md](./PRD.md)
