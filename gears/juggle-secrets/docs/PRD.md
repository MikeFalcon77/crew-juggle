---
description: "What juggle-secrets does: writes the OpenAI key into credstore when the server starts."
---

# PRD — juggle-secrets

## 1. Overview

### 1.1 Purpose

juggle-secrets writes app-wide secrets into credstore at start-up. Today there is one: `OPENAI_API_KEY`, stored as `openai-api-key`. oagw reads it from credstore and adds it to calls to OpenAI, so the shopping gear never holds the key.

### 1.2 Background / Problem Statement

credstore 0.6 serves a secret only when its own database has a metadata row for it, and only the credstore API writes that row. A secret listed in the `static-credstore-plugin` config alone cannot be read through credstore. So something must write the key through the credstore API. This gear is that something.

## 2. Actors

### 2.2 System Actors

| ID | Actor | Role |
|---|---|---|
| `cpt-cj-secrets-actor-provisioner` | Secrets provisioner | A service identity in the platform tenant (`secrets_provisioner_context()` in `juggle-crews-sdk`). The crews authz plugin allows it credstore secrets and nothing else. |

## 5. Functional Requirements

| ID | Requirement | Actors |
|---|---|---|
| `cpt-cj-secrets-fr-provision` | On every start, create each configured secret in credstore, or replace it if it exists. Skip entries with an empty value. Retry until credstore accepts. | provisioner |

## 6. Non-Functional Requirements

### 6.1 Gear-Specific NFRs

| NFR | Threshold |
|---|---|
| Key stays out of app tables | The key goes only to credstore; it is never logged (only the secret's name is). |
| Start-up never blocks | Writes run in the gear's background task, with back-off from 250 ms to 30 s. |

## 10. Dependencies

credstore with `static-credstore-plugin`; `juggle-crews-authz-plugin` (grants the provisioner its access).

## 12. Risks

- The key passes through this process at start-up. With a credstore backend that holds secrets itself, provisioning would move out of the app and this gear could go away.

## 14. Traceability

- **Design**: [DESIGN.md](./DESIGN.md)
