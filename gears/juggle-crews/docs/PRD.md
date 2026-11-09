---
description: "What juggle-crews does: crews, membership, invite codes, sign-in and the rule that keeps each crew's data to itself."
---

# PRD — juggle-crews

## 1. Overview

### 1.1 Purpose

juggle-crews stores the crews and who belongs to each. On every request it answers two questions for the platform. **Who is this?** A sign-in token becomes a security context whose tenant is the caller's crew. **May they?** A caller may access crew data only in its own crew. The routines and shopping gears rely on both answers and never read the crews tables.

### 1.2 Background / Problem Statement

Crews are made by end users at runtime and joined with a code. The Gears `account-management` gear provisions users from inside a tenant and has no self-registration. The Gears Firebase plugin maps accounts to tenants from static config, so a crew created at runtime cannot appear there. The crews ADR records this choice.

### 1.3 Goals (Business Outcomes)

- A person signs in, then creates a crew or joins one, with no admin.
- The tenant of every request comes from the server's membership table, never from the client.
- Other gears depend only on `juggle-crews-sdk`.

### 1.4 Glossary

| Term | Meaning |
|---|---|
| Crew | A tenant. The crew id is the tenant id. |
| Lobby tenant | Tenant of a signed-in account with no crew. No rule grants access to it. |
| Invite code | 8 characters without 0, O, 1, I or L, shown as `K7QX-M2PA`. Case, spaces and dashes are ignored on input. |
| Worker identity | The service subject background jobs use, bound to one crew per context. |
| Platform tenant | The Gears default tenant. It holds the OpenAI upstream and key and is never a crew. |

## 2. Actors

### 2.1 Human Actors

| ID | Actor | Role |
|---|---|---|
| `cpt-cj-crews-actor-newcomer` | Newcomer | Signed in, in no crew. Subject type `guest`, tenant = lobby. May read `/me`, create a crew or join one. |
| `cpt-cj-crews-actor-member` | Member | Belongs to one crew. Subject type `member`. Sees the crew, its invite code and its members. |
| `cpt-cj-crews-actor-owner` | Owner | The member who created the crew. May also rename it, change its time zone and rotate the invite code. |

### 2.2 System Actors

| ID | Actor | Role |
|---|---|---|
| `cpt-cj-crews-actor-data-gear` | Crew data gears | juggle-routines and juggle-shopping. They ask `CrewsClientV1` for the crew, its members and, as the worker, the list of crews. |
| `cpt-cj-crews-actor-resolver` | Platform resolvers | `authn-resolver` and `authz-resolver`. They pass sign-in and access questions to the two plugins of this gear. |

## 3. Operational Concept & Environment

### 3.1 Gear-Specific Environment Constraints

- PostgreSQL only. The migration fails on other backends.
- Sign-in mode is set per deployment: `firebase` in `config/production.yaml`, `demo` in `config/demo.yaml`. Firebase mode will not start without a project id.
- Demo tokens are signed with a key made at start-up, so they stop working on restart.

## 4. Scope

### 4.1 In Scope

- Crews, members and invite codes.
- A sign-in plugin for `authn-resolver`: Firebase ID tokens, or demo tokens in demo mode.
- An access rule plugin for `authz-resolver`.
- The SDK: the two client traits, models, stable ids, the `CrewPolicy` helper, the demo cast.
- Demo crews Flat 4B (Pip, Zed) and Robotics Club (Nova, Ike), rebuilt on every start in demo mode.

### 4.2 Out of Scope

- One account in several crews.
- Leaving a crew, removing a member, handing over ownership, deleting a crew. Only the demo seed deletes crews.
- Roles other than owner and member.
- Sign-in other than Firebase (Google) and the demo picker.

## 5. Functional Requirements

| ID | Requirement | Actors |
|---|---|---|
| `cpt-cj-crews-fr-sign-in` | Turn a bearer token into a security context: subject = a stable id of the account, tenant = its crew or its lobby. Firebase mode accepts only tokens of the configured project with a verified email, and only listed emails when `allowed_emails` is set. | newcomer, member, resolver |
| `cpt-cj-crews-fr-create-crew` | A newcomer creates a crew (name and own display name, 1–80 characters each; an IANA time zone) and becomes its owner. | newcomer |
| `cpt-cj-crews-fr-join-by-code` | A newcomer joins the crew that owns an invite code. A crew holds at most 20 accounts. An account already in a crew cannot create or join another. | newcomer |
| `cpt-cj-crews-fr-owner-actions` | Only the owner renames the crew, changes its zone or rotates the code. The old code stops working at once. | owner |
| `cpt-cj-crews-fr-access-rule` | Allow access to crew data only to a member of the crew in the caller's context, or to the worker for one non-nil crew, and limit the answer to that one crew. | resolver, data-gear |
| `cpt-cj-crews-fr-crew-contract` | Give crew data gears the caller's crew (id, name, zone) and members, and give only the worker the list of crew ids. | data-gear |
| `cpt-cj-crews-fr-demo` | In demo mode, list the demo characters and issue a demo token for one of them. | newcomer |

## 6. Non-Functional Requirements

### 6.1 Gear-Specific NFRs

| NFR | Threshold |
|---|---|
| Crew isolation | Every storage call is limited to one crew, except three named reads across crews (see the DESIGN). Checked by unit tests, a PostgreSQL test and the e2e isolation suite. |
| No hints for refused tokens | Every refused token gets the same `401`. The reason goes to the log only. Tokens over 8192 bytes are refused before parsing. |
| Provider outage is not the caller's fault | If the Firebase signing keys cannot be fetched, sign-in answers `503`, not `401`. |
| Fail closed | If the membership check fails, the access rule denies. |
| No caching of personal data | Every crews and sign-in response carries `Cache-Control: no-store`. |

## 7. Public Library Interfaces

### 7.1 Public API Surface

| Interface | Type | What it gives |
|---|---|---|
| `/juggle-crews/v1/*` | REST | `/me`, create, join, rotate the code, update the crew; sign-in config and demo sessions. |
| `CrewsClientV1` | Rust trait in `juggle-crews-sdk`, ClientHub | The caller's crew and members; crew ids for the worker. |
| `CrewDirectoryV1` | Rust trait in `juggle-crews-sdk`, in-process only | Membership checks and sign-in for the two plugins. Answers across crews, so it has no REST. |
| `CrewPolicy`, `CREW_DATA` | Rust, `juggle-crews-sdk` | The policy enforcement helper and the one resource type crew gears protect. |
| Identity helpers | Rust, `juggle-crews-sdk` | `subject_for`, `lobby_tenant`, `worker_context`, `is_worker`, `integration_context`, `secrets_provisioner_context`, `PLATFORM_TENANT`. |

All are versioned in their names (`V1`, `/v1`).

### 7.2 External Integration Contracts

| Contract | Direction | Format |
|---|---|---|
| `authn-resolver` plugin `cf.juggle.crews_authn.plugin.v1` | provided | `AuthNResolverPluginClient`. Client-credential exchange is not supported. |
| `authz-resolver` plugin `cf.juggle.crews_authz.plugin.v1` | provided | `AuthZResolverPluginClient`. Answers with an `owner_tenant_id` constraint. |
| Firebase ID tokens | required | RS256 JWT signed with Google's published keys. |

## 8. Use Cases

### Join a crew with an invite code

**Actor**: `cpt-cj-crews-actor-newcomer`, signed in, in no crew, with a code from a member.

**Main Flow**:
1. `POST /juggle-crews/v1/memberships` with `invite_code` and `display_name`.
2. The code is normalized. It must have 8 characters and belong to a crew with fewer than 20 accounts.
3. The account is added as `member`. The answer is the new `/me`.
4. From the next request on, sign-in puts the crew on the caller's context.

**Alternative Flows**: unknown code → `400`; already in a crew or crew full → `409`.

## 9. Acceptance Criteria

- Nova (Robotics Club) gets `409` when she tries Flat 4B's invite code and stays in her crew.
- A forged demo token gets `401`.
- Signed-in members cannot reach `/oagw/v1/*` or `/credstore/v1/*`.
- Only the owner can rotate the code or update the crew (`403` for others).

## 10. Dependencies

PostgreSQL, `types-registry`, `authn-resolver`, `authz-resolver`. Firebase Authentication in production only.

## 12. Risks

- Firebase token checks are a copy of the upstream `firebase-authn-plugin` checks. A fix there must be repeated here.
- `/me` reads email and name from an in-memory map filled at sign-in. At 10,000 entries the map is cleared; a request caught by that clear gets `401` "Sign in again".

## 14. Traceability

- **Design**: [DESIGN.md](./DESIGN.md)
- **ADRs**: [ADR/](./ADR/)
