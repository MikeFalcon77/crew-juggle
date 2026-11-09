"""Two crews on one server: Nova (Robotics Club) must not see or change Flat 4B.

Every gear decides the crew from the signed-in account, never from the
request, so the only way to reach another crew's data would be a missing
scope. These tests try the obvious ways across the API.
"""

from __future__ import annotations

import json

import pytest

from helpers import assert_problem, fixture_bytes, operation_id

AUTHENTICATED_READS = [
    "/juggle-crews/v1/me",
    "/juggle-routines/v1/me",
    "/juggle-routines/v1/day",
    "/juggle-routines/v1/schedule",
    "/juggle-routines/v1/people",
    "/juggle-shopping/v1/state",
    "/juggle-shopping/v1/ledger",
]


def flat_identifiers(pip) -> dict:
    """Ids that only Flat 4B has."""
    schedule = pip.get("/juggle-routines/v1/schedule").json()
    state = pip.get("/juggle-shopping/v1/state").json()
    me = pip.get("/juggle-crews/v1/me").json()
    return {
        "crew": me["crew"]["id"],
        "invite_code": me["crew"]["invite_code"],
        "people": [person["id"] for person in schedule["people"]],
        "list": [item["id"] for item in state["list"]],
        "catalog": [product["id"] for product in state["catalog"]],
    }


def test_each_account_sees_its_own_crew(pip, nova):
    assert pip.get("/juggle-crews/v1/me").json()["crew"]["name"] == "Flat 4B"
    assert nova.get("/juggle-crews/v1/me").json()["crew"]["name"] == "Robotics Club"


@pytest.mark.parametrize("path", AUTHENTICATED_READS)
def test_reads_show_no_trace_of_the_other_crew(pip, nova, path):
    flat = flat_identifiers(pip)
    seen = json.dumps(nova.get(path).json())
    for kind, values in flat.items():
        for value in values if isinstance(values, list) else [values]:
            assert value not in seen, f"Nova sees Flat 4B {kind} {value} in {path}"
    assert "Mochi" not in seen and "Flat 4B" not in seen


def test_nova_cannot_check_off_a_flat_task(pip, nova):
    today = pip.get("/juggle-routines/v1/me").json()["today"]
    before = pip.get("/juggle-routines/v1/day").json()
    response = nova.put(
        f"/juggle-routines/v1/occurrences/{today}/dishes",
        json={"operation_id": operation_id(), "expected_revision": 1, "completed": True},
    )
    assert response.status_code >= 400
    assert pip.get("/juggle-routines/v1/day").json()["items"] == before["items"]


def test_nova_cannot_read_a_flat_avatar(pip, nova):
    person = flat_identifiers(pip)["people"][0]
    assert pip.get(f"/juggle-routines/v1/people/{person}/avatar").status_code == 200
    assert_problem(nova.get(f"/juggle-routines/v1/people/{person}/avatar"), 404)


def test_nova_cannot_change_the_flat_list(pip, nova):
    before = pip.get("/juggle-shopping/v1/state").json()
    item = before["list"][0]
    response = nova.post(
        "/juggle-shopping/v1/commands",
        json={
            "operation_id": operation_id(),
            "action": {"kind": "mark", "id": item["id"], "revision": item["revision"], "done": True},
        },
    )
    assert_problem(response, 404)
    assert pip.get("/juggle-shopping/v1/state").json()["list"] == before["list"]


def test_receipt_photos_stay_in_their_crew(ike, pip):
    upload = ike.post(
        "/juggle-shopping/v1/receipts",
        data={"operation_id": operation_id()},
        files={"photo": ("receipt.jpg", fixture_bytes("receipt.jpg"), "image/jpeg")},
    )
    assert upload.status_code == 202, upload.text
    receipt = upload.json()["id"]
    assert ike.get(f"/juggle-shopping/v1/receipts/{receipt}/photos").status_code == 200
    assert_problem(pip.get(f"/juggle-shopping/v1/receipts/{receipt}/photos"), 404)


def test_invite_code_is_not_a_door_into_another_crew(pip, nova):
    code = flat_identifiers(pip)["invite_code"]
    response = nova.post("/juggle-crews/v1/memberships", json={"invite_code": code, "display_name": "Nova"})
    assert_problem(response, 409)
    assert nova.get("/juggle-crews/v1/me").json()["crew"]["name"] == "Robotics Club"


@pytest.mark.parametrize("path", AUTHENTICATED_READS)
def test_anonymous_requests_are_refused(anonymous, path):
    assert_problem(anonymous.get(path), 401)


def test_forged_demo_token_is_refused(anonymous):
    response = anonymous.get(
        "/juggle-crews/v1/me", headers={"authorization": "Bearer demo.pip.00000000"}
    )
    assert_problem(response, 401)


def test_members_cannot_reach_the_platform_services(nova):
    """oagw and credstore run in this server, for the server itself only."""
    proxy = nova.post("/oagw/v1/proxy/api.openai.com/v1/responses", json={})
    assert proxy.status_code in (403, 404), proxy.text
    secret = nova.get("/credstore/v1/secrets/openai-api-key")
    assert secret.status_code in (403, 404), secret.text
    assert "sk-" not in secret.text
