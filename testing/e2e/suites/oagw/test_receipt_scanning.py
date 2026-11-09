"""Receipts scanned through oagw, with the key in credstore.

Runs only under `make e2e-oagw`, which starts a stand-in for the OpenAI API
(mock_openai.py) and a server whose shopping gear calls it through oagw. The
stand-in records the Authorization header it receives: oagw must have added
the key, because the shopping gear never had it.
"""

from __future__ import annotations

import json
import os
import time

import pytest

from helpers import fixture_bytes, operation_id

MOCK_LOG = os.environ.get("E2E_OAGW_MOCK_LOG")
KEY = os.environ.get("E2E_OAGW_KEY")

pytestmark = pytest.mark.skipif(not MOCK_LOG, reason="needs make e2e-oagw")


def wait_for_receipt(client, receipt_id: str) -> dict:
    for _ in range(40):
        receipts = client.get("/juggle-shopping/v1/state").json()["receipts"]
        receipt = next(r for r in receipts if r["id"] == receipt_id)
        if receipt["status"] not in ("queued", "processing"):
            return receipt
        time.sleep(0.5)
    raise AssertionError(f"receipt {receipt_id} was not scanned in time")


def wait_until_scanning_is_set_up(client) -> None:
    # The upstream is created once juggle-secrets has put the key into credstore.
    for _ in range(60):
        if client.get("/juggle-shopping/v1/state").json()["aiConfigured"]:
            return
        time.sleep(0.5)
    raise AssertionError("receipt scanning was not set up in time")


def test_a_receipt_is_read_through_oagw_with_the_key_from_credstore(ike):
    wait_until_scanning_is_set_up(ike)
    upload = ike.post(
        "/juggle-shopping/v1/receipts",
        data={"operation_id": operation_id()},
        files={"photo": ("receipt.jpg", fixture_bytes("receipt.jpg"), "image/jpeg")},
    )
    assert upload.status_code == 202, upload.text
    receipt = wait_for_receipt(ike, upload.json()["id"])
    assert receipt["status"] == "saved", receipt["issues"]
    assert receipt["merchant"] == "Mock Market"
    assert receipt["total"] == 449

    calls = [json.loads(line) for line in open(MOCK_LOG)]
    assert calls, "the stand-in was never called"
    assert all(call["path"] == "/v1/responses" for call in calls)
    assert all(call["authorization"] == f"Bearer {KEY}" for call in calls)


def test_members_cannot_use_the_gateway_or_read_the_key(ike):
    assert ike.post("/oagw/v1/proxy/127.0.0.1/v1/responses", json={}).status_code in (403, 404)
    assert ike.get("/credstore/v1/secrets/openai-api-key").status_code in (403, 404)
