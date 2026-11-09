---
description: "Every crew-juggle gear follows the Gears layout, uses Gears plugin slots and typed config, and reaches OpenAI only through oagw."
status: accepted
date: 2026-11-09
---

# Follow the canonical Gears layout in every gear

**ID**: `cpt-cj-adr-canonical-gears`

## Context and Problem Statement

crew-juggle is an example that students read to learn how a Gears app is built. A review against the Gears conventions (the `simple-user-settings` reference gear, `docs/toolkit_unified_system`, the `cargo gears` lints) found that only one gear followed them. Two gears were flat files, so the layer lints never checked them. The shopping domain used serde for both the API and storage. The access rule ran inside an SDK crate. Gears read environment variables. OpenAI was called directly with a key from the environment.

Should the example follow the conventions fully, or stay as it was?

## Decision Drivers

- The example must look the way Gears asks apps to look.
- The lints must actually check every gear.
- The web app and existing databases must keep working without changes.

## Considered Options

- Keep the old structure and document the differences.
- Follow the conventions fully.

## Decision Outcome

Chosen option: **follow the conventions fully**.

- Gears 0.12.0.
- `gears/<gear>/<crate>`, `apps/juggle-server`, `libs/juggle-testing`. Every gear has `api/rest`, `domain`, `infra`, `config.rs`, `gear.rs`. An SDK crate only where another gear consumes it.
- No serde in `domain`. REST DTOs keep the wire format the web app reads; `infra/storage/codec.rs` keeps the stored JSON, so old rows and command fingerprints stay valid.
- Sign-in is a plugin of `authn-resolver`, the access rule a plugin of `authz-resolver`. Gears ask through `PolicyEnforcer`.
- Each gear reads its own typed config section. `config/demo.yaml` and `config/production.yaml` replace mode switches in the environment.
- OpenAI only through oagw; see [ADR 0003](0003-cpt-cj-adr-openai-key-through-credstore.md).
- `make` from gears-rust: `Makefile` + `crew-juggle.mk` + an unmodified `.gears-dna/gears.mk`. CI runs the same targets.

### Consequences

- Good: `make dylint` checks every gear, with no exceptions beyond the camelCase DTOs (lint DE0803), which the web app's wire format needs.
- Good: golden API snapshots taken before the change still pass, and a database created before it opens without new migrations.
- Bad: ten crates for three features. Each is small and has one job; [DESIGN](../DESIGN.md) is the map.
- Bad: `toolkit-security` 0.12.0 has no JWT helper yet, so the sign-in plugin uses `jsonwebtoken` and `aws-lc-rs` directly.

### Confirmation

`make ci`: fmt, clippy, unit tests, `deny`, `dylint`, PostgreSQL tests, web tests, end-to-end suites.
