# NSR3-B4E2D7R19R32 all-inequality Cauchy normal-step research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN NEXT / REPORT ONLY`

## Question left by R31

R31 proves that scalar damping cannot globalize the violated-row R30
direction. It does not show whether the all-inequality linearized feasibility
model has a useful bounded descent direction.

The next primitive should be cheaper and weaker than a full QP solver, but it
must be a lower bound on the quality expected from any later normal-step
method.

## Normal-step model

Use the frozen all-row R29 operator `A=SPACING*Jc` and constraint vector `c`:

```text
phi(v) = 1/2 ||[c + A v]_+||^2.
```

This convex piecewise-quadratic hinge model measures only linearized
inequality infeasibility. At `v=0`,

```text
g = A^T [c]_+.
```

If `g!=0`, define the unit global-L2 steepest direction

```text
d = -g / ||g||.
```

The trust bound is not fitted. The existing normalized inner solver starts
each solve at physical radius `0.25*SPACING`; because R29's coordinate is
dimensionless, R32 inherits exactly

```text
0 <= alpha <= Delta = 0.25,
v = alpha d.
```

For an infeasible iterate, trust-region SQP generally seeks reduction in
infeasibility inside the trust region rather than requiring exact linearized
feasibility. Wright and Tenny state this distinction explicitly in
[A Feasible Trust-Region Sequential Quadratic Programming Algorithm](https://pages.cs.wisc.edu/~swright/papers/WriT04a.pdf).
Matrix-free composite-step SQP separates a quasi-normal feasibility component
from a tangential component; see Heinkenschloss and Ridzal,
[A Matrix-Free Trust-Region SQP Method for Equality Constrained
Optimization](https://repository.rice.edu/bitstreams/13d38ec8-775a-4c51-94c4-52b608fc0d07/download).
Kearsley independently demonstrates that large linearly
inequality-constrained trust-region subproblems can be attacked matrix-free
using operator rows/columns and inner products:
[NIST publication](https://www.nist.gov/publications/matrix-free-algorithm-large-scale-constrained-trust-region-subproblem).

R32 does not implement or claim equivalence to any complete algorithm from
those papers. It closes only the Cauchy normal-step prerequisite.

## Exact piecewise line minimization

Let `z=A d`. Every row changes hinge membership only at

```text
t_i = -c_i/z_i,
```

when `t_i` lies in `(0,Delta]`. Order events in binary128 and break ties by
stable row ID. Between adjacent event groups,

```text
phi'(alpha) = a + alpha b,
b = sum(active z_i^2) >= 0.
```

Maintain `a,b` with a fixed long-double fold. The first root `-a/b` inside a
segment is the global line minimizer because `phi` is convex and continuously
differentiable. If no root occurs before `Delta`, the trust boundary is the
minimizer. Simultaneous events are processed as one group.

This is a deterministic sweep-line algorithm: `O(N log N)` row sorting plus
one `A^T w` and one `A d`. It maps naturally to a later GPU radix sort or
segmented event reduction, but R32 makes no performance claim.

## Controls and direct acceptance

Four dense analytic cases run first:

1. interior root without an event;
2. decreasing objective whose minimum is the trust boundary;
3. an inactive row entering before the interior root;
4. simultaneous leave/enter at the minimizer.

For the nominal model require:

- exact parent/source/workspace/topology roots;
- finite nonzero gradient or exact positive-violation stationarity route;
- unit direction and step norm at most `0.25`;
- positive direct model reduction when `g!=0`;
- interior `|phi'(alpha)|/||g|| <= 1e-10`, or nonpositive derivative at the
  trust boundary;
- exact event ordering/ties, two new pair passes and rollback.

The reduction magnitude, final violation ratio, active-set changes and step
localization are report-only. No threshold may be chosen after observation.

## Routes

- positive violation with exact zero `g`:
  `ALL_INEQUALITY_LINEARIZED_STATIONARY`;
- verified positive Cauchy reduction:
  `ALL_INEQUALITY_CAUCHY_NORMAL_STEP_CANDIDATE`;
- parent/source/workspace/dense/gradient/line/KKT/work failures remain hard.

Both successful routes are private classifications. Neither is a correction
or a production solver.

## Work and authority

Replay exact R31 once, build one read-only workspace, execute one VJP and one
JVP. No LSMR, HVP, objective/model trial, precision audit, outer update,
nonlinear moved state, substep, macro, trajectory or timing. Do not mutate
particle/dual/public/world state.

## Continuation

- useful Cauchy reduction: research a separately frozen iterative
  all-inequality normal-step method that must achieve at least this decrease;
- exact stationary positive violation: investigate an all-row range/
  constraint-qualification failure before any optimizer change;
- hard failure: repair the first exact boundary without interpreting nominal
  reduction.
