---
description: "What juggle-shopping does: a crew's shared list, receipt scanning and an append-only purchase ledger."
---

# PRD — juggle-shopping

## 1. Overview

### 1.1 Purpose

juggle-shopping keeps one shopping list per crew. A member photographs a receipt; the gear reads it, records the purchase in a ledger, and checks the bought items off the list. Crewmates get a push message about it.

### 1.2 Background / Problem Statement

In a shared flat nobody knows what was already bought or who paid. Typing receipts in is tedious, so it does not happen. Money records must not be silently edited, and a retried upload must not book a purchase twice.

### 1.3 Goals (Business Outcomes)

- One photo of a receipt is enough to record a purchase and update the list.
- A purchase, once booked, is never changed in place. Corrections are visible as reversals.
- The OpenAI key never reaches this gear's code.

### 1.4 Glossary

| Term | Meaning |
|---|---|
| List item | Something the crew wants to buy, with a quantity. |
| Catalog | Products the crew has listed or bought, with category and aliases. |
| Correction rule | A crew's fix for one receipt line at one merchant, given to the model next time. |
| Receipt | Photos plus what was read from them. Status `queued`, `processing`, `saved`, `review`, `failed` or `deleted`. |
| Ledger event | `purchase_recorded` or `purchase_reversed`, with postings that sum to zero in one currency. |
| Scanner | What reads receipts: OpenAI through oagw, the demo scanner, or none. |

## 2. Actors

### 2.1 Human Actors

| ID | Actor | Role |
|---|---|---|
| `cpt-cj-shopping-actor-member` | Member | Edits the list and catalog, pastes lists, uploads receipts, fixes receipts that need review, reads the ledger. |

### 2.2 System Actors

| ID | Actor | Role |
|---|---|---|
| `cpt-cj-shopping-actor-worker` | Shopping worker | Runs every 5 seconds inside the gear, one crew at a time with that crew's worker identity. Scans receipts, imports pasted lists, classifies new products, deletes expired photos. |
| `cpt-cj-shopping-actor-openai` | OpenAI | Optional. A vision model with structured outputs, called through oagw. |

## 3. Operational Concept & Environment

### 3.1 Gear-Specific Environment Constraints

| Setup | Scanner |
|---|---|
| `OPENAI_MODEL` set (and `OPENAI_API_KEY` for juggle-secrets) | OpenAI through oagw |
| no model, `demo: true` | demo scanner: picks items from the crew's own catalog and says so in the receipt's notes |
| no model, production | none: uploads are refused with a message |

## 4. Scope

### 4.1 In Scope

- The list: add (1–100 items at once), mark done, remove.
- Catalog products and correction rules.
- Pasted shopping text turned into list items by the model.
- Receipt photos: upload, scan, review, edit, delete, retry.
- The purchase ledger and its pages.
- A push message to the crew when a receipt is saved or needs review.
- Demo lists for Flat 4B and Robotics Club, rebuilt on every start in demo mode.

### 4.2 Out of Scope

- Budgets, splitting costs, payments between members.
- Holding or configuring the OpenAI key (juggle-secrets, credstore, oagw).
- Sending push messages itself (juggle-routines owns the devices).

## 5. Functional Requirements

| ID | Requirement | Actors |
|---|---|---|
| `cpt-cj-shopping-fr-commands` | Every change is a command with an `operation_id`. A repeat with the same id and command has no second effect; the same id with another command gets `409`. | member |
| `cpt-cj-shopping-fr-scan-receipt` | Accept 1–5 photos of one receipt, queue it, and let the worker read it. A clear reading is `saved`; an unclear one, or one that looks like an earlier receipt, goes to `review`. | member, worker, openai |
| `cpt-cj-shopping-fr-no-double-booking` | Uploading the same photos again returns the existing receipt instead of a new one. | member |
| `cpt-cj-shopping-fr-match-list` | When a receipt is saved, list items that were open at upload time and are covered by the receipt are checked off. | worker |
| `cpt-cj-shopping-fr-ledger` | Saving a receipt books a `purchase_recorded` event. Editing a saved receipt reverses and books again; deleting it reverses. Events are never changed or deleted. | member, worker |
| `cpt-cj-shopping-fr-photo-retention` | Receipt photos are kept 90 days, then deleted. Deleting a receipt deletes its photos at once. | worker |

## 6. Non-Functional Requirements

### 6.1 Gear-Specific NFRs

| NFR | Threshold |
|---|---|
| Crew isolation | Every read and write uses the scope `authz-resolver` returns. Writes need a scope of exactly one crew. Another crew's receipt photos answer `404`. |
| Append-only ledger | Enforced by a PostgreSQL trigger, not only by code: no `UPDATE` or `DELETE` on `js_ledger`; at least two postings that sum to zero in the row's currency. |
| No lost updates | Items, products, rules and receipts carry revisions. A stale revision gets `409 revision_conflict`. |
| Bounded work | Request body ≤ 51 MiB; each photo ≤ 10 MB (JPEG, PNG or WebP, stored as JPEG); at most 30 receipts waiting per crew; 3 scan attempts per receipt. |
| Key never in the gear | The gear calls OpenAI through oagw as the integration identity; oagw adds the key from credstore. |

## 7. Public Library Interfaces

### 7.1 Public API Surface

| Interface | Type | What it gives |
|---|---|---|
| `/juggle-shopping/v1/*` | REST | state, commands, receipt upload, receipt photos, ledger pages |

There is no SDK crate: no other gear calls this one.

### 7.2 External Integration Contracts

| Contract | Direction | Format |
|---|---|---|
| OpenAI Responses API | required (optional feature) | `POST /v1/responses` with a strict JSON schema, through oagw |

## 8. Use Cases

### Scan a receipt

**Actor**: `cpt-cj-shopping-actor-member`

**Preconditions**: a scanner is configured; the member is in a crew.

**Main Flow**:
1. Ike uploads two photos with an `operation_id` to `POST /juggle-shopping/v1/receipts`. The answer is `202` with the receipt id.
2. Within seconds the worker leases the receipt and sends the photos, the catalog and the crew's correction rules to the scanner.
3. The reading is clear. The receipt becomes `saved`, the purchase is booked, and matching open items on the list are checked off.
4. Nova gets a push message: "Ike scanned a receipt from …".

**Alternative Flows**:
- **Unclear date, currency or total, uncertain lines, or a likely duplicate**: status `review`. Nothing is booked until a member edits and saves it.
- **Scanner error**: back to `queued`, retried after 60 s × attempts; `failed` after 3 attempts. A member can retry a failed receipt while its photos exist.

## 9. Acceptance Criteria

- The e2e smoke suite adds, marks and removes list items, applies the same command twice with one effect, and uploads a receipt and reads its photos and ledger.
- `make e2e-oagw` scans a receipt through oagw against a stand-in for OpenAI, and the stand-in sees the key oagw added.
- Pip cannot read Ike's receipt photos.

## 10. Dependencies

juggle-crews (`CrewsClientV1`), juggle-routines (`RoutinesClientV1`), `authz-resolver` with `juggle-crews-authz-plugin`, oagw, credstore (through oagw), juggle-secrets (writes the key), PostgreSQL.

## 12. Risks

- Each read loads every ledger event of the crew. The operation map in the state document is never pruned. Both grow with use.
- Model output can be wrong. The `review` status and correction rules limit the damage, but a clear wrong reading is booked.

## 14. Traceability

- **Design**: [DESIGN.md](./DESIGN.md)
