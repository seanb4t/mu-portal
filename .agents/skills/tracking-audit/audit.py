#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Read-only audit of the tracking state of mu-portal.

It compares the GitHub Project, the issues, the milestones, the recent pull
requests and the OpenSpec change folders, and prints one line for each gap.
It changes nothing. Exit status 1 means that it found a gap.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

OWNER = os.environ.get("MU_PROJECT_OWNER", "seanb4t")
PROJECT = os.environ.get("MU_PROJECT_NUMBER", "1")
REPO = os.environ.get("MU_REPO", "seanb4t/mu-portal")
ACTIVE = {"In progress", "In review"}
NO_ISSUE_PR = re.compile(r"^(chore|fix)\((openspec|deps)\)")
RENOVATE = {"app/renovate", "app/fzymgc-renovate", "renovate", "renovate[bot]"}
ISSUE_REF = re.compile(r"#(\d+)")
CLOSES_NONE = re.compile(r"^[\s>*-]*Closes:\s*none\b", re.IGNORECASE | re.MULTILINE)
PART_OF = re.compile(
    r"^[\s>*-]*(?:Part of|Refs)\s+#(\d+)", re.IGNORECASE | re.MULTILINE
)


def gh(*args: str) -> object:
    out = subprocess.run(["gh", *args], check=True, capture_output=True, text=True)
    return json.loads(out.stdout)


PLANS = """
query($owner: String!, $name: String!) {
  repository(owner: $owner, name: $name) {
    issues(first: 20, labels: ["kind:plan"]) {
      nodes {
        number
        milestone { title }
        subIssues(first: 100) {
          nodes { number state blockedBy(first: 50) { nodes { state } } }
        }
      }
    }
  }
}
"""


def plans() -> list[dict]:
    """Each plan issue with its milestone and its sub-issues in order."""
    owner, name = REPO.split("/")
    data = gh(
        "api",
        "graphql",
        "-f",
        f"query={PLANS}",
        "-F",
        f"owner={owner}",
        "-F",
        f"name={name}",
    )
    return [
        {
            "number": plan["number"],
            "milestone": (plan["milestone"] or {}).get("title"),
            "subs": [
                {
                    "number": sub["number"],
                    "open": sub["state"] == "OPEN",
                    "blocked": any(
                        b["state"] == "OPEN" for b in sub["blockedBy"]["nodes"]
                    ),
                }
                for sub in plan["subIssues"]["nodes"]
            ],
        }
        for plan in data["data"]["repository"]["issues"]["nodes"]
    ]


def plan_gaps(
    issues: list[dict], status: dict[int, str], plans: list[dict]
) -> list[str]:
    """Gaps between the changes of a phase and the sub-issues of its plan.

    Each change with the milestone of a plan is a sub-issue of that plan. A change with
    the status Next is the first open sub-issue that no open issue blocks.
    """
    gaps: list[str] = []
    by_milestone = {plan["milestone"]: plan for plan in plans if plan["milestone"]}
    for issue in issues:
        labels = {label["name"] for label in issue["labels"]}
        milestone = (issue["milestone"] or {}).get("title")
        plan = by_milestone.get(milestone)
        if "kind:change" not in labels or plan is None:
            continue
        n = issue["number"]
        subs = [sub["number"] for sub in plan["subs"]]
        if n not in subs:
            gaps.append(f"change-not-in-plan #{n} is no sub-issue of #{plan['number']}")
            continue
        if status.get(n) == "Next":
            first = next(
                (s["number"] for s in plan["subs"] if s["open"] and not s["blocked"]),
                None,
            )
            if n != first:
                where = f"#{first}" if first else "none"
                gaps.append(
                    f"next-out-of-order #{n} (the first open, unblocked: {where})"
                )
    return gaps


def first_issue(proposal: Path) -> int | None:
    match = ISSUE_REF.search(proposal.read_text())
    return int(match[1]) if match else None


def audit(root: Path) -> list[str]:
    issues = gh(
        "issue",
        "list",
        "-R",
        REPO,
        "--state",
        "all",
        "--limit",
        "1000",
        "--json",
        "number,state,title,labels,milestone,body",
    )
    items = gh(
        "project",
        "item-list",
        PROJECT,
        "--owner",
        OWNER,
        "--limit",
        "1000",
        "--format",
        "json",
    )["items"]
    prs = gh(
        "pr",
        "list",
        "-R",
        REPO,
        "--state",
        "merged",
        "--limit",
        "30",
        "--json",
        "number,title,author,body,closingIssuesReferences",
    )

    status = {
        i["content"]["number"]: i.get("status") or "none"
        for i in items
        if i["content"].get("type") == "Issue"
    }
    by_number = {i["number"]: i for i in issues}
    gaps: list[str] = []

    for issue in issues:
        n, title, is_open = issue["number"], issue["title"], issue["state"] == "OPEN"
        labels = {label["name"] for label in issue["labels"]}
        st = status.get(n)
        if st is None:
            if is_open:
                gaps.append(f"not-on-board #{n} {title}")
        elif not is_open and st != "Done":
            gaps.append(f"closed-not-done #{n} ({st}) {title}")
        elif is_open and st == "Done":
            gaps.append(f"open-but-done #{n} {title}")
        elif st == "none":
            gaps.append(f"no-status #{n} {title}")
        if is_open and not labels:
            gaps.append(f"no-label #{n} {title}")
        if not is_open and "kind:change" in labels and issue["milestone"] is None:
            gaps.append(f"change-no-milestone #{n} {title}")

    bodies: dict[str, list[int]] = defaultdict(list)
    for issue in issues:
        if issue["state"] == "OPEN" and len(issue["body"]) > 40:
            bodies[issue["body"]].append(issue["number"])
    for numbers in bodies.values():
        if len(numbers) > 1:
            gaps.append("duplicate-body " + " ".join(f"#{n}" for n in sorted(numbers)))

    for pr in prs:
        if pr["author"]["login"] in RENOVATE or NO_ISSUE_PR.match(pr["title"]):
            continue
        if pr["closingIssuesReferences"]:
            continue
        links = [int(n) for n in PART_OF.findall(pr["body"])]
        unknown = [n for n in links if n not in by_number]
        if unknown:
            refs = " ".join(f"#{n}" for n in unknown)
            gaps.append(f"pr-links-no-issue #{pr['number']} names {refs}")
        elif not links and not CLOSES_NONE.search(pr["body"]):
            gaps.append(f"pr-closes-no-issue #{pr['number']} {pr['title']}")

    gaps.extend(plan_gaps(issues, status, plans()))

    in_progress = sum(1 for st in status.values() if st == "In progress")
    if in_progress > 2:
        gaps.append(
            f"too-many-in-progress {in_progress} items (rule: one change and one spike)"
        )

    changes = root / "openspec" / "changes"
    for folder in sorted(p for p in changes.iterdir() if p.is_dir()):
        if folder.name == "archive":
            continue
        proposal = folder / "proposal.md"
        if not proposal.is_file():
            gaps.append(f"change-without-proposal openspec/changes/{folder.name}")
            continue
        n = first_issue(proposal)
        if n is None:
            gaps.append(f"change-names-no-issue {folder.name}")
        elif status.get(n) not in ACTIVE:
            where = status.get(n, "not on the board")
            gaps.append(f"change-issue-status {folder.name} #{n} is {where}")

    for proposal in sorted((changes / "archive").glob("*/proposal.md")):
        n = first_issue(proposal)
        if n is not None and by_number.get(n, {}).get("state") == "OPEN":
            gaps.append(f"archived-change-issue-open {proposal.parent.name} #{n}")

    return gaps


def main() -> int:
    root = Path(
        subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
    )
    gaps = audit(root)
    print("\n".join(gaps) if gaps else "no gaps")
    return 1 if gaps else 0


if __name__ == "__main__":
    sys.exit(main())
