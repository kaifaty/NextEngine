---
name: maintain-task-context
description: Maintain bounded, durable task-state for long-running, resumed, handed-off, research-heavy, or approach-changing NextEngine work. Use when starting or resuming such a task, pausing or handing it off, when evidence invalidates the current approach, when a negative result must not be repeated, or when a new constraint changes the next action. Do not use for routine progress logging or as a substitute for normative architecture, roadmap authority, or exact evidence artifacts.
---

# Maintain durable task context

Keep one compact, reviewable resume surface for work whose important context
must survive chats, agents and compaction. Store task facts in the repository;
use this skill as the procedure for finding, reading and updating them.

## Start or resume work

1. Read `AGENTS.md` and route architecture- or roadmap-sensitive work through
   `docs/architecture/agent-routing.md` before relying on a task-state file.
2. Locate the task with `rg --files docs/development/task-state` and a focused
   `rg` query over task names, IDs and stage names. Do not infer identity from a
   vaguely similar filename.
3. Read the matching task-state file in full before large plans, logs or raw
   experiment output.
4. Read every item under its `Required context` section. Apply repository
   precedence: task-state is working material and never overrides Accepted
   SPEC/ADR, the roadmap for planning facts, tracked profiles/manifests, or
   exact external evidence.
5. Validate volatile claims against the current workspace and named evidence
   before acting. Mark stale or missing pointers explicitly.

If qualifying work has no task-state, create
`docs/development/task-state/<stable-task-slug>.md` from
[the template](references/task-state-template.md). Use a stable path without a
date; Git history preserves earlier snapshots.

## Update only at material transitions

Update task-state when at least one of these occurs:

- evidence falsifies the current approach or changes the primary hypothesis;
- a failed approach is likely to be repeated without a durable warning;
- a new constraint, prerequisite or authority changes the next action;
- the allowed claim, gate status, scope or rollback boundary changes;
- work pauses, hands off, resumes after stale context, or reaches a coherent
  checkpoint.

Do not update it for routine progress, every command, speculative ideas without
evidence, or unchanged status. Do not record private chain-of-thought, raw logs,
secrets, credentials, heavy artifacts, datasets, checkpoints or generated run
payloads. Record the reviewable engineering result instead:

`observation -> evidence -> conclusion -> decision -> future instruction`

## Record a decision

For every material change of direction, record:

- the exact observation and evidence pointer, including IDs, hashes or first
  failing boundary when available;
- the decision and its immediate consequence;
- rejected alternatives and the concrete reason not to retry them;
- remaining uncertainty;
- the condition that would justify reconsideration;
- the smallest next action and its rollback or non-regression check.

Keep competing hypotheses falsifiable. When the repository persistent-problem
rule applies, include evidence for and against each serious hypothesis and the
decision criterion for resuming implementation.

## Keep the resume surface bounded

- Put a `Resume in 60 seconds` section first and keep it independently useful.
- Maintain a current summary, not an append-only diary. Replace superseded
  wording and retain only decisions that still constrain future work.
- Target at most 250 lines. Move detailed analysis to a dated
  `docs/development/*-research-YYYY-MM-DD.md` report and link it.
- Point to raw or external evidence; do not duplicate it.
- Preserve exact negative results and `do not retry` instructions until their
  reconsideration condition is satisfied.
- Mark completed or superseded state explicitly; do not silently delete the
  final resume surface.

## Promote facts to their real authority

Task-state is never the final home for a durable cross-task rule:

| Material fact | Promote to |
| --- | --- |
| Accepted semantic or cross-context architecture decision | New superseding ADR plus affected SPEC/routing updates |
| Stage, blocker, exit criterion, queue or subsystem-status change | `docs/roadmap.md` |
| Permanent repository-wide working rule | `AGENTS.md` |
| Reusable workflow or domain procedure | Repository skill |
| Stable domain contract shared by many tasks | A referenced contract in the relevant skill |
| Detailed bounded investigation | Dated research or evidence report |

Update the task-state in the same coherent change with links to promoted
sources. Do not let it become parallel authority.

## Handoff

Before handing off qualifying work:

1. Reconcile the state with the actual diff, workspace status and exact
   evidence used in the decision.
2. Ensure `Next action`, `Do not retry`, `Reconsider when` and remaining
   uncertainty are explicit.
3. Update material roadmap or normative facts through their normal workflow.
4. Commit the task-state with the coherent work checkpoint when the repository
   workflow calls for a commit; do not create noisy per-thought commits.
5. Report task-state validation under the normal risk-scoped product checks.
