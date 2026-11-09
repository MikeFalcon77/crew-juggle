"""Golden snapshots of API responses.

A snapshot keeps the status, the headers the web client relies on, and the
JSON body after `normalize` has replaced values that change between runs:
dates become offsets from the server's today, timestamps, random ids, invite
codes, keys and tokens become placeholders. Key order is not compared (the
web client parses JSON), list order is.

Write or refresh snapshots with E2E_UPDATE_GOLDEN=1.
"""

from __future__ import annotations

import datetime as dt
import json
import os
import re
from pathlib import Path
from typing import Any, Callable, Optional

import httpx

GOLDEN_DIR = Path(__file__).parent / "golden"
UPDATE = os.environ.get("E2E_UPDATE_GOLDEN", "") not in ("", "0")

DATE = re.compile(r"(\d{4}-\d{2}-\d{2})")
TIMESTAMP = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$")
UUID4 = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")
PLACEHOLDER_KEYS = {
    "invite_code": "<invite-code>",
    "vapid_public_key": "<vapid-key>",
    "token": "<token>",
    "trace_id": "<trace-id>",
    "server_now": "<timestamp>",
}
# Problem bodies: the web client reads these; type, instance and context may change.
PROBLEM_KEYS = ("status", "title", "detail", "code", "error_code", "error_domain")
HEADERS = ("content-type", "cache-control")


def normalize(value: Any, today: dt.date) -> Any:
    if isinstance(value, dict):
        return {
            key: PLACEHOLDER_KEYS[key] if key in PLACEHOLDER_KEYS and item is not None
            else normalize(item, today)
            for key, item in value.items()
        }
    if isinstance(value, list):
        return [normalize(item, today) for item in value]
    if isinstance(value, str):
        if TIMESTAMP.match(value):
            return "<timestamp>"
        if UUID4.match(value):
            return "<uuid4>"
        return DATE.sub(lambda m: _relative(m.group(1), today), value)
    return value


def _relative(text: str, today: dt.date) -> str:
    try:
        day = dt.date.fromisoformat(text)
    except ValueError:
        return text
    return f"<day{(day - today).days:+d}>"


def snapshot(
    response: httpx.Response, today: dt.date, project: Optional[Callable[[Any], Any]] = None
) -> dict:
    content_type = response.headers.get("content-type", "")
    body: Any = response.json() if "json" in content_type else None
    if project is not None and response.status_code < 400:
        body = project(body)
    if content_type.startswith("application/problem+json") and isinstance(body, dict):
        body = {key: body[key] for key in PROBLEM_KEYS if key in body}
    return {
        "status": response.status_code,
        "headers": {name: response.headers.get(name) for name in HEADERS},
        "body": normalize(body, today),
    }


def assert_golden(
    name: str,
    response: httpx.Response,
    today: dt.date,
    project: Optional[Callable[[Any], Any]] = None,
) -> None:
    """Compares a response with its snapshot; `project` trims the body first."""
    actual = snapshot(response, today, project)
    path = GOLDEN_DIR / f"{name}.json"
    if UPDATE or not path.exists():
        GOLDEN_DIR.mkdir(exist_ok=True)
        path.write_text(json.dumps(actual, indent=2, sort_keys=True, ensure_ascii=False) + "\n")
        if not UPDATE:
            raise AssertionError(f"wrote new snapshot {path.name}; review it and run again")
        return
    expected = json.loads(path.read_text())
    assert actual == expected, (
        f"{name} differs from its snapshot\n"
        f"expected: {json.dumps(expected, sort_keys=True)[:2000]}\n"
        f"actual:   {json.dumps(actual, sort_keys=True)[:2000]}"
    )
