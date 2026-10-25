# Issue tracker: GitHub

Issues and specs for this repo live as GitHub issues. Use the `gh` CLI for all
operations.

Repository: `Aylmz0/folisift` (inferred from `git remote` when run inside the
clone).

## Conventions

- **Create an issue**: `gh issue create --title "..." --body "..."`. Use a
  heredoc for multi-line bodies.
- **Read an issue**: `gh issue view <number> --comments`, filtering comments
  with `jq` and fetching labels.
- **List issues**: `gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'`
  with appropriate `--label` and `--state` filters.
- **Comment**: `gh issue comment <number> --body "..."`
- **Labels**: `gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **Close**: `gh issue close <number> --comment "..."`

## Pull requests as a triage surface

**PRs as a request surface: no.** This is a private, maintainer-driven project.
If external PRs later become a feature-request surface, set this to `yes` and
use the `gh pr` equivalents.

## When a skill says “publish to the issue tracker”

Create a GitHub issue.

## When a skill says “fetch the relevant ticket”

Run `gh issue view <number> --comments`.

## Wayfinding operations

Used by `/wayfinder`. The map is a single issue labelled `wayfinder:map`, with
child issues as tickets.

- **Map**: `gh issue create --label wayfinder:map`.
- **Child ticket**: link as a GitHub sub-issue; fallback: task list in the map
  + `Part of #<map>` at the top of the child body.
- **Blocking**: GitHub native issue dependencies where available; otherwise a
  `Blocked by: #<n>` line.
- **Frontier**: open children, no open blocker, no assignee; first in map order.
- **Claim**: `gh issue edit <n> --add-assignee @me`.
- **Resolve**: comment the answer, close, append context to the map.
