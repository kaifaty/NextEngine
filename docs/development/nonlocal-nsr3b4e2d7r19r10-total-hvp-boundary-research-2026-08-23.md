# NSR3-B4E2D7R19R10 total-HVP boundary research

Date: `2026-08-23`

Status: `COMPLETE / PASSIVE BOUNDARY DIAGNOSTIC SELECTED`

## Question

R9 removes both known local Krylov-completion barriers. Its private
transaction reaches outer update `6`, accepts `20` trials without rejection
and then consumes the unchanged global allowance exactly:

```text
494 recurrence HVP + 18 direct-model HVP = 512 total HVP
```

The aggregate ledger does not say whether the global cap interrupted useful
convergence, an unsafe recurrence, or repeated work with no useful outer
progress. Raising the cap before distinguishing those cases would turn a
diagnostic boundary into an arbitrary policy change.

The next falsifiable question is:

> What exact outer/trial work led to HVP 512, and what numerical state did the
> in-flight trust recurrence have when the existing total budget stopped it?

## Competing explanations

### H1. Safe recurrence interrupted while still progressing

The in-flight prefix remains finite, positive-curvature and strictly inside
the trust region. Its late residual window improves over its early window.
This would justify a separate offline-continuation experiment, but not a live
budget increase.

### H2. Safe recurrence already near forcing convergence

In addition to H1's safety, the final prefix residual lies within `2 eta` and
the final up-to-eight residual ratios strictly decrease. A later experiment
could then test a small bounded continuation from this exact state.

### H3. Safe but inconclusive/stalled recurrence

The prefix is finite and interior but the late residual window does not
improve robustly. More global budget would have no evidence-based bound and
must not be proposed from R10.

### H4. Unsafe or contradictory recurrence

Nonfinite arithmetic, nonpositive curvature, a trust boundary, or forcing
already reached before the budget check would make a cap increase the wrong
response. The first such fact must own the route.

## Selected diagnostic design

Refactor the R9 report entry into an internal implementation with an optional
capture sink. The ordinary public R9 command passes no sink and must retain
its exact stdout and semantic roots. R10 calls that same implementation once,
so the parent-control transaction and the captured transaction are the same
execution rather than two nominal substeps.

Attach one optional total-budget sink to the tiered completion trace. Only
when `al_normalized_trust_step` returns because
`STRUCTURAL_BUDGET_TOTAL_HVP` is already set may the inner solver copy:

```text
outer / trial / solve identity
predicted / current / gradient / u / theta / radius
all recurrence iterations completed before HVP 512
budget, workspace, precision, static and adjacency counters
```

Copying is observational work only. It may not call an HVP, evaluate another
state, form a model or trial, update a radius, audit precision or mutate the
transaction.

## Progress projection

R10 must expose every completed outer update and its completed trials. The
projection includes:

- per outer: completion/failure, trials, accepted/rejected, inner HVP,
  stationarity and the available primal/dual/complementarity/update state;
- per trial: outer/trial identity, recurrence/model work, completion tier,
  stationarity/radius, predicted/divided ratio, decision and state roots;
- cumulative completed-trial work plus the interrupted recurrence work;
- exact equality with `494 + 18 = 512` and the R9 transaction root.

The failed outer has no completed final outer state, so its default primal or
dual fields must not be interpreted as physical progress. Its trustworthy
facts are the entry state, completed trials, stationarity before the
interrupted trust solve and its captured recurrence.

## Frozen prefix classifier

For a recurrence prefix of `n > 0`, define an early and late window of
`min(8,n)` `next_residual_ratio` values. Compare both minimum and median.

Precedence is:

1. any nonfinite iteration or HVP failure;
2. first nonpositive curvature;
3. first trust-boundary return;
4. forcing already reached in the captured prefix;
5. safe near-forcing: final ratio in `(eta,2 eta]`, final window strictly
   decreases, and both late minimum and median improve over the early window;
6. safe progressing: final ratio is below the initial ratio and both late
   minimum and median improve;
7. safe inconclusive/stalled.

This classifier describes only the captured prefix. It is not a convergence
proof, cap estimator or production admission rule.

## Rejected alternatives

### Raise `maximum_total_hvp`

Rejected. R9 provides no post-boundary recurrence or outer-progress evidence
from which to derive a new limit.

### Replay beyond HVP 512 in R10

Rejected. That would mix observation with the policy decision being studied.
A continuation, if justified, needs its own frozen contract and offline work
ledger.

### Target the known outer/trial by hard-coded identity

Rejected. Capture is triggered by the exact budget failure, not by an
expected ordinal. The observed ordinal is evidence, not control flow.

### Use wall-clock timing

Rejected. The open question is numerical work and convergence, while the
shared-host performance lane remains stopped.

## Decision

Freeze R10 as one passive total-HVP boundary diagnostic over the exact R9
execution. A PASS may authorize only research/freeze of a later offline
continuation or a global-budget-policy discriminator. It cannot raise a cap,
continue the recurrence, form an additional trial, commit state or claim
runtime/production readiness.
