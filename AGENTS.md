# Agent instructions

mu-portal is one Rust binary, `mu`, with a server and terminal clients for MUSH games. The rules of the project are in the Contributing pages, `docs/src/content/docs/contributing/`. Read the page before work depends on its rules. This file does not restate them.

## Where things live

- OpenSpec holds the content of the work: `openspec/specs/` and `openspec/changes/`. The file `openspec/config.yaml` holds the context and the rules of each artifact.
- The GitHub Project "mu-portal" holds the status. The page `contributing/openspec.md` states what sets each status, and which labels and milestones an issue gets.
- `mise.toml` holds the check commands. `mise run check` runs every check of the Rust workspace.

## Tracking duties

- Each issue that you open gets a kind or type label. If it serves a phase, it also gets the milestone of that phase.
- Each pull request names its issue with `Closes #<number>`. An archive or a follow-up to merged work writes `Part of #<number>` instead. Only a trivial fix, such as a typo, writes `Closes: none` and the reason.
- After you archive a change, run the project skill `tracking-audit` and repair each gap with the owner's consent.

## Project skills

- `tracking-audit`: audit and repair the board, the labels, the milestones and the OpenSpec folders.
- The `openspec-*` skills come from the OpenSpec CLI. Do not edit them by hand. `openspec update` writes them again.
