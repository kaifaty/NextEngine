# NSR3-B4E2D7R19R7 second guard-boundary research

Date: `2026-08-23`

Status: `COMPLETE / REPLAY-ONLY DIAGNOSTIC SELECTED / CONTRACT FROZEN NEXT`

## Question

R6 proves that guarded residual completion solves the original sixth-trial
barrier and lets the private transaction reach nine accepted trials. Why does
the next solve that reaches 32 recurrence HVPs fail the same frozen guard, and
what happens if only that recurrence is continued offline?

## What R6 establishes

The next boundary is not the original one:

```text
original guarded solve
  first 32 safe
  ratio in (eta, 1.25 eta]
  last eight ratios decrease
  HVP 33 converges
  sixth trial accepted

later denied solve
  reached 32 recurrence HVPs
  guard attempt recorded
  guard denied
  no HVP 33
  no trial
```

The transaction has already advanced through nine accepted trials and into a
second outer update when the later denial occurs. The first guard therefore
cannot be generalized from one local state to every later Newton-CG system.

## Selected diagnostic

Attach a passive capture to the exact first later guard denial in a separate
R7 control run. The capture owns:

- predicted/current positions, normalized dual, `theta` and trust radius;
- the current normalized gradient and static-support identity;
- all first-32 recurrence iteration projections;
- each frozen guard clause separately;
- outer/trial/solve identity and pre-denial work counters.

The public R6 command remains the byte-exact default and receives no capture.
The R7 command first reproduces complete R6 bytes, then reruns the same private
candidate only to capture the first denial after one successful guarded
completion. Passive capture may hash/copy existing values but cannot consume
an HVP, construct a trial, audit precision or change a decision.

## Guard decomposition

At the 32-HVP boundary record independently:

```text
prefix_safe =
  all 32 HVPs finite
  and all curvatures positive
  and all candidates interior

ratio_not_converged = ratio_after_32 > eta
ratio_within_window = ratio_after_32 <= 1.25 eta
trailing_decrease = last 8 next-residual ratios strictly decrease
```

`ratio_after_32 <= eta` would contradict the live forcing termination and is
not an admissible policy observation. A legitimate guard denial must identify
at least one false clause after `prefix_safe && ratio_not_converged`.

Do not infer which clause fails from the aggregate R6 counter. That is the
first result R7 must measure exactly.

## Offline continuation

Rebuild exactly one normalized sparse workspace at the captured state and run
the historical Steihaug recurrence with:

```text
same gradient
same radius
same dimensionless forcing eta
same binary64 topology
no preconditioner
diagnostic recurrence cap = 128 HVP
```

The first 32 iteration projections must match the live capture exactly before
any later observation is admissible. The offline lane may then continue the
recurrence to finite/nonfinite, negative curvature, trust boundary, forcing
convergence or the cap. It records residual ratios, curvature, recurrence
orthogonality/conjugacy and the same CG-derived Ritz diagnostics used in R3.

There is no model-image HVP, residual-derived model, trial formation,
acceptance, precision audit or transaction continuation. The 128-HVP cap is a
diagnostic bound already used by R3, not a proposed runtime cap.

## Why replay precedes policy research

Several incompatible mechanisms produce the same aggregate denial:

1. residual ratio is still above `1.25 eta`, so one extra HVP was not locally
   justified;
2. ratio is near the threshold but the last-eight monotonicity certificate
   fails, indicating an oscillatory recurrence;
3. both clauses fail;
4. the recurrence subsequently reaches boundary/negative curvature rather
   than forcing convergence;
5. it converges after a small finite continuation, but under a different
   locally defensible certificate.

Only the last case could motivate a later guard-policy discriminator. Even
then R7 cannot select that policy; it only supplies exact evidence for a new
research decision.

## Rejected actions

### Widen `(eta, 1.25 eta]` now

Rejected. The second ratio and convergence distance are not known.

### Drop the last-eight trend condition

Rejected. R6 does not identify it as the failing clause, and non-monotone
residual history may be the safety signal.

### Grant a global recurrence cap of 33 or 128

Rejected. The former bypasses the guard and the latter confuses a replay cap
with runtime authority.

### Continue the transaction after offline convergence

Rejected. An offline recurrence is not an accepted trial and has no divided,
precision, boundary or impulse evidence.

### Add a preconditioner

Rejected. R3 rejected it for the first barrier, while the conditioning of the
new system has not yet been measured.

## Smallest falsifiable experiment

Freeze D7R19R7 to:

1. reproduce exact R6/R5/R2 bytes and semantic roots;
2. passively capture the first later guard denial after one guard convergence;
3. bind the R6 transaction root, `2/9/0/227` outer/accepted/rejected/HVP facts
   and exact pre-denial state/work identity;
4. decompose every guard clause without changing the live denial;
5. rebuild one static-bound sparse workspace and reproduce the live first-32
   recurrence prefix exactly;
6. continue only that recurrence offline to a cap of 128 HVPs;
7. record termination, residual, curvature, recurrence and Ritz evidence;
8. retain finite, static binding, lifecycle, zero all-pair work and rollback;
9. form no trial, perform no acceptance/precision audit, continue no
   transaction and run no second substep, macro, trajectory or timing lane;
10. reproduce one report from each of two clean Release builds.

## Authority boundary

D7R19R7 may classify one denied recurrence. It cannot change the frozen guard
or any runtime cap, use an offline step, form a trial, continue the private
transaction, publish state, claim performance or grant runtime/production
authority.
