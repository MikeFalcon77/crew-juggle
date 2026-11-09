"""The API as the web client sees it, for Flat 4B as the demo seed leaves it.

These snapshots were taken before the gears were restructured; the refactor
must keep them passing without changes.
"""

from __future__ import annotations

import datetime as dt

import pytest

from golden import assert_golden


def daily_only(day: dict) -> dict:
    """Keeps the routines that repeat every day.

    Weekly routines (trash on Tuesdays, litter on Mon/Wed/Fri) appear only on
    their weekdays, which would make the snapshot depend on the day it runs.
    """
    items = [item for item in day["items"] if not item["routine"]["days"]]
    times = {item["routine"]["time"] for item in items}
    return {
        **day,
        "items": items,
        "scheduled_times": {k: v for k, v in day["scheduled_times"].items() if k in times},
    }


PROJECTIONS = {"routines_day_pip": daily_only, "routines_day_zed": daily_only}

READS = [
    ("pip", "crews_me_pip", "/juggle-crews/v1/me"),
    ("zed", "crews_me_zed", "/juggle-crews/v1/me"),
    ("pip", "routines_me_pip", "/juggle-routines/v1/me"),
    ("pip", "routines_preferences_pip", "/juggle-routines/v1/me/preferences"),
    ("pip", "routines_day_pip", "/juggle-routines/v1/day"),
    ("zed", "routines_day_zed", "/juggle-routines/v1/day"),
    ("pip", "routines_schedule_pip", "/juggle-routines/v1/schedule"),
    ("pip", "routines_people_pip", "/juggle-routines/v1/people"),
    ("pip", "shopping_state_pip", "/juggle-shopping/v1/state"),
    ("pip", "shopping_ledger_pip", "/juggle-shopping/v1/ledger"),
]


@pytest.fixture(scope="module")
def today(as_user) -> dt.date:
    return dt.date.fromisoformat(as_user("pip").get("/juggle-routines/v1/me").json()["today"])


@pytest.mark.parametrize("user,name,path", READS, ids=[name for _, name, _ in READS])
def test_reads(as_user, today, user, name, path):
    assert_golden(name, as_user(user).get(path), today, PROJECTIONS.get(name))


def test_history_of_yesterday(pip, today):
    yesterday = (today - dt.timedelta(days=1)).isoformat()
    response = pip.get("/juggle-routines/v1/day", params={"date": yesterday})
    assert_golden("routines_day_yesterday_pip", response, today, daily_only)


def test_sign_in_config(anonymous, today):
    assert_golden("crews_config", anonymous.get("/juggle-crews/v1/config"), today)


def test_demo_session(anonymous, today):
    response = anonymous.post("/juggle-crews/v1/demo/sessions", json={"user": "zed"})
    assert response.json()["token"].startswith("demo.zed.")
    assert_golden("crews_demo_session", response, today)


def test_unknown_demo_character(anonymous, today):
    response = anonymous.post("/juggle-crews/v1/demo/sessions", json={"user": "nobody"})
    assert_golden("error_unknown_character", response, today)


def test_missing_token(anonymous, today):
    assert_golden("error_missing_token", anonymous.get("/juggle-crews/v1/me"), today)


def test_join_while_in_a_crew(pip, today):
    response = pip.post(
        "/juggle-crews/v1/memberships", json={"invite_code": "ZZZZ-ZZZZ", "display_name": "Pip"}
    )
    assert_golden("error_join_while_in_crew", response, today)


def test_stale_check_off(pip, today):
    response = pip.put(
        f"/juggle-routines/v1/occurrences/{today.isoformat()}/dishes",
        json={
            "operation_id": "00000000-0000-4000-8000-000000000001",
            "expected_revision": 999,
            "completed": True,
        },
    )
    assert_golden("error_stale_check_off", response, today)


def test_mark_unknown_list_item(pip, today):
    response = pip.post(
        "/juggle-shopping/v1/commands",
        json={
            "operation_id": "00000000-0000-4000-8000-000000000002",
            "action": {"kind": "mark", "id": "missing", "revision": 1, "done": True},
        },
    )
    assert_golden("error_mark_unknown_item", response, today)
