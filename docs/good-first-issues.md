# Good first issues

Each of these is small enough for a first pull request and teaches one piece of Gears or of the app. Each one is also a GitHub issue with the `good first issue` label.

## 1. A read-only role

**Why:** a club wants mentors or coaches to see the schedule without checking things off.
**What:** add `Role::Viewer` in `juggle-crews-sdk`, let the owner change a member's role (`PUT /juggle-crews/v1/crew/members/{uid}`), and make the policy plugin (`juggle-crews-authz-plugin`) allow reads but deny writes for viewers. The gears would then ask with the action ("read" or "write") instead of the single "access" they use today.
**Done when:** a viewer gets `403` on check-offs and edits, sees everything else, and a test proves it.

## 2. A second UI language

**Why:** the UI is English only.
**What:** extract UI strings in `web/src` into a small dictionary, add a second language, pick the language from the browser with a switch in Settings. Server error messages stay English for now.
**Done when:** every screen in demo mode reads naturally in the new language, and tests run in both languages.

## 3. Live updates instead of polling

**Why:** pages ask the server every 5 to 15 seconds. When Pip checks off the trash, Zed should see it at once.
**What:** publish "crew changed" events through Gears `event-broker` (tenant-scoped topics) from the routines and shopping gears, stream them to the browser, and invalidate the matching TanStack queries.
**Done when:** two browsers on the same crew update within a second, and a browser on another crew receives nothing.

## 4. Who spent how much

**Why:** "Zed, you owe me for the cat food."
**What:** a monthly summary per person from the purchase ledger (`gears/juggle-shopping/juggle-shopping/src/domain/ledger.rs`), shown on the shopping page.
**Done when:** totals match the ledger for the demo crew, with a unit test for months that cross a time-zone boundary.

## 5. Chore rotation

**Why:** the dishes should not always be Zed's problem (even if they usually are).
**What:** let a routine rotate between several people every week. The day snapshot already freezes who was assigned, so history stays correct.
**Done when:** a rotating routine assigns the right person across four weeks, including a person leaving the crew.

## 6. Several crews per account

**Why:** you live in Flat 4B and you are in the Robotics Club.
**What:** allow more than one membership per account, add a crew switcher, and send the chosen crew with each request so the sign-in plugin can put it into the security context.
**Done when:** switching crews switches every page, and isolation tests still pass.

## 7. A crew's own OpenAI key

**Why:** today one key, in the platform tenant, scans every crew's receipts. A club with its own OpenAI account should be able to bring its key.
**What:** let a crew owner store a key in credstore for their crew, and make the oagw upstream for a crew prefer it. oagw resolves upstreams along the tenant chain, so crews would need to become children of the platform tenant in a tenant-resolver plugin.
**Done when:** a crew with its own key is billed on its own account, every other crew keeps using the shared key, and no crew can read another crew's key.

## 8. Sign Web Push with the Gears crypto provider

**Why:** reminders are signed by `web-push-native`, which brings old `rsa` and `crypto-bigint` crates (see the ignores in `deny.toml`).
**What:** once `toolkit-security` publishes its P-256 signing facade, sign VAPID tokens and encrypt payloads with it, and drop `web-push-native`.
**Done when:** push notifications still arrive in Chrome and Firefox, and the two advisories leave `deny.toml`.

## 9. Dev container

**Why:** Rust, Node, protoc and Postgres in one click.
**What:** `.devcontainer/` with the toolchain, a Postgres service, and `postCreateCommand` that builds the web app.
**Done when:** "Reopen in Container" in VS Code leads to a running demo with `make run`, and `make ci` passes inside it.
