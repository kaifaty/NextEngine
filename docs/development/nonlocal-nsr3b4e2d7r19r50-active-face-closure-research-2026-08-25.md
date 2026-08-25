# NSR3-B4E2D7R19R50 active-face closure research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / HILDRETH REFERENCE SELECTED / CONTRACT FREEZE RECOMMENDED`.

## Question

R49 makes the global hinge tiny but leaves 366 directed-positive rows. Should
the next stage continue nonlinear-CG, use semismooth Gauss--Newton, or switch
from aggregate minimization to direct projection onto the remaining linearized
halfspaces?

## Structural reading of R49

The R49 endpoint has:

```text
h                           1.4144370952740040e-10
maximum directed upper      1.9771445876661678e-11
directed-positive rows      366
witness norm                5.0385810268018367e-7
normal radius               0.03125
```

The trust/contact set is still far away, while the aggregate objective has
already fallen by three orders of magnitude relative to R48. Another observed
HZ block would answer only how long the same recurrence can be extended. The
remaining problem is better stated as a halfspace-intersection closure:

```text
find p of minimum norm such that u + B p <= 0
```

where `u` is the current directed row-upper vector on its positive rows and
`B` is the corresponding restriction of the unchanged `A` operator. This is a
generator model only; the fresh all-row directed audit remains authoritative.

## Algorithm research

The classical relaxation method projects an infeasible point toward a violated
linear halfspace and proves convergence properties for consistent systems;
the original paper also distinguishes projection, under-relaxation and over-
relaxation. See Motzkin and Schoenberg's
[primary paper](https://doi.org/10.4153/CJM-1954-038-X).

Hildreth formulates a quadratic-programming procedure for linear inequalities.
For the minimum-norm halfspace problem, its dual has nonnegative coordinates:

```text
maximize  lambda^T u - 0.5 lambda^T (B B^T) lambda
subject to lambda >= 0
```

A cyclic coordinate update is exact for one dual coordinate:

```text
lambda_i' = max(0, lambda_i + predicted_i / G_ii)
p'        = p - (lambda_i' - lambda_i) b_i
predicted = predicted - (lambda_i' - lambda_i) G[:,i]
```

where `G=B B^T`. This directly enforces feasibility while retaining a minimum-
norm correction interpretation. See Hildreth's
[primary paper](https://doi.org/10.1002/nav.3800040113). Dykstra's later
[restricted least-squares algorithm](https://doi.org/10.1080/01621459.1983.10477029)
supports the same projection-family interpretation.

Semismooth Gauss--Newton on `A_P^T A_P` remains viable, but it solves a
stationarity equation and needs a singular normal-equation policy. Hildreth is
the narrower test here: nonnegative dual variables, explicit halfspaces, no
generalized-Hessian choice and an exact coordinate minimizer. A single-row
Motzkin projection is simpler but discards the coupling already exposed by the
366-row active face.

## Selected reference

Freeze a four-outer active-face Hildreth reference from the exact R49 witness:

1. run one fresh all-row directed audit and select every row with `upper>0`;
2. stop structurally if more than 512 unique rows enter. `512` is the next
   power-of-two storage capacity above the exact 366-row parent set; rows are
   never truncated;
3. cache each unique row gradient with one VJP `A^T e_i` and its full Gram
   column with one JVP `A(A^T e_i)`. The fixed operator permits reuse across
   outer active-set changes;
4. require positive diagonal, finite values and symmetric active Gram entries
   under a predeclared `1e-12` relative audit;
5. perform exactly eight stable-row-order Hildreth sweeps, publishing
   checkpoints `1/2/4/8`; no convergence tolerance ends a sweep early;
6. form the minimum-norm correction `p=-B^T lambda`, project `d+p` with the
   unchanged R48 ball-box projector and use the feasible chord;
7. audit candidates on the fixed dyadic ladder `alpha=1,1/2,...,2^-8`; accept
   the first candidate that strictly lowers `psi`, `h` and maximum directed
   upper, or that independently certifies all rows;
8. rebuild the positive directed active set and repeat for at most four outers.

The active capacity and `1/2/4/8` sweep ladder are frozen from data-structure and
algorithm structure, not from a nominal R50 outcome. Four outers are the first
bounded active-set change ladder; no fifth retry is allowed.

Each unique active row costs one VJP and one JVP, at most 1024 pair passes. The
initial audit plus at most nine candidate audits per outer add at most 37 JVPs,
for a total new cap of 1061 pair passes. This is deliberately a correctness
reference, not a scalable implementation. If selected, a later stage must
replace basis-vector passes with local row extraction/overlap data structures
before making a performance claim.

## Certificate and failure semantics

- Only a fresh all-row R48 directed enclosure with zero positive uppers selects
  `RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE`.
- Strict progress without a certificate selects only an active-face closure
  candidate.
- Capacity overflow, nonsymmetric/nonpositive Gram geometry, projection
  failure, absent dyadic progress or work overflow are separate exact routes.
- A positive stationary dual/hinge value is not an infeasibility proof. R48's
  unchanged negative dual lower bound remains the only available dual fact.

## Alternatives not selected

- **More HZ iterations:** rejected because R49 already fixed and exhausted its
  transfer block; extension would fit work to the observed endpoint.
- **Semismooth Newton/LSMR first:** retained as fallback if Hildreth exposes
  rank/conditioning failure, but it requires a forcing/regularization policy.
- **Single-row Motzkin heap:** attractive for a local production kernel and
  sports-programming-style priority queue, but it ignores the small coupled
  face in the first reference.
- **Dense generic QP/HSDE:** too broad and less matrix-free; reserve for a later
  independent infeasibility oracle.
- **Directed tolerance:** prohibited; row upper must be nonpositive exactly.

## Recommendation

Freeze and implement R50 as a rollback-only active-face Hildreth reference.
Do not apply its correction, exit restoration, update filter/trust state, run a
following outer or admit timing/runtime/production authority.
