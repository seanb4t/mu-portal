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
NO_ISSUE_PR = re.compile(r"^chore\((openspec|deps)\)")
RENOVATE = {"app/renovate", "renovate", "renovate[bot]"}
ISSUE_REF = re.compile(r"#(\d+)")


def gh(*args: str) -> object:
    out = subprocess.run(["gh", *args], check=True, capture_output=True, text=True)
    return json.loads(out.stdout)


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
        "number,title,author,closingIssuesReferences",
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
        if not pr["closingIssuesReferences"]:
            gaps.append(f"pr-closes-no-issue #{pr['number']} {pr['title']}")

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
