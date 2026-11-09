# crew-juggle.mk: settings for .gears-dna/gears.mk and the targets of crew-juggle.
#
# The Makefile includes this file first, so the settings below win over the
# `?=` defaults of gears.mk. `make help` lists every target and parameter.

# -------- Settings for gears.mk --------

# cargo-gears that `make dylint` and `make install-tools` expect. Keep it equal
# to the tag in [workspace.metadata.dylint] of Cargo.toml.
CARGO_GEARS_VERSION := 0.0.10
GEARS_CI_TARGETS := fmt clippy test deny dylint test-integration web e2e e2e-oagw

# -------- Settings for crew-juggle --------

# Postgres image for integration and e2e tests; the one place its tag lives.
PG_IMAGE ?= postgres:18-alpine
COMPOSE ?= docker compose
PYTHON ?= python3
INTEGRATION_PACKAGES := -p juggle-crews -p juggle-routines -p juggle-shopping
E2E_VENV := testing/e2e/.venv
E2E_PORT ?= 18080
E2E_PG_PORT ?= 55433
E2E_PG_CONTAINER := crew-juggle-e2e-pg
E2E_DATABASE_URL := postgres://juggle:juggle@127.0.0.1:$(E2E_PG_PORT)/juggle
# The e2e server's own ports, set the canonical way: APP__ overrides of the YAML.
E2E_SERVER_ENV = DATABASE_URL=$(E2E_DATABASE_URL) \
	APP__GEARS__API_GATEWAY__CONFIG__BIND_ADDR=127.0.0.1:$(E2E_PORT)
E2E_PYTEST = $(E2E_VENV)/bin/python -m pytest -c testing/e2e/pytest.ini --rootdir testing/e2e
E2E_MOCK_PORT ?= 18090
E2E_MOCK_LOG := target/e2e-openai-requests.log
E2E_OAGW_KEY := sk-e2e-crew-juggle
# For make e2e-oagw: the shopping gear calls a stand-in for OpenAI on loopback
# through oagw; the stand-in records which Authorization header it got.
E2E_OAGW_ENV = $(E2E_SERVER_ENV) OPENAI_API_KEY=$(E2E_OAGW_KEY) OPENAI_MODEL=e2e-vision-model \
	APP__GEARS__JUGGLE_SHOPPING__CONFIG__OPENAI_BASE_URL=http://127.0.0.1:$(E2E_MOCK_PORT) \
	APP__GEARS__OAGW__CONFIG__ALLOW_HTTP_UPSTREAM=true \
	APP__GEARS__OAGW__CONFIG__SSRF_POLICY__ENABLED=false

HELP_PARAMS += PG_IMAGE E2E_PORT
help.PG_IMAGE.usage := PG_IMAGE=<image>
help.PG_IMAGE.targets := test-integration e2e
help.PG_IMAGE.text = Postgres image for the database tests. Default: $(PG_IMAGE).
help.E2E_PORT.usage := E2E_PORT=<port>
help.E2E_PORT.targets := e2e
help.E2E_PORT.text = Port of the server that make e2e starts. Default: $(E2E_PORT).

# -------- crew-juggle --------

.PHONY: run stop web test-integration e2e e2e-env helm-lint image

# Start the demo on http://localhost:8080 (Postgres and the app in Docker)
run:
	$(call print_target_banner)
	$(COMPOSE) up -d --build
	@echo "Open http://localhost:8080 and pick a demo character."

## Stop the demo that make run started
stop:
	$(COMPOSE) down

# Test and build the web app (npm ci, vitest, vite build)
web:
	$(call print_target_banner)
	cd web && npm ci --no-audit --no-fund && npm test && npm run build

# Run the PostgreSQL tests of the gears (needs Docker)
## The `*_postgres*` tests behind the `integration` feature start Postgres with testcontainers;
## JUGGLE_PG_REQUIRE_DOCKER=1 turns a missing Docker into a failure, not a skip.
test-integration: install-tools
	$(call print_target_banner)
	$(call nextest_run,integration,$(INTEGRATION_PACKAGES) --features integration -E 'test(postgres)',JUGGLE_PG_REQUIRE_DOCKER=1 JUGGLE_PG_IMAGE=$(PG_IMAGE))

## Create the Python environment for the e2e suites
e2e-env: $(E2E_VENV)/.installed

$(E2E_VENV)/.installed: testing/e2e/requirements.txt
	$(PYTHON) -m venv $(E2E_VENV)
	$(E2E_VENV)/bin/pip install --quiet -r testing/e2e/requirements.txt
	@touch $@

# Run the end-to-end suites against a fresh demo server (needs Docker and Python 3)
## Builds the server, starts Postgres in Docker and the server
## in demo mode on E2E_PORT, runs pytest, then stops both.
e2e: e2e-env
	$(call print_target_banner)
	cargo build -p juggle-server
	$(call e2e_postgres)
	@$(call start_server_and_wait,$(E2E_SERVER_ENV) target/debug/juggle-server --config config/demo.yaml,http://127.0.0.1:$(E2E_PORT)/readyz,120); \
	E2E_BASE_URL=http://127.0.0.1:$(E2E_PORT) E2E_REQUIRE_SERVER=1 $(E2E_PYTEST) testing/e2e; \
	rc=$$?; docker rm -f $(E2E_PG_CONTAINER) >/dev/null 2>&1; exit $$rc

# Scan a receipt through oagw against a stand-in for OpenAI (needs Docker and Python 3)
## Checks that the key travels juggle-secrets -> credstore -> oagw and that
## signed-in members reach neither oagw nor credstore.
e2e-oagw: e2e-env
	$(call print_target_banner)
	cargo build -p juggle-server
	$(call e2e_postgres)
	@mkdir -p target && : > $(E2E_MOCK_LOG)
	@$(PYTHON) testing/e2e/mock_openai.py $(E2E_MOCK_PORT) $(E2E_MOCK_LOG) & MOCK=$$!; \
	$(call start_server_and_wait,$(E2E_OAGW_ENV) target/debug/juggle-server --config config/demo.yaml,http://127.0.0.1:$(E2E_PORT)/readyz,120); \
	E2E_BASE_URL=http://127.0.0.1:$(E2E_PORT) E2E_REQUIRE_SERVER=1 E2E_OAGW_MOCK_LOG=$(E2E_MOCK_LOG) \
		E2E_OAGW_KEY=$(E2E_OAGW_KEY) $(E2E_PYTEST) testing/e2e/suites/oagw; \
	rc=$$?; kill $$MOCK; docker rm -f $(E2E_PG_CONTAINER) >/dev/null 2>&1; exit $$rc

# Starts the e2e Postgres in Docker and waits until it accepts connections.
# The check goes over TCP: during first start-up the image runs a temporary
# server on the unix socket only, then restarts.
define e2e_postgres
	@docker rm -f $(E2E_PG_CONTAINER) >/dev/null 2>&1 || true
	docker run -d --name $(E2E_PG_CONTAINER) -e POSTGRES_USER=juggle -e POSTGRES_PASSWORD=juggle \
		-e POSTGRES_DB=juggle -p 127.0.0.1:$(E2E_PG_PORT):5432 $(PG_IMAGE) >/dev/null
	@for i in $$(seq 1 60); do docker exec $(E2E_PG_CONTAINER) pg_isready -h 127.0.0.1 -U juggle >/dev/null 2>&1 && break; sleep 1; done
endef

# Lint and render the Helm chart (needs helm)
helm-lint:
	$(call print_target_banner)
	$(call check_tool,helm)
	helm lint deploy/helm/crew-juggle
	helm template crew deploy/helm/crew-juggle > /dev/null

## Build the container images crew-juggle:local (server) and crew-juggle-web:local (Caddy + web app)
image:
	$(call print_target_banner)
	docker build -f deploy/Dockerfile -t crew-juggle:local .
	docker build -f deploy/Dockerfile --target web -t crew-juggle-web:local .
