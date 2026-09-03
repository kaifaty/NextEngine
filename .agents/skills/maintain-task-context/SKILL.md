---
name: maintain-task-context
description: Resume or hand off NextEngine work whose essential state must survive context loss, or record a durable constraint or approach change that would otherwise be repeated. Do not use merely because work is long-running, for routine status, after every failed experiment, or to create research/roadmap ceremony.
---

# Maintain durable task context

Keep one cheap resume surface containing only facts needed for the next useful
action. This skill preserves continuity; it must not become a parallel research
process, an evidence warehouse or a reason to delay the primary deliverable.

## Activation boundary

Use this skill when work is being resumed after context loss, paused or handed
off, or when a new durable constraint changes what the next agent must do. An
individual run failure is not automatically a durable transition. Ordinary
implementation, a status answer and a sequence of experiments in one active
turn do not require task-state edits.

## Resume cheaply

1. Read `AGENTS.md`. Use architecture routing only if the immediate action
   changes a governed contract, authority boundary or product-level roadmap
   fact; a bounded lab experiment does not trigger full architecture retrieval.
2. Locate the task with `rg --files docs/development/task-state` and a focused
   `rg` query. Do not infer identity from a vaguely similar filename.
3. Read `Resume in 60 seconds` first. Read the rest only when the immediate
   action needs a particular decision or evidence item.
4. Select at most seven active `Required context` pointers relevant to that
   action. Do not recursively chase their references and do not load superseded
   reports merely because they appear in a historical index.
5. Validate only volatile claims that affect the immediate action. Task-state
   remains working material and never overrides Accepted SPEC/ADR, the roadmap,
   tracked profiles/manifests or exact external evidence.

If qualifying work has no task-state, create one only when it will actually
cross a pause, handoff or context boundary and forgetting it would cause costly
or unsafe repetition. Use `docs/development/task-state/<stable-task-slug>.md`
from [the template](references/task-state-template.md). Git preserves history.

## Protect the primary outcome

The resume section must name:

- the task's primary user-observable deliverable;
- the last primary artifact the user could inspect, run, hear or see;
- how many consecutive completed checkpoints produced only supporting work.

Plans, protocols, validators, manifests, hashes and task-state edits are
supporting work unless the user explicitly requested one as the primary
deliverable. They do not reset outcome debt. After two consecutive supporting-
only checkpoints, stop creating more support infrastructure and explicitly
report the missing outcome. The next checkpoint must be the smallest end-to-end
primary artifact or a concrete blocker requiring user authority. A failed or
unadmitted experimental artifact may still be shown with an honest label.

## Update only when future action changes

Update task-state when evidence changes the approach, a failed path is likely
to be repeated, a durable constraint changes the next action or the work pauses
or hands off. Consolidate related failures into one decision; do not record
every gate or run as a transition.

Do not update for routine progress, every command, speculation or unchanged
status. Do not record private chain-of-thought, raw logs, secrets, credentials,
heavy artifacts, datasets, checkpoints or generated payloads. Record the
reviewable result instead:

`observation -> decisive evidence -> decision -> next artifact`

For a material decision, retain only the immediate consequence, a rejected
alternative when repetition is plausible, the reopening condition and the
smallest artifact-producing next action.

## Keep the resume surface bounded

- Put a `Resume in 60 seconds` section first and keep it independently useful.
- Maintain a current summary, not an append-only diary; Git is the history.
- Target at most 150 lines, seven active context pointers and ten active
  decisions. A historical link index may exist but is not resume input.
- Prefer a machine-produced experiment report over new prose. Create a dated
  research report only when its conclusion will be reused outside this task.
- Point to raw or external evidence; do not duplicate it.
- Preserve negative results only while their reconsideration condition matters.
- Do not create a new roadmap version because one experiment failed.

## Promote sparingly

Promote a fact only when another task truly needs it as authority. Product-level
stage or order changes belong in `docs/roadmap.md`; architecture semantics use
the ADR/SPEC workflow; reusable repository rules belong in `AGENTS.md` or a
skill. Routine experimental status stays in its machine report or compact
task-state. Do not promote a run result merely to satisfy documentation shape.

## Handoff

1. Reconcile the primary artifact, relevant diff and smallest decisive evidence.
2. State the next artifact, blocker, `Do not retry` and reopening condition.
3. Update normative or roadmap authority only if its semantics actually changed.
4. Commit context with the code/result checkpoint; avoid a context-only commit
   unless the task is pausing without another artifact.
