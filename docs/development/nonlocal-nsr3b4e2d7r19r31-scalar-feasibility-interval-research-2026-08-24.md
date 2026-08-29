# NSR3-B4E2D7R19R31 scalar-feasibility interval research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN NEXT / REPORT ONLY`

## Question left by R30

R30 proves that the 1,420 violated-row right-hand side is almost entirely in
the range of the exact linearized constraint operator. It does not prove that
the resulting direction is a feasible inequality step. The same direction
creates 450 positive rows outside the selected set.

The next question is deliberately narrower than a new solver:

> Does any scalar `alpha in [0,1]` along the exact R30 direction make every
> originally violated row nonpositive without making any originally inactive
> row positive in the frozen R29 linear model?

This question has an exact interval answer and needs no fitted progress
threshold.

## Research basis

Fraction-to-boundary rules are safeguards on a computed direction; in a
large-scale interior-point implementation they are followed by a line search,
not treated as a complete feasibility solver. See Wächter and Biegler,
[On the implementation of an interior-point filter line-search algorithm for
large-scale nonlinear programming](https://cepac.cheme.cmu.edu/pasilectures/biegler/ipopt.pdf),
equations 15a--15b.

For an infeasible current iterate, simply imposing a trust region on exact
linearized feasibility can make the subproblem infeasible. Trust-region SQP
therefore commonly computes a step that reduces infeasibility while remaining
inside the trust region. See Wright and Tenny,
[A Feasible Trust-Region Sequential Quadratic Programming Algorithm](https://pages.cs.wisc.edu/~swright/papers/WriT04a.pdf),
especially the discussion around equations 1.2--1.3. Composite-step
matrix-free SQP likewise separates quasi-normal and tangential work; see
Heinkenschloss and Ridzal,
[A Matrix-Free Trust-Region SQP Method for Equality Constrained
Optimization](https://repository.rice.edu/bitstreams/13d38ec8-775a-4c51-94c4-52b608fc0d07/download).

If scalar globalization is impossible, a matrix-free linearly
inequality-constrained trust-region subproblem is a credible later direction,
not an invented GPU-specific workaround. Kearsley describes such a method
using only inner products and matrix rows/columns in
[A Matrix-Free Algorithm for the Large-Scale Constrained Trust-Region
Subproblem](https://www.nist.gov/publications/matrix-free-algorithm-large-scale-constrained-trust-region-subproblem).

These sources motivate the branch but do not choose a production algorithm
for this project.

## Exact scalar interval

Let `c` be the frozen all-row constraint vector and `r=A v` the exact full-row
R30 response. Let `V={i:c_i>0}` and `I={i:c_i<=0}`.

For a selected violated row, feasibility requires

```text
c_i + alpha r_i <= 0.
```

If `r_i>=0`, no nonnegative scalar step can repair that row. Otherwise it
requires

```text
alpha >= c_i / (-r_i).
```

Therefore

```text
alpha_lower = max over V of c_i/(-r_i).
```

For an inactive row with `r_i>0`, retaining nonpositivity requires

```text
alpha <= (-c_i)/r_i.
```

Therefore

```text
alpha_upper = min(1, min over I,r_i>0 of (-c_i)/r_i).
```

A scalar linearized-feasibility interval exists exactly when no selected row
has nonnegative response and `alpha_lower<=alpha_upper`. This is stronger and
less arbitrary than asking whether the first inactive crossing occurs before
some chosen fraction such as `0.5` or `0.99`.

## Arithmetic and direct predicates

The source vectors and membership are binary64-owned. Order all candidate
ratios independently in Linux x86-64 `__float128`, then convert the winning
bounds to binary64. Repair the binary64 lower bound upward and upper bound
downward with at most 64 `nextafter` operations until separate binary64
multiply-add predicates reproduce the intended selected/inactive sides.

The wider arithmetic is diagnostic-only. It does not enter runtime state and
does not change R30's vector. Report both bound rows, exact membership roots,
binary128 bound encodings, binary64 bound bits, repair counts and direct
predicate counts.

Four tiny controls precede the nominal scan:

1. a nonempty feasible scalar interval;
2. an inactive row that blocks before selected repair;
3. a selected row with nondecreasing response;
4. a zero-margin inactive row whose upper bound is zero.

## Required observations

Without adding a pass threshold, report:

- lower/upper bounds, interval gap and blocker roots;
- selected positive count/norm at the strict safe upper bound;
- inactive positive count/norm at the repaired required lower bound;
- predicted selected violation reduction at the safe bound;
- R30 preimage RMS/maximum multiplied by each bound;
- zero-margin and nondecreasing selected counts;
- exact parent/source/partition and rollback roots.

## Classification

- nonempty direct interval: `SCALAR_FEASIBILITY_INTERVAL_CANDIDATE`;
- empty direct interval with all proof controls passing:
  `INEQUALITY_ACTIVE_SET_REFORMULATION_REQUIRED`.

Both are successful report-only classifications. The second route means
scalar damping of this exact direction cannot satisfy all frozen linearized
inequalities simultaneously; it does not by itself select an active-set,
interior-point or trust-region implementation.

## Authority boundary

R31 replays R30 once and performs zero new pair passes, HVPs, model
evaluations, trials, precision audits or outer updates. It does not apply a
direction, evaluate a moved nonlinear state, classify a nonlinear floor,
execute another outer/substep/macro/trajectory, measure timing or acquire
runtime/production authority.

## Continuation

- nonempty interval: separately research a nonlinear trial contract inside
  the interval;
- empty interval: separately research/freeze a matrix-free minimum-norm or
  trust-region inequality normal-step subproblem with active-set closure;
- arithmetic/source failure: repair that boundary without interpreting the
  nominal interval.
