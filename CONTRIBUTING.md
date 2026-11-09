# Contributing

Thanks for wanting to help. crew-juggle is small on purpose, so most changes touch one gear and one screen.

## Get it running

Follow [Develop](README.md#develop) in the README. Before opening a pull request, run what CI runs:

```bash
make ci
```

It needs Docker (PostgreSQL tests and the end-to-end suites) and Python 3. `make help` lists the single steps, and `make test GEAR=routines` runs one gear's tests.

## Where things live

| You want to change | Look in |
|---|---|
| Routines, days, check-offs, reminders | `gears/juggle-routines/juggle-routines/src/domain` |
| Shopping list, receipts, ledger | `gears/juggle-shopping/juggle-shopping/src/domain` |
| Crews and invite codes | `gears/juggle-crews/juggle-crews/src/domain` |
| Sign-in | `gears/juggle-crews/plugins/juggle-crews-authn-plugin` |
| Who may access what | `gears/juggle-crews/plugins/juggle-crews-authz-plugin/src/domain/service.rs` |
| REST routes and their wire shapes | `gears/*/*/src/api/rest` (`routes.rs`, `dto.rs`) |
| How things are stored | `gears/*/*/src/infra/storage` |
| Screens | `web/src` |
| How the web app is served | `deploy/web/Caddyfile` |
| Demo crews and their data | `gears/juggle-crews/juggle-crews-sdk/src/demo.rs`, `gears/*/*/src/domain/demo.rs` |
| Settings | `config/demo.yaml`, `config/production.yaml`, each gear's `src/config.rs` |

[docs/DESIGN.md](docs/DESIGN.md) explains how a request flows through the gears. Each gear has its own `docs/` with a PRD and a DESIGN; update them when you change what a gear does.

## House rules

- **Keep the layers.** Rules and use cases go in `domain`, with no serde, SQL or HTTP; wire shapes go in `api/rest/dto.rs`; stored shapes in `infra/storage`. `make dylint` checks this.
- **Crew data goes through a scope.** New tables get a `tenant_id` column and a `Scopable` entity; repositories take an `AccessScope` from the use case. A read across crews needs a comment saying why, and a test.
- **Gears talk through SDK traits**, not through each other's tables or modules.
- **Add a test with the change,** in a `*_tests.rs` file next to the code. Anything that touches isolation gets a case with two crews; storage changes get a PostgreSQL test behind the `integration` feature; a change to what the API returns updates the snapshots in `testing/e2e/suites/contract/golden` (run the suite with `E2E_UPDATE_GOLDEN=1` and review the diff).
- **Settings go through the gear's `config.rs`**, never `std::env`.
- **Keep the demo honest.** Demo data is made up and lives in code. Never commit real names, emails, project IDs, hosts or keys; `.env` files are ignored for a reason.
- **English in the UI and errors** for now. A second language is welcome as a proper locale (see the good first issues).

## Pull requests

- One topic per pull request, with a short description of what changes for someone using the app.
- Screenshots for UI changes (demo mode is the easiest way to get them).
- By contributing you agree that your contribution is licensed under Apache-2.0, like the rest of the project.

Ideas to start with: [docs/good-first-issues.md](docs/good-first-issues.md).
