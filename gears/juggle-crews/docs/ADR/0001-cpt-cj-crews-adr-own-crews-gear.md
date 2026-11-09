---
description: "Crews, membership and invite codes live in crew-juggle's own gear, juggle-crews, with a sign-in plugin and an access-rule plugin."
status: accepted
date: 2026-11-08
---

# Keep crews in an app gear, juggle-crews

**ID**: `cpt-cj-crews-adr-own-crews-gear`

## Context and Problem Statement

Every crew is a tenant. Its routines, list, receipts, photos and devices must stay invisible to every other crew. The app needs:

1. Google sign-in for real users, and a demo sign-in with no outside service.
2. Self-service crews: the first sign-in creates a crew, the owner shares an invite code, friends join with it. Nobody runs an admin console.
3. An answer to "may this person touch this crew's data?" for every operation, turned into a scope that the secure ORM applies.

Gears has platform gears in this area. Can crew-juggle use them, or does it need its own?

## Decision Drivers

- Crews are created by end users at runtime and joined with a code.
- Crews are flat: no hierarchy.
- PostgreSQL should be the only infrastructure.

## Considered Options

- `account-management`: tenant hierarchy, tenant types, users provisioned through the API by someone already in the tenant. Self-registration is out of its scope.
- `firebase-authn-plugin` with tenants from configuration: verifies Firebase tokens, but maps users to tenants from a config file, so a crew created at runtime cannot appear.
- An app gear for crews, with its own sign-in and access-rule plugins.

## Decision Outcome

Chosen option: **an app gear**, `juggle-crews`:

- two tables, crews and members; one account belongs to one crew;
- invite codes of eight characters with no look-alike letters; the owner can rotate them;
- `juggle-crews-authn-plugin` under `authn-resolver`: verifies a Firebase ID token (or a demo token) and puts the member's crew into the security context as the tenant. Someone with no crew gets a private "lobby" tenant that owns no data;
- `juggle-crews-authz-plugin` under `authz-resolver`: a member, or a background worker acting for one crew, may access that crew's data. The answer is the constraint `owner_tenant_id = crew`.

The routines and shopping gears depend only on `juggle-crews-sdk`, never on the crews tables.

### Consequences

- Good: no infrastructure beyond PostgreSQL.
- Good: another deployment can replace either plugin without touching the gears.
- Bad: Firebase token checks are repeated from the upstream plugin (`juggle-crews-authn-plugin/src/infra/firebase.rs`). When the upstream plugin supports looking up tenants at runtime, this plugin can use it instead.
- Bad: one account, one crew. Several crews per account need a crew switcher and a crew chosen per request (a good first issue).
