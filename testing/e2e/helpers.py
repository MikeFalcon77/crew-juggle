"""Small helpers shared by the e2e suites."""

from __future__ import annotations

import uuid
from pathlib import Path

import httpx

FIXTURES = Path(__file__).parent / "fixtures"


def operation_id() -> str:
    return str(uuid.uuid4())


def fixture_bytes(name: str) -> bytes:
    return (FIXTURES / name).read_bytes()


def assert_problem(response: httpx.Response, status: int) -> dict:
    """Checks an RFC 9457 error and returns its body."""
    assert response.status_code == status, response.text
    assert response.headers["content-type"].startswith("application/problem+json")
    body = response.json()
    assert body["status"] == status
    assert body.get("title") or body.get("detail")
    assert "stack" not in body and "backtrace" not in body
    return body
