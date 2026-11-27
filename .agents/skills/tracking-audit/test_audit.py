"""Tests of the plan checks of audit.py. Run: python3 -m unittest discover .agents/skills/tracking-audit"""

import unittest

from audit import plan_gaps

PHASE = {"title": "Phase 1"}


def change(n: int, milestone: dict | None = PHASE) -> dict:
    return {"number": n, "labels": [{"name": "kind:change"}], "milestone": milestone}


def plan(*subs: tuple[int, bool, bool]) -> dict:
    return {
        "number": 61,
        "milestone": "Phase 1",
        "subs": [{"number": n, "open": o, "blocked": b} for n, o, b in subs],
    }


class PlanGaps(unittest.TestCase):
    def test_a_change_in_the_plan_is_no_gap(self) -> None:
        self.assertEqual(plan_gaps([change(1)], {}, [plan((1, True, False))]), [])

    def test_a_change_outside_the_plan(self) -> None:
        self.assertEqual(
            plan_gaps([change(2)], {}, [plan((1, True, False))]),
            ["change-not-in-plan #2 is no sub-issue of #61"],
        )

    def test_a_change_of_another_milestone_is_no_gap(self) -> None:
        other = change(2, {"title": "Phase 2"})
        self.assertEqual(plan_gaps([other, change(3, None)], {}, [plan()]), [])

    def test_an_issue_that_is_no_change_is_no_gap(self) -> None:
        bug = {"number": 2, "labels": [{"name": "bug"}], "milestone": PHASE}
        self.assertEqual(plan_gaps([bug], {}, [plan()]), [])

    def test_next_is_the_first_open_unblocked_sub_issue(self) -> None:
        subs = plan(
            (1, False, False), (2, True, True), (3, True, False), (4, True, False)
        )
        changes = [change(n) for n in (1, 2, 3, 4)]
        self.assertEqual(plan_gaps(changes, {3: "Next"}, [subs]), [])
        self.assertEqual(
            plan_gaps(changes, {4: "Next"}, [subs]),
            ["next-out-of-order #4 (the first open, unblocked: #3)"],
        )

    def test_a_blocked_change_is_not_next(self) -> None:
        self.assertEqual(
            plan_gaps([change(2)], {2: "Next"}, [plan((2, True, True))]),
            ["next-out-of-order #2 (the first open, unblocked: none)"],
        )


if __name__ == "__main__":
    unittest.main()
