---
name: tracking-audit
description: Audit and repair the tracking state of mu-portal, so that it agrees with the code. The state is the Project board, the labels, the milestones, the phase plan and the OpenSpec change folders. Use it after an archive, at the end of a phase, and before you choose the next item. Also use it for a request to triage, sync or clean up the issues, the board or the changes.
---

# Tracking audit

The rules are on the page `docs/src/content/docs/contributing/openspec.md`. This skill finds where the tracking state breaks them and repairs it with your consent.

## 1. Run the audit

```bash
.agents/skills/tracking-audit/audit.py
```

The script runs with `uv run --script` and needs `gh`. It is read-only. It prints one line for each gap, or `no gaps`. Exit status 1 means that it found a gap.

| Gap | Repair |
|---|---|
| `not-on-board` | `gh project item-add 1 --owner seanb4t --url <issue url>`. The Project workflow sets Parked |
| `closed-not-done`, `open-but-done`, `no-status` | Set the Status field with `gh project item-edit` |
| `no-label` | Add one kind or type label. The page lists them |
| `change-no-milestone` | Set the milestone of the phase that the change served |
| `duplicate-body` | Close the newer issue as a duplicate of the older one, and add the `duplicate` label |
| `pr-closes-no-issue` (a body with `Part of #n`, `Refs #n` or `Closes: none` passes) | If the issue of that work is still open, close it by hand. Otherwise edit the body to name the issue it belongs to. Older pull requests are history only |
| `pr-links-no-issue` | The body names a number that is no issue, such as a pull request. Edit the body |
| `too-many-in-progress` | Ask the owner which item goes back to Next |
| `change-without-proposal` | An empty folder in `openspec/changes/`. Look at it, then delete it |
| `change-names-no-issue`, `change-issue-status` | Name the issue in the proposal, or set its status to In progress |
| `archived-change-issue-open` | If the change is merged, close the issue |

## 2. Check what the script cannot check

- The plan issue of the phase (label `kind:plan`) lists each change of the phase in order. Each archived change shows Done in its row.
- Each `kind:followup` issue either has a parent issue or is Parked. At the end of a phase, group the follow-up issues into themed parent issues with sub-issues, so that no review row is lost.
- The next item on the board (Status Next) is the item that the plan names next.

## 3. Repair

Show the gaps and the planned repairs to the owner first. Ask once for all of them. Close an issue only with a comment that gives the evidence: a commit, a file and line, or the duplicate issue. Run the script again afterward and report its output.
