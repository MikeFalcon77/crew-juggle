---
description: "The OpenAI key is written into credstore at start-up by the small juggle-secrets gear, and only oagw adds it to requests."
status: accepted
date: 2026-11-09
---

# Keep the OpenAI key in credstore and call OpenAI through oagw

**ID**: `cpt-cj-adr-openai-key-through-credstore`

## Context and Problem Statement

Receipt scanning calls OpenAI with an API key. The key belongs to the deployment, not to any crew. The shopping gear should not hold it, and no crew should be able to read it.

Gears has `oagw` (outgoing calls that add credentials) and `credstore` (secrets with an owner tenant). How does the key get into credstore?

## Decision Drivers

- The shopping code never handles the key.
- The key is set the usual way: the `OPENAI_API_KEY` environment variable.
- No crew, and no signed-in member, can read the key or use oagw directly.

## Considered Options

- List the key in the `static-credstore-plugin` config.
- A small gear that writes the key through the credstore API at start-up.
- Let the shopping gear write the key before it creates the upstream.

## Decision Outcome

Chosen option: **a small gear, `juggle-secrets`**. At start-up it writes the key into credstore as the *secrets provisioner*, a service identity in the platform tenant. The shopping gear creates the OpenAI upstream in oagw with API-key auth that points at the secret, and calls it as the *integration* identity. oagw reads the key and adds the header.

The plugin config option does not work with credstore 0.6: credstore serves a secret only when its own database has a metadata row for it, and only the credstore API writes that row. A key listed only in the plugin config is invisible to oagw.

### Consequences

- Good: the shopping gear never sees the key. Only `juggle-secrets` handles it, for a moment at start-up.
- Good: `juggle-crews-authz-plugin` allows the two service identities only oagw and credstore, only in the platform tenant. Members get neither.
- Bad: one more gear, about 150 lines. It goes away if credstore learns to import secrets from configuration.

### Confirmation

`make e2e-oagw` scans a receipt against a stand-in for OpenAI, checks that the stand-in got `Authorization: Bearer <key>`, and checks that a member is refused by oagw and credstore.
