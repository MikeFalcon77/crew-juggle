---
description: "What juggle-routines does: a crew's people, routines, days, check-offs, photos and push reminders."
---

# PRD — juggle-routines

## 1. Overview

### 1.1 Purpose

juggle-routines keeps each crew's recurring chores: who does what on which weekdays, what got done each day, and reminders on members' devices. It also owns the crew's push devices, so other gears send push messages through it.

### 1.2 Background / Problem Statement

A shared chore list fails in three ways: past days get rewritten when a routine changes, two people tap the same task at once, and nobody is reminded. This gear freezes each day, guards every change with a revision, and sends reminders in the crew's time zone.

### 1.3 Goals (Business Outcomes)

- Last Tuesday shows what was planned and done last Tuesday, whatever changed since.
- A retried or concurrent tap never counts twice and never overwrites a newer change.
- Members are reminded of due tasks, including tasks of people who cannot sign in.

### 1.4 Glossary

| Term | Meaning |
|---|---|
| Person | Someone routines are assigned to. Has an account (Pip) or not (Mochi the cat). |
| Routine | A task at a clock time on chosen weekdays, between a start and an optional end date. |
| Version | A routine as it applies from a date on. Edits add versions. |
| Occurrence | One routine on one day, with its status: `pending`, `done` or `cancelled`. |
| Two-step routine | A routine with a prerequisite routine; both share one slot. |
| Display zone | The zone a member wants times shown in. Days still follow the crew's zone. |

## 2. Actors

### 2.1 Human Actors

| ID | Actor | Role |
|---|---|---|
| `cpt-cj-routines-actor-member` | Member | Belongs to the crew. Plans routines and people, checks tasks off, uploads photos, subscribes devices. |

### 2.2 System Actors

| ID | Actor | Role |
|---|---|---|
| `cpt-cj-routines-actor-worker` | Reminder worker | Runs every 30 seconds inside the gear. Acts with the worker identity of one crew at a time. |
| `cpt-cj-routines-actor-shopping` | Shopping gear | Reads the crew's people and pushes receipt news to the crew through `RoutinesClientV1`. |

## 3. Operational Concept & Environment

### 3.1 Gear-Specific Environment Constraints

- Days and reminder times follow the crew's IANA zone, read from juggle-crews on every load, including DST gaps and overlaps.
- Web Push needs a VAPID key pair. Without one, a non-production server makes a throwaway pair per run; a production server refuses to start.
- `demo` and `production` cannot both be on.

## 4. Scope

### 4.1 In Scope

- People (with or without accounts), routines with versions, two-step routines, moods.
- Day views, check-offs and undo, the event trail.
- Per-member display zone.
- Person photos (square PNG).
- Push subscriptions, test pushes, due-task reminders.
- `RoutinesClientV1` for other gears.
- Demo routines for Flat 4B and Robotics Club, rebuilt on every start in demo mode.

### 4.2 Out of Scope

- Crew name, zone and membership (juggle-crews).
- Push channels other than Web Push.
- Live updates; the web app polls.

## 5. Functional Requirements

| ID | Requirement | Actors |
|---|---|---|
| `cpt-cj-routines-fr-people` | Every member account has a person, added by the server. A client cannot link a person to an account. A crew has at most 20 people. | member |
| `cpt-cj-routines-fr-schedule` | A routine change applies from a chosen date, today or later. Earlier days keep their versions. | member |
| `cpt-cj-routines-fr-frozen-days` | Today's plan follows routine edits. Once a day is past, its stored tasks never change. Future days are not shown as days, only in the schedule. | member, worker |
| `cpt-cj-routines-fr-check-off` | Only today's tasks can be checked off or undone. A second step needs its first step done; a first step cannot be undone while its second step is done. | member |
| `cpt-cj-routines-fr-confirm-undo` | Undoing someone else's check-off needs the original checker's account id in the request. | member |
| `cpt-cj-routines-fr-reminders` | When a task's time has come and it is still pending, each device of a member concerned gets one combined reminder per pass. A member is concerned with their own tasks and with tasks of people without accounts. | worker |
| `cpt-cj-routines-fr-crew-push` | Other gears can push a message to all crew devices, except one account's, through `RoutinesClientV1`. Only the worker identity may do this. | shopping |

## 6. Non-Functional Requirements

### 6.1 Gear-Specific NFRs

| NFR | Threshold |
|---|---|
| Crew isolation | Every read and write uses the scope `authz-resolver` returns for the caller. Writes are refused unless the scope admits exactly one crew. |
| Idempotent commands | Each write carries an `operation_id`. The same id with the same command returns the stored answer; with a different command, `409`. |
| No lost updates | Each changed item carries an expected revision. A stale one gets `409 revision_conflict`. |
| Bounded uploads | Request body at most 768 KiB. Photos: PNG, at most 512 KiB, exactly 256 × 256. |
| Safe push targets | Subscriptions only to HTTPS endpoints of the FCM, Mozilla, Apple and Windows push services. Pushed URLs only inside the app. |

## 7. Public Library Interfaces

### 7.1 Public API Surface

| Interface | Type | What it gives |
|---|---|---|
| `/juggle-routines/v1/*` | REST | Me, preferences, day, check-off, schedule, appearance, people, photos, push. |
| `RoutinesClientV1` | Rust trait in `juggle-routines-sdk`, ClientHub | `people(ctx)` and `notify_crew(ctx, message, skip_account)`. |

### 7.2 External Integration Contracts

| Contract | Direction | Format |
|---|---|---|
| Web Push | required | Encrypted Web Push with VAPID (ES256). |

## 8. Use Cases

### Check off a task

**Actor**: `cpt-cj-routines-actor-member`

**Preconditions**: the task is on today's plan in the crew's zone.

**Main Flow**:
1. `PUT /juggle-routines/v1/occurrences/{date}/{id}` with `operation_id`, `expected_revision`, `completed: true`.
2. The gear checks the revision and the step order.
3. The task becomes `done` with who and when; an event is added.

**Postconditions**: the task's revision is one higher. Reminders for it stop.

**Alternative Flows**:
- **Stale revision**: `409 revision_conflict`; the client refetches.
- **Undo of Zed's check-off by Pip** without `confirmed_completed_by`: `409 confirmation_required`.
- **First step not done**, or **date is not today**: `400`.

## 9. Acceptance Criteria

- The e2e smoke suite checks off and undoes a task, uploads and removes a photo, and round-trips preferences.
- Nova (Robotics Club) cannot check off a Flat 4B task or read a Flat 4B photo.
- The migration name stays `storage` (`tests/migration_names.rs`).

## 10. Dependencies

juggle-crews (`CrewsClientV1`), `authz-resolver` with `juggle-crews-authz-plugin`, PostgreSQL, browser push services.

## 12. Risks

- The crew's state is one JSON row. The operation journal, events, days and delivery records are never pruned, so the row grows for as long as the crew is used.
- Throwaway VAPID keys outside production break existing subscriptions on every restart.

## 14. Traceability

- **Design**: [DESIGN.md](./DESIGN.md)
