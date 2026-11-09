---
description: "The React app is served by a small Caddy image next to juggle-server, not by a gear inside it."
status: accepted
date: 2026-11-09
---

# Serve the web app from a proxy, not from a gear

**ID**: `cpt-cj-adr-web-served-by-proxy`

## Context and Problem Statement

The web app is a set of static files (`web/dist`). Gears' `api-gateway` serves gear routes, API docs and health probes, but not static files. An earlier version had a `juggle-web` gear just to serve them, plus its own `/readyz` that repeated the gateway's.

Where should the static files be served from?

## Decision Drivers

- A gear should own data or rules. Serving files is neither.
- The server should be an API only, as in gears-rust.
- One way of serving for laptop, single server and Kubernetes.

## Considered Options

- A `juggle-web` gear inside the server.
- A web image with Caddy next to the server.

## Decision Outcome

Chosen option: **a web image with Caddy**. `deploy/Dockerfile --target web` builds it from `web/` and [`deploy/web/Caddyfile`](../../deploy/web/Caddyfile). Caddy serves the files, answers every client-side page with `index.html`, and passes the API paths, `/docs`, `/openapi.json` and the probes to the server. The probes come from `api-gateway`.

### Consequences

- Good: the server has no file serving and no duplicate probes.
- Good: the same Caddyfile runs everywhere. On a single server Caddy also gets the HTTPS certificate.
- Bad: two images instead of one. Compose runs a `web` service, and Helm runs a second container in the pod.
- Bad: a new API path prefix must also be added to the Caddyfile.

### Confirmation

`docker compose up`, then open <http://localhost:8080>: the app loads, and `/readyz` and `/docs` answer through Caddy.
