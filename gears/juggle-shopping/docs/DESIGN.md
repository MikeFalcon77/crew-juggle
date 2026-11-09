---
description: "How juggle-shopping is built: crew state as one revisioned document, idempotent commands, an append-only ledger and receipt scanning through oagw."
---

# Technical Design — juggle-shopping

## 1. Architecture Overview

### 1.1 Architectural Vision

One gear, `juggle-shopping` (`db`, `rest`, `stateful`), with no SDK crate. It declares `deps = [juggle_crews, juggle_routines, authz_resolver, oagw]`.

| Data | Where |
|---|---|
| List, catalog, rules, receipts, imports, operation map | one JSON document per crew in `js_state`, with a revision |
| Receipt photos | `js_photos`, one row per photo, with an expiry |
| Purchase events | `js_ledger`, append-only, guarded by a trigger |

A background loop (every 5 s) does the slow work: scanning, imports, classification, photo expiry.

### 1.2 Architecture Drivers

#### Functional Drivers

| Requirement | Design response |
|---|---|
| `cpt-cj-shopping-fr-commands` | `Service::command`: the operation map stores `operation_id → fingerprint` (`JsonFingerprint` of the action). |
| `cpt-cj-shopping-fr-scan-receipt` | `Service::upload_photos` queues; the worker leases and calls `Intelligence::receipt`; `State::finish_receipt` decides `saved` or `review`. |
| `cpt-cj-shopping-fr-no-double-booking` | SHA-256 digest over the normalized photos; an existing non-deleted receipt with that digest is returned. |
| `cpt-cj-shopping-fr-match-list` | `State::match_list` on save, against the list items recorded as `eligible` at upload. |
| `cpt-cj-shopping-fr-ledger` | `domain/ledger.rs` events; `State::book` reverses the active event, then records. |
| `cpt-cj-shopping-fr-photo-retention` | `expires_at` = upload + 90 days; `Store::expire` per worker pass; commit deletes photos of deleted receipts. |

#### NFR Allocation

| NFR | Allocated to | Design response |
|---|---|---|
| Crew isolation | `Service::scope`, `PgStore` | `CrewPolicy::scope` on every operation. `crew_of(scope)` refuses scopes that are not exactly one crew. |
| Append-only ledger | migration, `PgStore::commit` | Trigger `js_ledger_guard`; the repo also validates balance before insert and inserts only new events. |
| No lost updates | `Store::commit`, item revisions | Row update `WHERE revision = r`; `Service::mutate` retries up to 8 times, then `409`. |
| Key never in the gear | `OpenAiUpstream`, `OpenAiScanner` | Upstream auth points at `cred://openai-api-key`; requests go through `ServiceGatewayClientV1::proxy_request`. |

#### Key ADRs

- [0001: canonical Gears layout, plugins, typed config](../../../docs/ADR/0001-cpt-cj-adr-canonical-gears.md)
- [0003: the OpenAI key goes through credstore](../../../docs/ADR/0003-cpt-cj-adr-openai-key-through-credstore.md)

### 1.3 Architecture Layers

| Layer | Holds |
|---|---|
| `api/rest` | DTOs (`dto.rs`, the web app's wire shape, some camelCase fields), handlers, routes, problem mapping |
| `domain` | `State` rules (`state.rs`, `rules.rs`), ledger, `Service`, worker, demo lists, ports `Store`, `Intelligence`, `ActionFingerprint`, `PhotoNormalizer` |
| `infra` | `PgStore` (entities, migration, `codec.rs` for stored JSON), `OpenAiScanner` / `OpenAiUpstream`, `DemoScanner`, `NoScanner`, `JpegNormalizer` |

## 2. Principles & Constraints

### 2.1 Design Principles

- **This gear is a policy enforcement point.** It never builds a crew scope itself, except for demo seeding at start-up.
- **Money is append-only.** The database refuses changes even if the code had a bug. Even the demo reset adds reversals.
- **The model is untrusted.** Its instructions say to treat image and user text as data. Its output must match a strict JSON schema; unclear readings go to `review`.
- **No scanner, no fake data in production.** Without a model, production uses `NoScanner`, and uploads are refused with a message.

### 2.2 Constraints

- PostgreSQL only. Migration name `m20261031_shopping` is fixed since v0.1.
- 1–5 photos per receipt; each ≤ 10 MB; JPEG, PNG or WebP; stored as upright JPEG.
- At most 30 receipts `queued` or `processing` per crew.
- Pasted text ≤ 10,000 characters; an import yields 1–100 items.
- Model response ≤ 4 MiB; catalog context ≤ 500 products; ≤ 500 correction rules.

## 3. Technical Architecture

### 3.1 Domain Model

**Location**: [`juggle-shopping/src/domain/model.rs`](../juggle-shopping/src/domain/model.rs), [`ledger.rs`](../juggle-shopping/src/domain/ledger.rs)

| Type | Notes |
|---|---|
| `State` | `list`, `catalog`, `receipts`, `imports`, `rules`, `operations`, `classification_next_at`; plus `events` loaded from `js_ledger` |
| `ListItem`, `Product`, `CorrectionRule`, `Receipt` | each carries a `revision` |
| `Receipt` | photos digest, status, attempts, lease, `eligible` list snapshot, `matched_ids`, `suspected_duplicate`, `photo_expires_at` |
| `Command` | `operation_id` + `Action` |
| `Action` | `add`, `mark`, `remove`, `product`, `rule`, `rule_delete`, `edit_receipt`, `delete_receipt`, `retry`, `import` |
| `Event` | `purchase_recorded` or `purchase_reversed` (`reverses` = event id); `Posting`s |

A recorded purchase has one `expense` posting per item and one `payment:unallocated` posting that balances them.

### 3.2 Component Model

| ID | Where | Does |
|---|---|---|
| `cpt-cj-shopping-component-rest` | `api/rest` | 5 routes; multipart upload; problems with `error_code`. |
| `cpt-cj-shopping-component-service` | `domain/service.rs` | `scope`, `member`, `mutate` (CAS loop), `command`, `upload_photos`, `ledger_page`, `receipt_photos`, `announce`. |
| `cpt-cj-shopping-component-worker` | `domain/worker.rs` | `tick` over all crews; per crew under `worker_context(crew)`: expire photos, then one receipt, else one import, else one classification batch. |
| `cpt-cj-shopping-component-store` | `infra/storage/repo.rs` | `ensure`, `read` (state row with a share lock plus all ledger events), `commit` (one transaction), `photos`, `expire`, demo `reset`. |
| `cpt-cj-shopping-component-openai` | `infra/openai` | `OpenAiUpstream` creates the oagw upstream and route; `OpenAiScanner` calls the Responses API through oagw. |
| `cpt-cj-shopping-component-scanners` | `infra/demo_scanner.rs`, `infra/no_scanner.rs` | Demo readings from the crew's catalog; or "not set up". |

### 3.3 API Contracts

**Technology**: REST. Every route is authenticated and needs a member of the crew. Errors are problems with `error_code` (no `error_domain`) and `Cache-Control: no-store`.

| Method | Path | Does |
|---|---|---|
| `GET` | `/juggle-shopping/v1/state` | `revision`, list, catalog, rules, receipts (not deleted), imports (not done), purchases, `aiConfigured`, `today` |
| `POST` | `/juggle-shopping/v1/commands` | `{operation_id, action: {kind, …}}` → `{ok: true}` |
| `POST` | `/juggle-shopping/v1/receipts` | multipart `operation_id` + `photo` × 1–5 → `202 {id}` |
| `GET` | `/juggle-shopping/v1/receipts/{id}/photos` | `data:image/jpeg;base64,…` URLs, or `expired: true` |
| `GET` | `/juggle-shopping/v1/ledger?receipt_id=&after=` | up to 200 events, oldest first, and `next` cursor |

| Error | Status | `error_code` |
|---|---|---|
| `Forbidden` | 403 | `access_denied` |
| `Conflict` | 409 | `revision_conflict` |
| `NotFound` | 404 | `not_found` |
| `Storage` | 503 | `unavailable` |
| `Invalid`, `Model` | 400 | `invalid_request` |

### 3.4 Internal Dependencies

| Dependency | Interface | Purpose |
|---|---|---|
| juggle-crews | `CrewsClientV1` | members (display name of the actor), crew zone, crew ids for the worker |
| juggle-routines | `RoutinesClientV1::notify_crew` | push "receipt saved / needs a check" to the crew, except the uploader; best effort |
| `authz-resolver` | `PolicyEnforcer` via `CrewPolicy` | scope per operation |
| oagw | `ServiceGatewayClientV1` | create upstream and route; proxy requests |
| `toolkit-db` | secure ORM | scoped queries and transactions |

### 3.5 External Dependencies

#### OpenAI (optional)

- Reached only through oagw. Upstream alias = the host of `openai_base_url`; route `POST /v1/responses`; auth plugin API key, header `authorization`, prefix `Bearer `, secret `cred://<openai_secret_ref>`.
- Called as `integration_context()`: a service identity in the platform tenant, allowed only oagw and credstore reads by the crews authz plugin.
- `provision_openai` retries with back-off (0.5 s to 60 s) until the key is in credstore. Until then `OpenAiScanner::configured()` is false, so queued receipts keep their attempts.
- Request: `store: false`, strict JSON schema, photos as JPEG data URLs, the crew's catalog and rules as context.

### 3.6 Interactions & Sequences

#### Command

1. `member(ctx)` → actor's display name.
2. `mutate`: scope, `ensure` the row, `read` (revision + state + events).
3. Known `operation_id`: same fingerprint → done; different → `409`. Else `State::apply`, record the fingerprint.
4. `commit(scope, revision, state, photos)`: in one transaction, update the row `WHERE revision = r`, insert new ledger events, insert new photos, delete photos of deleted receipts. Lost race → retry.

#### Receipt scan (worker)

1. Lease: pick a receipt that is `queued` and due, or `processing` with an expired lease, with photos still kept. Set `processing`, a 3-minute lease, attempts + 1.
2. Load its photos; call the scanner outside any transaction.
3. Write back only if the lease is still ours. Success → `finish_receipt` (`saved` books and matches the list; `review` waits for a member). Failure → `queued` with retry after 60 s × attempts, or `failed` at 3.
4. On success, `announce` to the crew.

### 3.7 Database schemas & tables

Migration `m20261031_shopping` (`infra/storage/migrations/mod.rs`). All three entities are `Scopable` with `tenant_col = "tenant_id"`.

#### Table: js_state

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK; equals the crew id |
| `tenant_id` | UUID | the crew |
| `revision` | BIGINT | CHECK > 0 |
| `payload` | JSONB | `State` in the `codec.rs` shape |

#### Table: js_photos

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK; UUID v5 of receipt id and ordinal |
| `tenant_id` | UUID | |
| `receipt_id`, `ordinal` | TEXT, INTEGER | UNIQUE with `tenant_id` |
| `image` | BYTEA | JPEG |
| `expires_at` | TIMESTAMPTZ | index `js_photos_expiry` |

#### Table: js_ledger

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK; event id |
| `tenant_id` | UUID | |
| `receipt_id` | TEXT | index with `tenant_id`, `sequence` |
| `sequence` | BIGSERIAL | UNIQUE; insertion order |
| `purchase_date` | DATE | index with `tenant_id`, `currency`, `sequence` |
| `recorded_at` | TIMESTAMPTZ | |
| `currency` | TEXT | |
| `payload` | JSONB | the event, with `postings` |

Trigger `js_ledger_guard` (BEFORE INSERT OR UPDATE OR DELETE): raises "Financial ledger is append-only" on anything but insert, and "Financial postings must balance in one currency" unless `postings` is an array of at least two whose `amount`s sum to 0 and whose `currency` equals the row's.

### 3.8 Deployment Topology

Runs inside `juggle-server`. Config section `juggle-shopping`:

| Key | Default | Set from |
|---|---|---|
| `demo` | `false` | `true` in `demo.yaml` |
| `openai_model` | empty | `OPENAI_MODEL`; empty means no OpenAI |
| `openai_base_url` | the public OpenAI API | |
| `openai_secret_ref` | `openai-api-key` | must match the key juggle-secrets writes |

oagw runs with `proxy_timeout_secs: 120` and `management_api_enabled: false`, so upstreams are created only by this gear.

In demo mode, start-up reverses every booked purchase ("Demo reset"), deletes the demo crews' `js_state` and `js_photos` rows, and writes fresh demo lists, through `AccessScope::for_tenant(demo crew)`.

## 4. Additional context

- Tests: `domain/state_tests.rs` and `service_tests.rs` for rules; `infra/storage/codec_tests.rs` for the stored shape; `infra/pg_tests.rs` (feature `integration`) for the store and the trigger on PostgreSQL; `make e2e-oagw` for the oagw path.
- New products typed by members are classified in batches of 12, at most every 5 minutes per crew.

## 5. Traceability

- **PRD**: [PRD.md](./PRD.md)
