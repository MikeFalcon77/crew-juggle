"""Shared fixtures for the crew-juggle end-to-end tests.

The tests talk to a running server in demo mode over HTTP. `make e2e` starts
one; to reuse a server that is already up, set E2E_BASE_URL.

Only the Robotics Club is changed by the tests. Flat 4B stays as the demo
seed left it, so the golden snapshots in suites/contract can read it. The
snapshots assume the server was started today: the demo seed is built
relative to the day it runs.
"""

from __future__ import annotations

import os
from typing import Callable, Dict, Iterator

import httpx
import pytest

BASE_URL = os.environ.get("E2E_BASE_URL", "http://localhost:8080").rstrip("/")
REQUIRE_SERVER = os.environ.get("E2E_REQUIRE_SERVER", "") not in ("", "0")


@pytest.fixture(scope="session")
def base_url() -> str:
    """The server under test; skips the run when it is not reachable."""
    try:
        response = httpx.get(f"{BASE_URL}/readyz", timeout=5)
        ready = response.status_code == 200
    except httpx.HTTPError:
        ready = False
    if not ready:
        message = f"no ready crew-juggle server at {BASE_URL}"
        if REQUIRE_SERVER:
            pytest.fail(message)
        pytest.skip(message)
    return BASE_URL


@pytest.fixture(scope="session")
def anonymous(base_url: str) -> Iterator[httpx.Client]:
    with httpx.Client(base_url=base_url, timeout=10) as client:
        yield client


@pytest.fixture(scope="session")
def as_user(base_url: str) -> Iterator[Callable[[str], httpx.Client]]:
    """Returns a client signed in as a demo character (pip, zed, nova, ike)."""
    clients: Dict[str, httpx.Client] = {}

    def client_for(user: str) -> httpx.Client:
        if user not in clients:
            response = httpx.post(
                f"{base_url}/juggle-crews/v1/demo/sessions", json={"user": user}, timeout=10
            )
            response.raise_for_status()
            token = response.json()["token"]
            clients[user] = httpx.Client(
                base_url=base_url, timeout=10, headers={"authorization": f"Bearer {token}"}
            )
        return clients[user]

    yield client_for
    for client in clients.values():
        client.close()


@pytest.fixture
def pip(as_user):
    return as_user("pip")


@pytest.fixture
def zed(as_user):
    return as_user("zed")


@pytest.fixture
def nova(as_user):
    return as_user("nova")


@pytest.fixture
def ike(as_user):
    return as_user("ike")
