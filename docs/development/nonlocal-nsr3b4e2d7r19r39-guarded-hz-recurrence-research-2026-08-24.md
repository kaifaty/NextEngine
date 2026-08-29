# NSR3-B4E2D7R19R39 guarded Hager--Zhang recurrence research -- 2026-08-24

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`.

## Problem

R38 proves that one raw Hager--Zhang direction, after the existing trust-ball
projection and exact hinge line solve, strictly dominates a steepest step at
three exact R37 states. It does not prove that the direction remains useful
after it owns and updates its own history. The next smallest falsifiable
question is therefore a short recurrence, not integration or timing.

## Primary-source boundary

The Hager--Zhang
[primary method](https://doi.org/10.1137/030601880) guarantees sufficient
descent for its unconstrained direction and reduces to a nonlinear
Hestenes--Stiefel method under exact line search. Its global result assumes a
Wolfe line search. Neither guarantee transfers automatically through our
normalization, global trust-ball projection and squared-hinge active changes.

Hager and Zhang's later
[PASA implementation](https://doi.org/10.1145/3583559) treats constrained CG
as an active-face phase: projected gradient identifies or grows the active
face, CG operates while that face is stable, and CG restarts when a new
constraint becomes active. Our hinge active set is part of the objective
rather than the feasible polyhedron, so PASA is not a proof for this problem.
It is, however, direct primary evidence against assuming that unguarded CG
memory survives arbitrary active changes.

Consequently R39 retains four independent numerical obligations at every
step: finite beta, raw descent, projected-chord descent and exact line KKT.
Theory selects the experiment; only the frozen replay may select a candidate.

## Selected trajectory

Start both lanes from exact R37 checkpoint `24`, including its maintained
response, residual, current gradient, previous gradient and previous accepted
feasible normalized direction. R38 must replay byte-exactly and its source
Hager--Zhang direction must reproduce record
`185635ed433b785f5df56533c467a0ff2a13ab61c89ce9e8cddca9632777e18b`.

Run two private linearized lanes for at most eight accepted steps:

1. `STEEPEST`: the unchanged R37 projected normalized negative gradient;
2. `GUARDED_HAGER_ZHANG`: raw R38 beta and direction, followed by the exact
   R38 projection/descent guard and steepest restart.

For the Hager--Zhang lane, each accepted feasible normalized chord becomes
`d_prev`, and the gradient that formed it becomes `g_prev`. A restarted
steepest chord owns history in exactly the same way, so the next iteration may
attempt memory again without resurrecting the rejected direction.

Both lanes use the unchanged exact R32 piecewise squared-hinge line solve and
accept only strict objective reduction with direct KKT and trust feasibility.
Capture fresh metrics after steps `1/2/4/8`. Exact zero projected mapping is
the only early stationarity route; no tolerance is inferred.

The first Hager--Zhang checkpoint must reproduce R38 state-24 one-step
objective, violation, active count and line/direction roots. This distinguishes
a true recurrence from a subtly changed first step.

## Equal-work ledger

Each lane owns exactly:

```text
1 source JVP
+ 8 line JVPs
+ 7 intermediate gradient VJPs
+ 1 terminal direct JVP
+ 1 terminal direct VJP
= 18 pair passes
```

The two lanes therefore admit at most `36` new pair passes: `20` JVP and `16`
VJP passes, with zero HVP/model/trial/outer work. Early exact stationarity may
spend less; no other early stop receives performance credit. This is an
algorithmic-work comparison, not a wall-time benchmark.

## Frozen selection

Select `GUARDED_HZ_RECURRENCE_CANDIDATE` only if:

- every executed direction, line, state and direct-terminal gate passes;
- the Hager--Zhang lane has at least two consecutive accepted non-restarted
  memory directions;
- at every common checkpoint `1/2/4/8`, Hager--Zhang strictly improves
  objective, violation norm and active projected mapping over steepest; and
- it does not consume more pair passes than steepest.

Exact projected stationarity has higher route precedence. If Hager--Zhang
retains terminal benefit but never establishes a two-step memory streak,
classify `HZ_ONE_STEP_BENEFIT_ONLY`. Otherwise retain the steepest reference.

The selected route remains rollback-only. It authorizes no correction,
nonlinear moved-state evaluation, tolerance, runtime policy or production
claim.

Frozen contract:
[R39 guarded recurrence](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r39-guarded-hz-recurrence-contract.md).
