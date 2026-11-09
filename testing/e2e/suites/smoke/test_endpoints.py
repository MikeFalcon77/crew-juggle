"""One happy path per REST endpoint, all inside the Robotics Club.

Nova owns the club and Ike is a member. Flat 4B is left alone so the golden
snapshots stay valid.
"""

from __future__ import annotations

from helpers import assert_problem, fixture_bytes, operation_id


def club_today(client) -> str:
    return client.get("/juggle-routines/v1/me").json()["today"]


# ---- crews ----


def test_me_and_sign_in_config(anonymous, ike):
    config = anonymous.get("/juggle-crews/v1/config")
    assert config.status_code == 200 and config.json()["sign_in"] == "demo"
    me = ike.get("/juggle-crews/v1/me")
    assert me.status_code == 200
    assert me.headers["cache-control"] == "no-store"
    assert me.json()["crew"]["role"] == "member"


def test_create_crew_while_in_one_is_refused(ike):
    response = ike.post(
        "/juggle-crews/v1/crews",
        json={"name": "Another", "display_name": "Ike", "timezone": "Europe/Berlin"},
    )
    assert_problem(response, 409)


def test_owner_rotates_the_invite_code_and_members_cannot(nova, ike):
    before = nova.get("/juggle-crews/v1/me").json()["crew"]["invite_code"]
    rotated = nova.post("/juggle-crews/v1/crew/invite-code")
    assert rotated.status_code == 200, rotated.text
    assert rotated.json()["crew"]["invite_code"] != before
    assert_problem(ike.post("/juggle-crews/v1/crew/invite-code"), 403)


def test_owner_updates_the_crew(nova):
    crew = nova.get("/juggle-crews/v1/me").json()["crew"]
    response = nova.put(
        "/juggle-crews/v1/crew", json={"name": crew["name"], "timezone": crew["timezone"]}
    )
    assert response.status_code == 200, response.text
    assert response.json()["crew"]["name"] == "Robotics Club"


# ---- routines ----


def test_routines_reads(ike):
    for path in (
        "/juggle-routines/v1/me",
        "/juggle-routines/v1/me/preferences",
        "/juggle-routines/v1/day",
        "/juggle-routines/v1/schedule",
        "/juggle-routines/v1/people",
    ):
        response = ike.get(path)
        assert response.status_code == 200, f"{path}: {response.text}"
        assert response.headers["content-type"].startswith("application/json")


def test_preferences_round_trip(ike):
    preferences = ike.get("/juggle-routines/v1/me/preferences").json()["preferences"]
    response = ike.put(
        "/juggle-routines/v1/me/preferences",
        json={
            "operation_id": operation_id(),
            "expected_revision": preferences["revision"],
            "timezone": preferences["timezone"],
        },
    )
    assert response.status_code == 200, response.text


def test_check_off_and_undo(ike):
    day = ike.get("/juggle-routines/v1/day").json()
    item = next(i for i in day["items"] if i["status"] != "done")
    date, routine = item["id"].split("/", 1)
    done = ike.put(
        f"/juggle-routines/v1/occurrences/{date}/{routine}",
        json={"operation_id": operation_id(), "expected_revision": item["revision"], "completed": True},
    )
    assert done.status_code == 200, done.text
    after = next(i for i in ike.get("/juggle-routines/v1/day").json()["items"] if i["id"] == item["id"])
    assert after["status"] == "done"
    undo = ike.put(
        f"/juggle-routines/v1/occurrences/{date}/{routine}",
        json={"operation_id": operation_id(), "expected_revision": after["revision"], "completed": False},
    )
    assert undo.status_code == 200, undo.text


def test_routine_mood(nova):
    schedule = nova.get("/juggle-routines/v1/schedule").json()
    routine, appearance = next(iter(schedule["appearances"].items()))
    response = nova.put(
        f"/juggle-routines/v1/schedule/{routine}/appearance",
        json={"operation_id": operation_id(), "expected_revision": appearance["revision"], "mood": "happy"},
    )
    assert response.status_code == 200, response.text


def test_avatar_upload_read_and_remove(ike):
    people = ike.get("/juggle-routines/v1/people").json()
    person = next(p for p in people["people"] if p["account"] == "demo-ike")
    meta = people["avatars"].get(person["id"], {"revision": 0})
    upload = ike.put(
        f"/juggle-routines/v1/people/{person['id']}/avatar",
        data={"operation_id": operation_id(), "expected_revision": str(meta["revision"])},
        files={"file": ("avatar.png", fixture_bytes("avatar.png"), "image/png")},
    )
    assert upload.status_code == 200, upload.text
    image = ike.get(f"/juggle-routines/v1/people/{person['id']}/avatar")
    assert image.status_code == 200 and image.headers["content-type"].startswith("image/")
    removed = ike.request(
        "DELETE",
        f"/juggle-routines/v1/people/{person['id']}/avatar",
        json={"operation_id": operation_id(), "expected_revision": upload.json()["revision"]},
    )
    assert removed.status_code == 200, removed.text


def test_push_subscription_endpoints_answer(ike):
    subscribe = ike.post(
        "/juggle-routines/v1/push/subscriptions",
        json={"endpoint": "https://push.example.invalid/e2e", "keys": {"p256dh": "x", "auth": "y"}},
    )
    assert subscribe.status_code < 500, subscribe.text
    if subscribe.status_code == 200:
        subscription = subscribe.json()["id"]
        assert ike.delete(f"/juggle-routines/v1/push/subscriptions/{subscription}").status_code == 200


# ---- shopping ----


def shopping_command(client, action: dict):
    return client.post("/juggle-shopping/v1/commands", json={"operation_id": operation_id(), "action": action})


def test_list_add_mark_remove(ike):
    item = operation_id()
    added = shopping_command(ike, {"kind": "add", "items": [{"id": item, "name": "e2e solder", "quantity": "1"}]})
    assert added.status_code == 200, added.text
    state = ike.get("/juggle-shopping/v1/state").json()
    entry = next(i for i in state["list"] if i["id"] == item)
    marked = shopping_command(ike, {"kind": "mark", "id": item, "revision": entry["revision"], "done": True})
    assert marked.status_code == 200, marked.text
    entry = next(i for i in ike.get("/juggle-shopping/v1/state").json()["list"] if i["id"] == item)
    removed = shopping_command(ike, {"kind": "remove", "id": item, "revision": entry["revision"]})
    assert removed.status_code == 200, removed.text


def test_same_command_twice_counts_once(ike):
    name = f"e2e once {operation_id()[:8]}"
    body = {
        "operation_id": operation_id(),
        "action": {"kind": "add", "items": [{"id": operation_id(), "name": name, "quantity": "2"}]},
    }
    assert ike.post("/juggle-shopping/v1/commands", json=body).status_code == 200
    assert ike.post("/juggle-shopping/v1/commands", json=body).status_code == 200
    names = [i["name"] for i in ike.get("/juggle-shopping/v1/state").json()["list"]]
    assert names.count(name) == 1


def test_receipt_upload_photos_and_ledger(ike):
    upload = ike.post(
        "/juggle-shopping/v1/receipts",
        data={"operation_id": operation_id()},
        files={"photo": ("receipt.jpg", fixture_bytes("receipt.jpg"), "image/jpeg")},
    )
    assert upload.status_code == 202, upload.text
    photos = ike.get(f"/juggle-shopping/v1/receipts/{upload.json()['id']}/photos")
    assert photos.status_code == 200
    assert photos.json()["photos"][0].startswith("data:image/jpeg;base64,")
    ledger = ike.get("/juggle-shopping/v1/ledger")
    assert ledger.status_code == 200 and "events" in ledger.json()


# The receipt as the web app reads it: scanner bookkeeping (digest, lease,
# attempts, eligible items) never leaves the server.
RECEIPT_KEYS = {
    "id", "merchant", "date", "dateMissing", "currency", "total", "items", "status",
    "revision", "createdAt", "photoExpiresAt", "matchedIds", "issues", "time",
    "registeredBy", "registeredAccount", "suspectedDuplicate",
}


def test_receipts_in_the_state_keep_their_shape(ike):
    upload = ike.post(
        "/juggle-shopping/v1/receipts",
        data={"operation_id": operation_id()},
        files={"photo": ("receipt.jpg", fixture_bytes("receipt.jpg"), "image/jpeg")},
    )
    assert upload.status_code == 202, upload.text
    state = ike.get("/juggle-shopping/v1/state").json()
    receipt = next(r for r in state["receipts"] if r["id"] == upload.json()["id"])
    assert set(receipt) == RECEIPT_KEYS
    assert receipt["registeredAccount"] == "demo-ike"
    assert set(state) == {
        "revision", "list", "catalog", "rules", "receipts", "imports", "purchases",
        "aiConfigured", "today",
    }


# ---- probes and API docs (the web app itself is served by the web image) ----


def test_probes_and_api_docs(anonymous):
    assert anonymous.get("/readyz").status_code == 200
    assert anonymous.get("/healthz").status_code == 200
    assert anonymous.get("/openapi.json").status_code == 200
