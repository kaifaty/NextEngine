# <Task name> — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE`, `PAUSED`, `COMPLETE`, `SUPERSEDED`, or a repository-defined bounded status |
| Updated | `YYYY-MM-DD` |
| Task key | `<stable-task-slug>` |
| Scope | `<one bounded outcome>` |
| Definition of done | `<observable completion criterion>` |
| Primary deliverable | `<what the user can inspect, run, hear or see>` |
| Last primary artifact | `<path/result/date, or None>` |
| Supporting-only checkpoints | `<0, 1, or 2 — at 2 the next checkpoint produces the primary artifact or exposes a concrete blocker>` |
| Authority | Working context only; name the sources that outrank this file |

## Resume in 60 seconds

- **Current conclusion:** <what is believed now>
- **Why:** <smallest decisive evidence>
- **Next artifact:** <smallest end-to-end user-observable result>
- **Current blocker:** <blocker or `None`>
- **Do not retry:** <approach and exact reason>
- **Reconsider when:** <new evidence that would reopen the decision>

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `<path, artifact ID or hash>` | `PASS`, `FAIL`, `REPORT_ONLY`, `NOT_RUN`, or a typed observation | <bounded meaning; no broader claim> |

## Decisions that still constrain the work

| Decision | Smallest decisive evidence | Reconsider when |
| --- | --- | --- |
| <what future work must do> | <path, result or exact failing boundary> | <falsifiable condition> |

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1 | <evidence> | <evidence> | <small counterfactual or source check> |

## Required context

Read only the sources needed for the immediate action, at most seven:

1. `<routing or normative authority>`
2. `<roadmap or tracked profile/manifest>`
3. `<implementation plan or domain contract>`
4. `<bounded evidence/research report>`

Historical and superseded links belong in Git or an explicitly non-required
index; they are not recursive resume input.

## Next action

1. <smallest evidence-backed action>
2. <success criterion>
3. <rollback or non-regression check>

## Do not retry

- <approach> — <decisive reason>; reconsider only when <condition>.

## Handoff

- **Workspace state:** <relevant tracked/untracked changes without touching unrelated user work>
- **Checks:** <passed, failed and not run>
- **Remaining risk:** <known uncertainty or unavailable evidence>
- **Promotion needed:** <ADR/SPEC, roadmap, AGENTS, skill, research report, or `None`>
