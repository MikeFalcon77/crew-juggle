---
description: "How juggle-secrets writes the OpenAI key into credstore at start-up."
---

# Technical Design — juggle-secrets

## 1. Architecture Overview

### 1.1 Architectural Vision

One small gear, `juggle-secrets` (`stateful`, `deps = [credstore]`, stop timeout 5 s). `init` reads and checks the config. The `serve` task writes each secret, then waits for shutdown.

### 1.2 Architecture Drivers

#### Key ADRs

- [0003: the OpenAI key goes through credstore](../../../docs/ADR/0003-cpt-cj-adr-openai-key-through-credstore.md)

## 3. Technical Architecture

### 3.2 Component Model

| ID | Where | Does |
|---|---|---|
| `cpt-cj-secrets-component-provisioner` | `src/domain/service.rs` | `Provisioner::put`: `CredStoreClientV1::create` with `SharingMode::Tenant`; on `Conflict`, `put` with `WritePrecondition::Exists`. Runs as `secrets_provisioner_context()`. |
| `cpt-cj-secrets-component-gear` | `src/gear.rs` | Turns config entries into `SecretRef`s (an invalid name stops start-up), drops empty values, retries each write with back-off 250 ms → 30 s until it succeeds or the server stops. |

### 3.6 Interactions & Sequences

1. The server starts; `init` loads `gears.juggle-secrets.config`.
2. `serve` writes `openai-api-key` through the credstore API as the provisioner, in the platform tenant. credstore records the metadata row; `static-credstore-plugin` keeps the value in memory.
3. The shopping gear's oagw upstream points at `cred://openai-api-key`. Its set-up retries until this write has happened.

It runs on every start because the plugin's values do not survive a restart.

### 3.8 Deployment Topology

| Key | Value in both configs |
|---|---|
| `secrets[].key` | `openai-api-key` |
| `secrets[].value` | `${OPENAI_API_KEY:-}` (empty: nothing is written) |

## 5. Traceability

- **PRD**: [PRD.md](./PRD.md)
