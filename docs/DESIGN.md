---
description: "How crew-juggle is built on Constructor Gears: the gears, the request path, crew isolation, OpenAI through oagw, and the ways to run it."
---

# Technical Design — crew-juggle

## 1. Architecture Overview

### 1.1 Architectural Vision

crew-juggle is one Rust binary, `juggle-server`, that starts the Gears runtime with Gears' own gears and four app gears. A separate web image (Caddy) serves the React app and passes API calls to the server.

The app writes only what is specific to it: crews, routines, shopping, the sign-in plugin and the access rule. Routing, sign-in and policy slots, scoped database access, outgoing calls with secrets, migrations, config and lifecycle come from Gears.

### 1.2 Architecture Drivers

#### Functional Drivers

| Requirement | Design response |
|---|---|
| `cpt-cj-fr-self-service-crews` | Own crews gear with invite codes; see [crews ADR 0001](../gears/juggle-crews/docs/ADR/0001-cpt-cj-crews-adr-own-crews-gear.md). |
| `cpt-cj-fr-receipt-scan` | Scan worker in the shopping gear; OpenAI through oagw. |

#### NFR Allocation

| NFR | Where it is met |
|---|---|
| `cpt-cj-nfr-isolation` | `juggle-crews-authz-plugin` answers with `owner_tenant_id = crew`; the secure ORM applies it to every query. |
| `cpt-cj-nfr-concurrency`, `cpt-cj-nfr-idempotency` | Crew state as one document with a revision; operation ids recorded in it. |
| `cpt-cj-nfr-secret-handling` | `juggle-secrets` → credstore → oagw; see [ADR 0003](ADR/0003-cpt-cj-adr-openai-key-through-credstore.md). |

#### Key ADRs

- [0001 Canonical Gears layout](ADR/0001-cpt-cj-adr-canonical-gears.md)
- [0002 The web app is served by a proxy, not a gear](ADR/0002-cpt-cj-adr-web-served-by-proxy.md)
- [0003 The OpenAI key goes through credstore](ADR/0003-cpt-cj-adr-openai-key-through-credstore.md)
- [crews 0001 Crews live in their own gear](../gears/juggle-crews/docs/ADR/0001-cpt-cj-crews-adr-own-crews-gear.md)

### 1.3 Architecture Layers

Inside every gear:

| Layer | Holds | Never holds |
|---|---|---|
| `api/rest/` | DTOs, handlers, routes with their OpenAPI description, error mapping | rules |
| `domain/` | models, rules, use cases (`service.rs`), ports, the SDK implementation (`local_client.rs`) | serde, SQL, HTTP |
| `infra/` | storage (entities, repository, migrations, stored JSON shape in `codec.rs`), adapters (Firebase, oagw, image normalizers) | rules |
| `config.rs`, `gear.rs` | typed config; wiring and lifecycle | |

`make dylint` runs the Gears architecture lints, which check these boundaries.

## 2. Principles & Constraints

### 2.1 Design Principles

#### The platform holds the crew boundary

No query filters by crew by hand. A use case gets an `AccessScope` and passes it to the repository; the secure ORM adds the tenant filter. Entities are `Scopable` with `tenant_id` as the tenant column. The routines and shopping gears get the scope from the policy (`authz-resolver`); the crews gear, which is the source of membership, builds it from the caller's crew itself.

#### Gears talk through SDK traits

A gear reads another gear's data only through its SDK trait in ClientHub, never through its tables. An SDK crate exists only where another gear consumes it (crews, routines).

### 2.2 Constraints

#### Wire and storage formats are stable

The REST shapes are what the web app reads, and the stored JSON is in existing databases. Changing either needs a migration path. End-to-end golden snapshots (`testing/e2e/suites/contract`) catch API changes.

## 3. Technical Architecture

### 3.2 Component Model

```
                 ┌──────────────────────────────── juggle-server ───────────────────────────────┐
web image ──────►│ api-gateway ── authn-resolver ── juggle-crews-authn-plugin   (who are you?)    │
(Caddy + React)  │      │                                                                        │
                 │      ├── juggle-crews ─────── crews, members, invite codes                    │
                 │      │        ▲ CrewsClientV1, CrewDirectoryV1 (juggle-crews-sdk)              │
                 │      ├── juggle-routines ──── people, routines, days, reminder worker         │
                 │      │        ▲ RoutinesClientV1 (juggle-routines-sdk)                         │
                 │      └── juggle-shopping ──── list, receipts, ledger, scan worker             │
                 │                                                                               │
                 │ authz-resolver ── juggle-crews-authz-plugin   (may you?)                       │
                 │ oagw ── credstore ── static-credstore-plugin ◄── juggle-secrets (key at start)  │
                 │ tenant-resolver ── single-tenant-tr-plugin, types-registry                     │
                 └──────────────────────────────────┬────────────────────────────────────────────┘
                                                    ▼
                                               PostgreSQL
```

| Gear | Owns | Docs |
|---|---|---|
| `juggle-crews` (+ SDK, two plugins) | crews, members, invite codes, sign-in, the access rule | [PRD](../gears/juggle-crews/docs/PRD.md), [DESIGN](../gears/juggle-crews/docs/DESIGN.md) |
| `juggle-routines` (+ SDK) | people, routines, days, check-offs, preferences, push devices, photos | [PRD](../gears/juggle-routines/docs/PRD.md), [DESIGN](../gears/juggle-routines/docs/DESIGN.md) |
| `juggle-shopping` | list, catalog, receipts and photos, purchase ledger | [PRD](../gears/juggle-shopping/docs/PRD.md), [DESIGN](../gears/juggle-shopping/docs/DESIGN.md) |
| `juggle-secrets` | writes the OpenAI key into credstore at start-up | [PRD](../gears/juggle-secrets/docs/PRD.md), [DESIGN](../gears/juggle-secrets/docs/DESIGN.md) |

### 3.3 API Contracts

Each gear registers its routes under `/<gear>/v1`. The gateway publishes `/openapi.json` and `/docs`, and serves `/healthz` and `/readyz`. Errors use the Gears problem format (`type`, `title`, `status`, `detail`, `error_code`).

### 3.4 Internal Dependencies

| Trait | Provider | Used by | For |
|---|---|---|---|
| `CrewsClientV1` | `juggle-crews` | routines, shopping | the caller's crew, its members and time zone; the crew list for workers |
| `CrewDirectoryV1` | `juggle-crews` | both crews plugins | membership questions before any crew is known; in-process only |
| `RoutinesClientV1` | `juggle-routines` | shopping | the crew's people (who paid), "push this to the crew" |

### 3.5 External Dependencies

#### PostgreSQL

All app tables and credstore's metadata. Every crew table has `tenant_id`.

#### Firebase Authentication

Production sign-in. The authn plugin verifies Google ID tokens against Firebase's public keys.

#### OpenAI

Optional. Called only through oagw by the shopping gear's scan worker.

### 3.6 Interactions & Sequences

#### One request

Zed taps "Do the dishes":

1. **Who is this?** The gateway passes the bearer token to `authn-resolver`, which routes it to `juggle-crews-authn-plugin`. The plugin verifies the token and asks the crews directory for Zed's crew. The security context gets subject = Zed's account, tenant = Flat 4B.
2. **May he?** `juggle-routines` asks `authz-resolver` for a scope. `juggle-crews-authz-plugin` checks that Zed is a member of Flat 4B and answers with `owner_tenant_id = Flat 4B`. The toolkit turns that into an `AccessScope`.
3. **Only his crew's rows.** The use case passes the scope to the repository, and the repository to the secure ORM, which adds the tenant filter.
4. **The change.** The domain applies the check-off and records the operation id. The write is a compare-and-swap on the state's revision; the server retries a lost race.

Nova from the Robotics Club gets her own crew in step 1, so the same code reads only the Robotics Club's rows. `testing/e2e/suites/isolation` tries to cross crews through the API.

#### Calling OpenAI

1. At start-up, `juggle-secrets` writes `OPENAI_API_KEY` into credstore as the *secrets provisioner*, a service identity in the platform tenant.
2. `juggle-shopping` creates the OpenAI upstream in oagw as the *integration* identity, with API-key auth that points at the credstore secret.
3. To scan a receipt, the worker reads the crew's state with its worker identity for that crew, then calls `/{host}/v1/responses` through oagw as the integration identity. oagw reads the key from credstore and adds the `Authorization` header.

The policy plugin allows those two identities only oagw and credstore, and only in the platform tenant. Signed-in members get neither. `make e2e-oagw` runs this path against a stand-in for OpenAI and checks the header it receives.

### 3.7 Database schemas & tables

| Gear | Tables |
|---|---|
| `juggle-crews` | `cj_crews`, `cj_members` |
| `juggle-routines` | `jr_crew_state`, `jr_avatars` |
| `juggle-shopping` | `js_state`, `js_photos`, `js_ledger` |
| `credstore` (Gears) | its own |

Three reads in `juggle-crews` cross crews on purpose, each in one helper of `domain/service.rs` that says why: the crew of an account (sign-in), the crew of an invite code (joining), and the crew list (workers).

### 3.8 Deployment Topology

| Way | What runs |
|---|---|
| Laptop: `docker compose up` | `db`, `app` (server, `config/demo.yaml`), `web` on port 8080 |
| One server: `deploy/single-host` | the same three; `web` gets a certificate for your domain |
| Kubernetes: `deploy/helm/crew-juggle` | one pod with `app` (port 8081) and `web` (port 8080); Postgres in the release or your own |

`deploy/Dockerfile` builds both images: the default target is the server, `--target web` is Caddy with the built app and [`deploy/web/Caddyfile`](../deploy/web/Caddyfile).

## 4. Additional context

### Demo mode

`config/demo.yaml`:

- `juggle-crews-authn-plugin` runs in `mode: demo`. `GET /juggle-crews/v1/config` lists the characters, and `POST /juggle-crews/v1/demo/sessions` issues an HMAC-signed token. The key is made at start-up, so tokens end with the process.
- The crews, routines and shopping gears rebuild the demo crews on every start.
- Without `OPENAI_MODEL`, receipts go to a demo scanner that picks items from the crew's own catalog and says so in the receipt's notes.

`config/production.yaml` turns all of that off: Google sign-in only, no demo data, no scanning until `OPENAI_MODEL` and `OPENAI_API_KEY` are set, and real push keys required.

### Frontend

React 19, TanStack Query and Vite in `web/`. The service worker caches the app shell and shows push notifications. The shopping list is kept in IndexedDB, so it opens without a connection.
