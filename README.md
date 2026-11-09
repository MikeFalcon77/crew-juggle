# crew-juggle

**Shared routines and shopping for your crew.** Chores on repeat, a shared shopping list, and receipts that check things off by themselves. One app, many crews: a flat, a team, a club. Nobody outside a crew sees its stuff.

It is an open-source example app built in Rust on [Constructor Gears](https://github.com/constructorfabric/gears-rust): the gateway, sign-in, permission checks, crew-scoped database access and API docs come from Gears; the app code is the routines, the shopping and the crews themselves.

<p align="center">
  <img src="web/public/icon.svg" width="96" alt="">
</p>

## Try it in one command

```bash
docker compose up
```

Open <http://localhost:8080> and pick a character:

| Crew | Who | Deal |
|---|---|---|
| **Flat 4B** | **Pip** | Plans everything. Started this app. Already checked off today. |
| | **Zed** | Builds robots in the kitchen. The dishes are overdue. |
| | **Mochi** 🐈 | No account. Has chores anyway (“Supervise kitchen cleaning 😼”). |
| **Robotics Club** | **Nova**, **Ike** | Lock the workshop, check the chargers, buy snacks. |

Things to try:

- Check off Zed's dishes as Zed, then try undoing it as Pip: the app asks you to confirm someone else's check-off.
- Try “Hang the laundry” before “Start the laundry” (Sundays): a second step waits for its first step.
- Scan any photo as a receipt. Without an OpenAI key, a **demo scanner** pretends to read it, saves a purchase and checks the bought things off the list.
- Sign in as Nova: Flat 4B's routines, list and receipts simply do not exist for her.
- Open <http://localhost:8080/docs>: every gear's API, generated from the routes it declares.

Demo data resets whenever the server restarts.

## How it is built

```
browser ──► web (Caddy): the React app; API calls go on to juggle-server
              │
              ▼
            api-gateway ──► authn-resolver ──► juggle-crews-authn-plugin   who are you?
            (Gears)    │                          (Firebase or demo token)
                       ├──► juggle-crews        crews, members, invite codes
                       ├──► juggle-routines     routines, check-offs, reminders, people
                       └──► juggle-shopping     list, receipts, purchase ledger
                                 │
     every gear with crew data asks authz-resolver ──► juggle-crews-authz-plugin   may you?
                                 │                      (members of the crew, or the worker)
                                 ▼
                     reads and writes only through the crew's scope (secure ORM, PostgreSQL)

receipt scanner ──► oagw ──► OpenAI        the key comes from credstore, put there at start-up
                                           by juggle-secrets; the shopping gear never holds it
```

| Piece | Where it comes from |
|---|---|
| HTTP gateway, routing, request IDs, problem-format errors, `/docs`, `/readyz` | Gears (`api-gateway`, toolkit) |
| Sign-in plugin slot, policy plugin slot | Gears (`authn-resolver`, `authz-resolver`) |
| Sign-in (Firebase ID tokens, demo characters) | this app, as a plugin: `gears/juggle-crews/plugins/juggle-crews-authn-plugin` |
| The rule "members may access their crew's data" | this app, as a plugin: `gears/juggle-crews/plugins/juggle-crews-authz-plugin` |
| Policy enforcement and crew-scoped queries | Gears toolkit (`PolicyEnforcer`, secure ORM with `AccessScope`) |
| Calls to OpenAI with a key nobody in the app sees | Gears (`oagw`, `credstore`) |
| Start order, lifecycle, migrations, typed config | Gears runtime |
| Routines, shopping, crews, the web app | this app |
| Serving the web app | Caddy, in a small image of its own ([why](docs/ADR/0002-cpt-cj-adr-web-served-by-proxy.md)) |

Every gear has the same shape: `api/rest` (DTOs, handlers, routes), `domain` (rules and use cases, no serde, no SQL, no HTTP) and `infra` (storage, adapters). The architecture lints of `cargo gears` check it on every build of CI (`make dylint`).

Gears talk to each other through SDK crates over ClientHub, never through each other's tables:

- `juggle-routines` and `juggle-shopping` ask `juggle-crews-sdk` for the crew's members and time zone;
- `juggle-shopping` asks `juggle-routines-sdk` to push "Pip scanned a receipt" to the rest of the crew (the routines gear owns the devices).

Documents, in the gears-rust format:

- the app: [PRD](docs/PRD.md), [DESIGN](docs/DESIGN.md), [decision records](docs/ADR);
- each gear: `gears/<gear>/docs/` ([crews](gears/juggle-crews/docs), [routines](gears/juggle-routines/docs), [shopping](gears/juggle-shopping/docs), [secrets](gears/juggle-secrets/docs)).

## Run it anywhere

The same two images (the server and the web app), three ways:

- **Your laptop:** `docker compose up` (above).
- **One server:** [deploy/single-host](deploy/single-host) runs Postgres, the server and the web image, which gets an HTTPS certificate for your domain. Rent a small VPS, point a domain at it, fill in `.env`, `docker compose up -d --build`.
- **Kubernetes:** [deploy/helm/crew-juggle](deploy/helm/crew-juggle):

  ```bash
  make image
  kind load docker-image crew-juggle:local crew-juggle-web:local
  helm install crew deploy/helm/crew-juggle
  kubectl port-forward svc/crew 8080:80
  ```

For real users, start with `config/production.yaml` (the image's default) and configure Google sign-in, push keys and, optionally, receipt scanning; see [Configuration](#configuration).

## Develop

Requirements: Rust (the version in `rust-toolchain.toml`), Node 24+, `protoc`, Docker, Python 3 (for the end-to-end tests), GNU Make.

```bash
make help        # every target and parameter
make run         # the demo in Docker, as above
make ci          # everything CI checks: fmt, clippy, tests, deny, dylint, PostgreSQL tests, web, e2e
```

`make` comes from Gears: [`.gears-dna/gears.mk`](.gears-dna/gears.mk) is an unmodified copy of the shared rules of [gears-rust](https://github.com/constructorfabric/gears-rust), and [`crew-juggle.mk`](crew-juggle.mk) holds this project's settings and targets. Useful ones:

| Target | What it runs |
|---|---|
| `make test` | Unit tests (nextest) |
| `make test-integration` | Storage tests on a real PostgreSQL in Docker (testcontainers) |
| `make e2e` | A fresh demo server and the end-to-end suites: API snapshots, crew isolation, every endpoint |
| `make e2e-oagw` | A receipt scanned through oagw against a stand-in for OpenAI |
| `make dylint` | The Gears architecture lints |
| `make deny` | Licenses, advisories and sources of every dependency |
| `make web` | Web tests and build |

To run the server from source:

```bash
docker run -d --name juggle-db -e POSTGRES_USER=juggle -e POSTGRES_PASSWORD=juggle -p 5432:5432 postgres:18-alpine
cp .env.example .env
cargo run -p juggle-server -- --config config/demo.yaml
```

In a second terminal, the web app with live reload on <http://localhost:5173> (Vite passes API calls to the server on port 8080):

```bash
npm ci --prefix web && npm run dev --prefix web
```

## Configuration

Two files, one per way to run: [`config/demo.yaml`](config/demo.yaml) is the sandbox (character sign-in, demo data reset on start), [`config/production.yaml`](config/production.yaml) is the real thing and the image's default. Each gear reads its own section. Values written as `${VAR}` come from the environment:

| Variable | What it does |
|---|---|
| `DATABASE_URL` | PostgreSQL connection string. |
| `FIREBASE_PROJECT_ID` | Google sign-in through Firebase (production). |
| `JUGGLE_ALLOWED_EMAILS` | Optional comma-separated allowlist for private deployments. |
| `VAPID_PRIVATE_KEY`, `VAPID_PUBLIC_KEY`, `VAPID_SUBJECT` | Web Push identity (`npx web-push generate-vapid-keys`); required in production. |
| `OPENAI_API_KEY`, `OPENAI_MODEL` | Real receipt scanning through oagw with a vision model that supports structured outputs. Without a model: the demo scanner in demo mode, no scanning in production. |
| `JUGGLE_DEMO_TIMEZONE` | Calendar zone of the demo crews (default `Europe/Berlin`). |
| `VITE_FIREBASE_*` | Public Firebase web config, baked into the web build. |

Any other value can be overridden without editing a file: `APP__GEARS__<GEAR>__CONFIG__<KEY>`, for example `APP__GEARS__API_GATEWAY__CONFIG__BIND_ADDR=0.0.0.0:9000`.

## Contributing

Ideas that make good first pull requests are in [docs/good-first-issues.md](docs/good-first-issues.md). Read [CONTRIBUTING.md](CONTRIBUTING.md) first.

## License

[Apache-2.0](LICENSE). All people, crews and receipts in the demo are made up.
