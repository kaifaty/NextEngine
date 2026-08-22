# NSR3-B4E2D7R1 inner-floor diagnostic research

Date: `2026-08-22`

Status: `RESEARCHED / CONTRACT_READY / REPLAY_ONLY`

## Problem

B4E2D7R closes the dimensional outer-state mistake but exposes a nested-solve
failure. At outer 9, the fixed inner stationarity threshold returns without a
primal step while PHR still changes the multiplier materially. The following
inner call rejects more than eight trust trials.

Two mechanisms can now coexist:

1. the inner accuracy is not tightened as the outer KKT state converges;
2. the raw subtraction `E(current)-E(trial)` may lose a valid microscopic
   descent below the ULP scale of the complete objective.

Changing either mechanism before observing the failed trial sequence would
repeat the earlier pattern of moving a symptom between gates.

## Method basis

Inexact augmented-Lagrangian methods do not generally use one fixed inner
accuracy forever. Fernández and Solodov describe practical truncated
subproblems with an approximation tolerance tied to the current KKT violation,
rather than a constant unrelated to outer progress
([DOI 10.1137/10081085X](https://doi.org/10.1137/10081085X)). Earlier method-
of-multipliers work likewise shows that inexact minimization can retain the
exact method's convergence rate when its stopping rule is operated accordingly
([DOI 10.1016/B978-0-12-468650-2.50011-0](https://doi.org/10.1016/B978-0-12-468650-2.50011-0)).

This supports a future nested accuracy schedule, but does not prove that
binary64 raw energy subtraction can admit the resulting smaller steps. The
next stage therefore measures both causes and changes neither.

## Exact replay

Reproduce B4E2D7R through its failed state, requiring:

- exact D7 first-eight state and outer-array roots;
- exact post-outer-9 forced-private root `31840abd...5830`;
- unchanged `beta=1226.25 J`, `1e-8` inner stationarity threshold, trust
  radii/ratios and reject limit;
- zero public commit and exact rollback.

Then replay only the first failing inner solve and record its initial state plus
all nine rejected trials. No trial may update the replay state.

## Direct difference diagnostic

For each trial, retain the raw total-energy subtraction and independently
evaluate the same binary64 energy delta without subtracting large totals.

For inertia, with `s=trial-current`:

```text
Delta E_I = mass/(2*dt^2)
          * sum s dot (trial + current - 2*predicted).
```

For fixed multiplier and active coefficient `a`:

```text
Delta E_AL = sum (a_trial-a_current)*(a_trial+a_current)/(2*beta).
```

The diagnostic actual reduction is `-(Delta E_I + Delta E_AL)`. Evaluation
order is fixed by particle then constraint index. This is algebraically the
same objective difference; it is not a new merit function and cannot accept a
step in this stage.

Also record the ULP above the current total energy, step norm, predicted
reduction, raw actual reduction/ratio, HVP calls and exact active/topology
correspondence.

## Decision routes

- Positive model reduction, unchanged active topology and positive direct
  descent hidden by the raw gate select
  `NESTED_ACCURACY_AND_DIRECT_MERIT_RECLOSURE_REQUIRED`.
- If raw and direct differences already agree and would admit the smaller
  solve, select `INNER_ACCURACY_SCHEDULE_ONLY`.
- Nonpositive direct descent, nonpositive model or active/topology change
  selects `MODEL_OR_ACTIVE_SET_RESEARCH_REQUIRED`.
- Replay, finite-value, trial-count or rollback mismatch is a hard failure.

Every route authorizes only the corresponding remediation contract research.
No tolerance, beta, formula, state, trajectory or runtime path changes here.
