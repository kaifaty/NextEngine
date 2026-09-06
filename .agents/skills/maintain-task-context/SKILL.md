---
name: maintain-task-context
description: Recover NextEngine task state after context loss, hand work off, or preserve a durable constraint or decision whose loss would cause costly repetition. Do not use merely because work is long-running, for routine status or after every failed experiment.
---

# Maintain durable task context

Apply the [shared execution guidance](../astra-guidance.md) once per task alongside this skill; it governs process defaults in the references too.

Keep one resume surface containing the facts needed for the next useful action.
Repository policy for activation, outcomes, documentation and commits lives in
[AGENTS.md](../../../AGENTS.md); this skill describes how to recover and preserve
task state.

## Resume

1. Locate the task with `rg --files docs/development/task-state` and a focused
   query. Do not infer identity from a vaguely similar filename.
2. Read `Resume in 60 seconds` first. Read the rest only when the immediate
   action needs a particular decision or evidence item.
3. Follow evidence pointers relevant to that action. Do not recursively chase
   their references or load superseded reports merely because they are linked.
4. Validate volatile claims that affect the immediate action. Task-state is
   working context; it does not override governing contracts or exact evidence.

## Preserve what the next agent needs

Update when losing a decision, constraint or unfinished state would change the
next action or cause costly repetition. Merely ending a turn, running an
experiment or answering status does not require an update.

If qualifying work has no task-state, use
`docs/development/task-state/<stable-task-slug>.md` and the optional structure in
[the template](references/task-state-template.md). Omit irrelevant fields;
existing files do not need migration just to match the template.

Record the objective, last inspectable result, constraints, next action and
decisive evidence. If a failed approach is likely to be repeated, include its
failure reason and the evidence that would justify trying it again. Keep each
fact in one place.

- Put `Resume in 60 seconds` first and keep it independently useful, including
  the last primary artifact or an explicit statement that none exists.
- Replace stale conclusions instead of appending a diary; Git is the history.
- Link existing result artifacts instead of copying logs or creating summaries
  of summaries. Add a separate report only when the investigation itself needs
  to be reused.
- At handoff, include relevant uncommitted work or missing verification only
  when it changes how the next agent should proceed.
