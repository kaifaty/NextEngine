---
name: investigate-with-hypotheses
description: "Investigate uncertain causes with falsifiable competing hypotheses, observable predictions, minimal discriminating experiments and explicit belief updates before fixes or conclusions. Use for non-obvious debugging, flaky or nondeterministic failures, performance attribution, reverse engineering, unfamiliar APIs or protocols, physics or ML experiments, repeated failed attempts, and expensive or hard-to-reverse actions with uncertain consequences. Do not use for routine implementation, obvious compiler errors, mechanical refactors, or well-specified changes with a direct validation path. Russian triggers include: найди причину, почему происходит, сложная отладка, флейки, гипотеза, эксперимент, профилирование, неизвестная механика."
---

# Investigate with hypotheses

Turn unresolved uncertainty into the smallest safe experiment that can change
the next decision. Apply the prediction checkpoint before uncertainty-reducing,
expensive or hard-to-reverse actions, not before every routine tool call.

## Establish the evidence boundary

1. State the exact observation without embedding a cause. Name the first known
   failing boundary, reproduction and authoritative evidence when available.
2. Reproduce through production inputs before changing behavior. If the symptom
   cannot be reproduced, investigate the reproduction boundary or collect
   evidence; do not patch a guessed cause.
3. Name both the success oracle and a non-regression oracle. A disappearing
   symptom, passing compilation or one green test does not by itself prove the
   root cause or preserve intended behavior.
4. Separate observed facts, external claims, assumptions and unknowns. Validate
   volatile claims before using them as premises.

## Build falsifiable hypotheses

- Keep two to four serious competing hypotheses when evidence permits. Use one
  only when the evidence has already excluded meaningful alternatives.
- For each hypothesis, state the causal claim, an observable prediction if it is
  true, an observation that would falsify it and any assumption the experiment
  cannot test.
- Include an adjacent-layer or non-local explanation after repeated variants
  only move the symptom.
- Route every explicit unknown to exactly one state: `TEST`, `IRRELEVANT` with a
  reason, or `UNRESOLVED` with its consequence. Never silently continue past it.

Use a compact working table when more than one hypothesis remains:

| ID | Causal hypothesis | Prediction if true | Falsifier | Current evidence |
| --- | --- | --- | --- | --- |
| H1 | <cause> | <observable result> | <contradicting result> | <pointer> |

## Choose the discriminator

1. Prefer the cheapest safe and reversible experiment that produces different
   expected outcomes across the live hypotheses. Treat "information gain" as
   hypothesis separation, not as a reason for unnecessary measurement.
2. Change one causal variable at a time. Add a successful control or a small
   counterfactual when it distinguishes infrastructure failure from the target
   mechanism.
3. Predeclare the action, expected split, stopping condition, evidence to retain
   and rollback or cleanup. Inspect exact existing evidence before adding probes.
4. Account for observer effects: logging can hide races, profiling changes timing,
   caches contaminate comparisons and retries can erase deterministic failures.
5. Before an expensive or hard-to-reverse action, predict its observable effects
   and recovery boundary. Obtain user direction when the required mutation or
   authority is outside the current request.

## Execute, compare and update

Run only the selected discriminator, then record:

```text
EXPERIMENT: <one bounded action>
EXPECTED: H1 -> <result>; H2 -> <different result>
OBSERVED: <exact result and evidence pointer>
UPDATE: <supported, falsified, revised or inconclusive>
NEXT: <smallest changed action, or stop>
```

- `SUPPORTED` means consistent with the evidence, not proven true.
- `FALSIFIED` retires or narrows the hypothesis before further implementation.
- `INCONCLUSIVE` must name why the experiment failed to discriminate.
- A contradiction with every prediction widens the model before another similar
  attempt; do not relabel surprise as execution noise without evidence.
- Never repeat a failed experiment unless the hypothesis, inputs or conditions
  changed in a way that predicts a different result.

## Gate the fix or conclusion

Proceed only when the selected explanation accounts for the causal order and the
relevant evidence better than its alternatives. Make the smallest production-path
change, rerun the original reproduction and non-regression oracle, and remove or
bound temporary instrumentation. Distinguish `ROOT_CAUSE_FIXED`,
`SYMPTOM_SUPPRESSED`, `REPORT_ONLY` and `UNRESOLVED`; report remaining uncertainty
and the condition that would reopen the conclusion.

## Preserve only durable transitions

For long-running, resumed, handed-off or approach-changing work, use
`$maintain-task-context`. Promote only material observations, evidence-backed
decisions, rejected approaches, remaining uncertainty and the next discriminator;
do not turn task-state into a per-probe diary or copy raw logs into it. When the
repository persistent-problem rule applies, also follow its bounded research and
source-quality requirements.
