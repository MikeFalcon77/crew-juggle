---
description: "What crew-juggle does and for whom: shared routines and shopping for small groups, one deployment for many groups."
---

# PRD — crew-juggle

## 1. Overview

### 1.1 Purpose

crew-juggle lets a small group share recurring chores and one shopping list. A photo of a receipt checks the bought things off the list.

It is also an open example of an app built on Constructor Gears. The code should be easy to read and copy.

### 1.2 Background / Problem Statement

Flatmates, teams and clubs share chores and purchases. They usually do it in a group chat, where tasks get lost and nobody knows who bought what.

An app for this serves many groups at once. One missed filter in one query shows a group's data to another group. crew-juggle keeps that boundary in the platform, not in each query.

### 1.3 Goals (Business Outcomes)

- A new person can try the app in one command, with no accounts and no keys.
- No crew can see or change another crew's data, through any endpoint.
- A reader can find every Gears piece the app uses and what the app writes itself.

### 1.4 Glossary

| Term | Meaning |
|---|---|
| Crew | A group that shares the app: a flat, a team, a club. In Gears terms, a tenant. |
| Member | A person with an account in a crew. The first member is the owner. |
| Person | Someone with chores. May have no account (Mochi the cat). |
| Routine | A chore that repeats on some days. |
| Check-off | Marking a routine done for a day. |
| Receipt scan | Reading the items of a receipt photo and recording the purchase. |

## 2. Actors

### 2.1 Human Actors

#### Member

**ID**: `cpt-cj-actor-member`

Signs in with Google (or as a demo character), plans routines, checks things off, edits the list, scans receipts.

#### Owner

**ID**: `cpt-cj-actor-owner`

The member who created the crew. Can rename it, set its time zone and rotate the invite code.

#### Newcomer

**ID**: `cpt-cj-actor-newcomer`

Signed in, but in no crew yet. Can create a crew or join one with an invite code. Sees no crew data.

### 2.2 System Actors

#### Background workers

**ID**: `cpt-cj-actor-worker`

Send routine reminders and scan uploaded receipts. They act for one crew at a time.

#### OpenAI

**ID**: `cpt-cj-actor-openai`

Optional. Reads receipt photos with a vision model.

## 3. Operational Concept & Environment

One server binary and one web image. They run the same way on a laptop (`docker compose`), on one server (Caddy with HTTPS), or on Kubernetes (Helm). PostgreSQL holds all data.

Two configurations: `config/demo.yaml` (demo characters, demo data reset on every start) and `config/production.yaml` (Google sign-in, no demo data).

## 4. Scope

### 4.1 In Scope

- Crews: create, join by invite code, rename, time zone, rotate the code.
- Routines: weekly schedule, two-step routines, check-offs, undo, history per day, reminders by Web Push, people without accounts, photos.
- Shopping: list, catalog, receipt photos, receipt scanning, purchase ledger.
- Demo mode with made-up crews.

### 4.2 Out of Scope

- One account in several crews (see the good first issues).
- Roles other than owner and member.
- Payments, budgets, splitting costs.
- Live updates (pages poll every 5 to 15 seconds).

## 5. Functional Requirements

### 5.1 Crews and sign-in

| ID | Requirement |
|---|---|
| `cpt-cj-fr-sign-in` | A person signs in with a Firebase ID token (production) or a demo token (demo). |
| `cpt-cj-fr-self-service-crews` | A signed-in person without a crew can create one or join one with an invite code. No admin is needed. |
| `cpt-cj-fr-one-crew` | An account belongs to at most one crew. |

### 5.2 Routines

| ID | Requirement |
|---|---|
| `cpt-cj-fr-schedule` | Each routine has an owner (a person) and the weekdays it repeats on. |
| `cpt-cj-fr-frozen-history` | Past days keep the tasks they had. Editing a routine changes today and later days, never the past. |
| `cpt-cj-fr-two-step` | A routine can depend on another routine on the same day. It cannot be checked off before the first one. |
| `cpt-cj-fr-confirm-undo` | Undoing someone else's check-off needs that person's confirmation. |
| `cpt-cj-fr-reminders` | Members get push reminders for unfinished routines in the crew's time zone. |

### 5.3 Shopping

| ID | Requirement |
|---|---|
| `cpt-cj-fr-shared-list` | The crew shares one list. Items can be added, marked, removed. |
| `cpt-cj-fr-receipt-scan` | A receipt photo is read, the purchase is recorded, and bought items are checked off the list. Without OpenAI, demo mode uses a demo scanner and production turns scanning off. |
| `cpt-cj-fr-ledger` | Every purchase is kept in an append-only ledger. Corrections add reversing entries. |

## 6. Non-Functional Requirements

### 6.1 Gear-Specific NFRs

| ID | Requirement |
|---|---|
| `cpt-cj-nfr-isolation` | Every read and write of crew data is limited to the caller's crew by the platform's scope, not by a hand-written filter. End-to-end tests try to cross crews through every endpoint. |
| `cpt-cj-nfr-idempotency` | Repeating a command with the same operation id has no second effect. |
| `cpt-cj-nfr-concurrency` | Concurrent writes to a crew's state never lose an update. The server retries a lost race; a change based on a stale item gets `409` and the client refetches. |
| `cpt-cj-nfr-secret-handling` | The OpenAI key is never stored in app tables and never handled by the shopping code. |
| `cpt-cj-nfr-offline-list` | The shopping list opens without a network connection (cached in the browser). |

## 7. Public Library Interfaces

### 7.1 Public API Surface

REST, versioned per gear: `/juggle-crews/v1`, `/juggle-routines/v1`, `/juggle-shopping/v1`. The server publishes an OpenAPI document at `/openapi.json` and interactive docs at `/docs`.

### 7.2 External Integration Contracts

| Contract | Used for |
|---|---|
| Firebase Authentication ID tokens | Google sign-in |
| Web Push (VAPID) | Reminders |
| OpenAI Responses API with structured output | Receipt scanning, optional |

## 8. Use Cases

### Check off a chore

**ID**: `cpt-cj-uc-check-off`

Zed opens Today and taps "Do the dishes". The check-off is saved for Flat 4B, Pip sees it on the next refresh, and the reminder for it stops.

### Scan a receipt

**ID**: `cpt-cj-uc-scan-receipt`

Pip uploads a photo. The scan worker reads it, records the purchase in the ledger and checks the bought items off the list. The rest of the crew gets a push notification.

### Join a crew

**ID**: `cpt-cj-uc-join`

Ike signs in for the first time, enters the Robotics Club's invite code and lands on the club's Today page.

## 9. Acceptance Criteria

- `make ci` passes: unit, PostgreSQL, end-to-end (including isolation) and architecture lint checks.
- `docker compose up` starts the demo, and every demo character can sign in.

## 10. Dependencies

Constructor Gears 0.12 (gateway, sign-in and policy plugin slots, secure ORM, oagw, credstore), PostgreSQL, a browser with Web Push for reminders.

## 12. Risks

| Risk | Mitigation |
|---|---|
| A new endpoint forgets the crew scope | Repositories only accept an `AccessScope`; isolation tests cover every endpoint. |
| A real OpenAI key leaks through logs or config | The key comes from the environment, passes only through `juggle-secrets` into credstore, and gitleaks runs in CI. |
