---
description: "How juggle-crews, its sign-in and access plugins, and juggle-crews-sdk are built."
---

# Technical Design — juggle-crews

## 1. Architecture Overview

### 1.1 Architectural Vision

Four crates under `gears/juggle-crews/`:

| Crate | Kind | Job |
|---|---|---|
| `juggle-crews` | gear (`db`, `rest`) | Owns `cj_crews` and `cj_members`. Serves the crews REST API. Registers `CrewsClientV1` and `CrewDirectoryV1` in ClientHub. |
| `juggle-crews-authn-plugin` | gear (`rest`), plugin of `authn-resolver` | Token → verified account → crew → `SecurityContext`. Serves two anonymous sign-in routes. |
| `juggle-crews-authz-plugin` | gear, plugin of `authz-resolver` | The access rule. |
| `juggle-crews-sdk` | library | The contract other gears use: traits, models, stable ids, `CrewPolicy`, demo cast. No ORM, HTTP or token types. |

Both plugins declare `deps = [types_registry, juggle_crews]`, so the crews gear has registered `CrewDirectoryV1` before they start.

### 1.2 Architecture Drivers

#### Functional Drivers

| Requirement | Design response |
|---|---|
| `cpt-cj-crews-fr-sign-in` | `SignIn` use case in the authn plugin: an `IdentityVerifier` (Firebase or demo) and `CrewDirectoryV1::sign_in`. |
| `cpt-cj-crews-fr-create-crew`, `cpt-cj-crews-fr-join-by-code`, `cpt-cj-crews-fr-owner-actions` | `CrewsService` in `domain/service.rs`. |
| `cpt-cj-crews-fr-access-rule` | `CrewAccessPolicy` in the authz plugin, asked through `authz-resolver`. |
| `cpt-cj-crews-fr-crew-contract` | `LocalClient` in `domain/local_client.rs`. |
| `cpt-cj-crews-fr-demo` | `DemoTokens` and the two anonymous routes; `CrewsService::seed_demo`. |

#### NFR Allocation

| NFR | Allocated to | Design response |
|---|---|---|
| Crew isolation | `CrewsService`, secure ORM | Every repo call takes an `AccessScope`. The service passes `AccessScope::for_tenant(crew)`, except in three named helpers. |
| No hints for refused tokens | `SignIn` | `Refusal` has three variants for the log; the caller sees one `Unauthorized`. |
| Provider outage | `SignIn` | `Refusal::KeysUnavailable` and directory errors map to `ServiceUnavailable` (503). |
| Fail closed | `CrewAccessPolicy` | A membership lookup error is logged and treated as "not a member". |

#### Key ADRs

- [crews 0001: crews live in their own gear](ADR/0001-cpt-cj-crews-adr-own-crews-gear.md)
- [0001: canonical Gears layout, plugins, typed config](../../../docs/ADR/0001-cpt-cj-adr-canonical-gears.md)

### 1.3 Architecture Layers

| Layer | `juggle-crews` | Plugins |
|---|---|---|
| `api/rest` | DTOs, handlers, routes, `RestError` | authn plugin only: two routes |
| `domain` | models, `CrewsService`, `CrewRepo` port, invite codes, `LocalClient` | `SignIn`, `IdentityVerifier` port, `CrewAccessPolicy`, plugin clients |
| `infra` | `SeaOrmCrewRepo`, entities, migration | authn plugin: `FirebaseVerifier`, `DemoTokens` |

## 2. Principles & Constraints

### 2.1 Design Principles

- **The server picks the tenant.** No request carries a crew id. Sign-in looks up the account's membership and sets the tenant.
- **Cross-crew reads have names.** Three helpers in `domain/service.rs` use `AccessScope::allow_all()`, and each says why:

  | Helper | Why it crosses crews |
  |---|---|
  | `membership_of(uid)` | Sign-in and `/me` must find the crew before any crew is known. |
  | `crew_by_code(code)` | Joining is the moment an account enters a crew it is not in yet. |
  | `all_crew_ids()` | The workers walk crews one by one. `CrewsClientV1::crew_ids` serves it to the worker identity only. |

- **The directory stays in-process.** `CrewDirectoryV1` answers across crews, so it has no REST route.
- **The crews gear is the rule's data source, not its client.** It does not ask `authz-resolver`; it builds its scopes from membership. Routines and shopping ask through `CrewPolicy`.

### 2.2 Constraints

- One crew per account: `cj_members.uid` is the primary key.
- PostgreSQL only.
- The migration is named `store`, as recorded in existing databases since v0.1. Renaming it would run it again.
- A crew holds at most 20 accounts (`MAX_MEMBERS`).

## 3. Technical Architecture

### 3.1 Domain Model

**Location**: [`juggle-crews/src/domain/model.rs`](../juggle-crews/src/domain/model.rs), [`juggle-crews-sdk/src/models.rs`](../juggle-crews-sdk/src/models.rs)

| Type | Fields | Notes |
|---|---|---|
| `Crew` | id, name, timezone, invite_code, created_at | id = tenant id. Code stored normalized (8 chars, no dash). |
| `Member` | uid, subject_id, crew_id, display_name, email, role, joined_at | `subject_id = subject_for(uid)`, a UUID v5. |
| `Me`, `CrewSummary`, `MemberSummary` | | What `/me` shows. Code shown as `XXXX-XXXX`. |
| `Role` (SDK) | `Owner`, `Member` | Stored as `owner` / `member`. |
| `AccountIdentity`, `AccountMembership`, `MembershipKind` (SDK) | | Sign-in input and output. `Member` or `Guest`; also the `subject_type`. |

A crew has 1 to 20 members. Exactly one is the owner.

### 3.2 Component Model

| ID | Where | Does | Does not |
|---|---|---|---|
| `cpt-cj-crews-component-rest` | `juggle-crews/src/api/rest` | Five authenticated routes; maps `DomainError` to RFC 9457 problems. | Hold rules. |
| `cpt-cj-crews-component-service` | `domain/service.rs` | Sign-in lookup, `/me`, create, join, rotate, update, demo seed. Validates names (1–80), zones (IANA), codes. Keeps the identity map for `/me`. | Verify tokens. |
| `cpt-cj-crews-component-repo` | `infra/storage/repo.rs` | `CrewRepo` over the secure ORM. Applies the scope it gets. Creates a crew and its owner in one transaction. | Choose scopes. |
| `cpt-cj-crews-component-local-client` | `domain/local_client.rs` | Implements both SDK traits. Admits a member of the context's crew or the worker; `crew_ids` only for the worker. | Answer over REST. |
| `cpt-cj-crews-component-authn-plugin` | `plugins/juggle-crews-authn-plugin` | Verifies the token, calls `CrewDirectoryV1::sign_in`, builds the `SecurityContext`. Serves `/config` and `/demo/sessions`. | Store anything. |
| `cpt-cj-crews-component-authz-plugin` | `plugins/juggle-crews-authz-plugin` | Evaluates access requests (3.6). `in_process::crew_policy` runs the same rule without `authz-resolver`, for tests. | Read crew tables directly. |
| `cpt-cj-crews-component-sdk` | `juggle-crews-sdk` | Traits, models, ids, `CREW_DATA` (`cf.juggle.crew.data.v1~`, action `access`), `CrewPolicy::scope`. | Depend on the gear. |

### 3.3 API Contracts

**Technology**: REST, OpenAPI from the route builders. Errors are `application/problem+json` with `error_domain = "juggle-crews.v1"`. Every answer has `Cache-Control: no-store`.

| Method | Path | Who | Answer |
|---|---|---|---|
| `GET` | `/juggle-crews/v1/me` | any signed-in caller | `MeDto`; `crew` is null for a newcomer |
| `POST` | `/juggle-crews/v1/crews` | newcomer | `MeDto` with the new crew |
| `POST` | `/juggle-crews/v1/memberships` | newcomer | `MeDto` with the joined crew |
| `POST` | `/juggle-crews/v1/crew/invite-code` | owner | `MeDto` with a new code |
| `PUT` | `/juggle-crews/v1/crew` | owner | `MeDto` with new name and zone |
| `GET` | `/juggle-crews/v1/config` | anonymous (authn plugin) | `sign_in`: `google` or `demo`; `demo_users` in demo mode |
| `POST` | `/juggle-crews/v1/demo/sessions` | anonymous, demo mode (authn plugin) | `token`: `demo.<character>.<hmac hex>` |

| Domain error | Status | `error_code` |
|---|---|---|
| `Unauthorized` (identity not in the map) | 401 | `unauthenticated` |
| `Forbidden` (not owner, no crew) | 403 | `access_denied` |
| `Invalid` (name, zone, unknown code) | 400 | `invalid_request` |
| `Conflict` (already in a crew, crew full) | 409 | `conflict` |
| `NotFound` | 404 | `not_found` |
| `Unavailable` (storage; cause logged) | 503 | `unavailable` |
| Demo session outside demo mode / unknown character | 404 / 400 | `not_found` / `invalid_request` |

**SDK traits** (ClientHub):

| Trait | Method | Who may call |
|---|---|---|
| `CrewsClientV1` | `crew(ctx)`, `members(ctx)` | a member of `ctx`'s crew, or the worker |
| `CrewsClientV1` | `crew_ids(ctx)` | the worker only |
| `CrewDirectoryV1` | `is_member(subject, crew)`, `sign_in(identity)` | the two plugins, in-process |

### 3.4 Internal Dependencies

| Dependency | Interface | Purpose |
|---|---|---|
| `types-registry` | `TypesRegistryClient` | Registers the two plugin GTS instances. |
| `authn-resolver` | `AuthNResolverPluginClient` (scoped client) | Sends bearer tokens to the sign-in plugin. |
| `authz-resolver` | `AuthZResolverPluginClient` (scoped client) | Sends access requests to the policy plugin. |
| `oagw-sdk`, `credstore-sdk` | resource type names | The access rule allows the two service identities on these types. |
| `toolkit-db` | secure ORM, `Scopable` | Applies scopes to every query. |

Both plugins register under vendor `constructorfabric` with priority 10 (config `vendor`, `priority`).

### 3.5 External Dependencies

| System | Use |
|---|---|
| PostgreSQL | `cj_crews`, `cj_members`. |
| Firebase Authentication | Production sign-in. Public keys fetched over HTTPS and cached by the response's `max-age`, else for one hour. |

### 3.6 Interactions & Sequences

#### Sign-in on every request

1. The gateway hands the bearer token to `authn-resolver`, which calls `SignInPlugin::authenticate`.
2. Tokens over 8192 bytes are refused.
3. The verifier checks the token:
   - demo: `demo.<key>.<hmac>`, HMAC-SHA256 with a per-process key, key must be a demo character;
   - Firebase: RS256; audience = project id; issuer = the Firebase issuer for that project; `exp`, `iat`, `aud`, `iss`, `sub` required; email present and verified; in `allowed_emails` if set (compared in lower case).
4. `CrewDirectoryV1::sign_in` → `membership_of(uid)`. Tenant = crew, or `lobby_tenant(uid)` if none. The identity is kept in the map for `/me`.
5. The plugin returns a `SecurityContext`: subject = `subject_for(uid)`, tenant, subject type `member` or `guest`.

A refused token → `401`. Unreachable keys or crews storage → `503`.

#### Access decision

`CrewAccessPolicy::evaluate` reads the tenant from the subject's `tenant_id` property:

| Caller | Resource / action | Answer |
|---|---|---|
| member of the crew in its context | `CREW_DATA` / `access` | allow, `owner_tenant_id = crew` |
| worker, any non-nil crew | `CREW_DATA` / `access` | allow, `owner_tenant_id = crew` |
| integration identity, platform tenant | oagw proxy, upstream, route; credstore secret `read` | allow, platform tenant only |
| secrets provisioner, platform tenant | credstore secret, any action | allow, platform tenant only |
| anyone else, anything else | | deny |

The last row also closes `/oagw/v1/*` and `/credstore/v1/*` to signed-in members. A worker is `subject_type = service` with the fixed worker subject id; a member who sets `service` is not the worker.

### 3.7 Database schemas & tables

Migration `store` (`infra/storage/migrations/initial_001.rs`). Both entities are `Scopable` with `tenant_col = "tenant_id"` and no resource, owner or type column.

#### Table: cj_crews

| Column | Type | Notes |
|---|---|---|
| `tenant_id` | UUID | PK; the crew id |
| `name` | VARCHAR(80) | |
| `timezone` | VARCHAR(64) | IANA zone |
| `invite_code` | VARCHAR(16) | UNIQUE, normalized |
| `created_at` | TIMESTAMPTZ | |

#### Table: cj_members

| Column | Type | Notes |
|---|---|---|
| `uid` | VARCHAR(128) | PK; identity-provider account id |
| `subject_id` | UUID | UNIQUE |
| `tenant_id` | UUID | FK → `cj_crews` ON DELETE CASCADE; index `cj_members_tenant` |
| `display_name` | VARCHAR(80) | |
| `email` | VARCHAR(320) | verified email |
| `role` | VARCHAR(16) | CHECK `owner` or `member` |
| `joined_at` | TIMESTAMPTZ | |

### 3.8 Deployment Topology

The three gears run inside `juggle-server`. Config sections:

| Section | Key | Default | Set from |
|---|---|---|---|
| `juggle-crews` | `demo` | `false` | `true` in `demo.yaml` |
| `juggle-crews` | `demo_timezone` | `Europe/Berlin` | `JUGGLE_DEMO_TIMEZONE` |
| `juggle-crews-authn-plugin` | `mode` | `firebase` | `demo` in `demo.yaml` |
| `juggle-crews-authn-plugin` | `firebase_project_id` | empty | `FIREBASE_PROJECT_ID` |
| `juggle-crews-authn-plugin` | `allowed_emails` | empty = open sign-up | `JUGGLE_ALLOWED_EMAILS` (comma-separated) |
| both plugins | `vendor`, `priority` | `constructorfabric`, `10` | |

In demo mode the crews gear deletes and recreates Flat 4B and Robotics Club at start-up. Their ids are UUID v5 of the crew key, so routines and shopping can seed the same crews.

## 4. Additional context

- Tests: `domain/service_tests.rs` runs the use cases on a fake repo that checks scopes; `infra/storage/pg_tests.rs` (`crews_stay_apart_in_postgres`, feature `integration`) runs them on PostgreSQL; the authz plugin's `service_tests.rs` covers each row of the access table.
- The identity map holds at most 10,000 entries and is cleared when full. Every request signs in again, so it refills.

## 5. Traceability

- **PRD**: [PRD.md](./PRD.md)
- **ADRs**: [ADR/](./ADR/)
