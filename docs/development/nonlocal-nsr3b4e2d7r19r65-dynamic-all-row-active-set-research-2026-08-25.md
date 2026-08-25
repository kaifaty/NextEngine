# NSR3-B4E2D7R19R65 dynamic all-row active-set research

Date: `2026-08-25`

Status: `RESEARCH CONTINUES / MATRIX-FREE OWNERSHIP VALIDATED /
ACTIVE-FACE SHQP ACCELERATION REQUIRED`.

## Mathematical owner

R63's pure-inertia TRQP is the best-approximation problem

```text
minimize     0.5 * ||s - target||^2
subject to   c_i + a_i^T s <= 0,  i = 0..5999
             s in C = contact_box intersect trust_ball(0.0625)
```

R63 already has the correct Dykstra decomposition: every density halfspace
owns a nonnegative scalar correction `lambda_i`, while the exact joint
`box∩ball` projector owns one vector correction `p_C`. For an owned row,

```text
r_i       = c_i + a_i^T s
lambda'   = max(0, lambda_i + r_i / ||a_i||^2)
delta     = lambda' - lambda_i
s         = s - delta * a_i
```

and the joint block is

```text
q         = s + p_C
s         = project_box_ball(q)
p_C       = q - s
```

For Euclidean halfspaces Dykstra reduces to Hildreth's quadratic-programming
method. Essentially cyclic control is allowed, and warm-started dual states
remain a valid convergence direction. Primary references:

- [Bauschke and Lewis, Dykstra's algorithm with Bregman projections](https://people.orie.cornell.edu/aslewis/publications/00-dykstras.pdf)
  (Euclidean Dykstra, halfspace/Hildreth equivalence and essentially cyclic
  control);
- [Pang, Supporting Halfspace--Quadratic Programming Strategy](https://doi.org/10.1137/16M106090X)
  (dual alternating-minimization view and warm-start convergence);
- [Perkins, A Convergence Analysis of Dykstra's Algorithm for Polyhedral Sets](https://doi.org/10.1137/S0036142900367557)
  (polyhedral projection convergence and the warning that intermediate
  iterates need not be feasible).

## Selected monotone constraint generation

Initialize the working set from every row whose fresh R61 row-local upper is
positive at `target`. Keep a 6000-entry `lambda` vector and a monotone bitset;
new rows start with zero dual and are inserted in ascending order. Never drop
an owned row in R65.

Run stable ascending Hildreth sweeps over the owned set, followed by the joint
box-ball Dykstra block. After the joint block, execute one fresh exact-order
sparse all-row action and the unchanged R61 row-local enclosure. Add all
candidate-positive rows outside the working set. Because ownership only grows,
there can be at most 6000 additions. If a reduced-set projection is feasible
for every omitted halfspace, it is also the full problem's projection: the
reduced minimizer is feasible for the smaller full feasible set and therefore
retains its minimum objective there.

R65 remains a bounded-depth reference, not a finite-convergence claim. An
exploratory no-credit probe must inspect fixed checkpoints before one depth is
frozen. No observed residual may be turned into a fitted tolerance.

## Do not use a Gram update per coordinate

R64 has 535,588 row-particle entries, about 89.3 per row and per particle.
Updating every row sharing those particles would visit on the order of the
square of that degree before deduplication. For cyclic Hildreth this is more
work than directly evaluating the one current row.

The selected coordinate path is therefore matrix-free:

1. compute `r_i` directly from the current row's unique sparse entries;
2. update `lambda_i`;
3. update only `s` entries owned by that row;
4. after the joint projection, refresh every row once through directed slots.

This is `O(|W| K + N K)` per sweep rather than dense Gram work or roughly
`O(|W| K^2)` incidence-overlap propagation. Particle incidence remains
available for later dirty queues, coloring and GPU scheduling, but receives no
mandatory solver credit in R65.

## KKT and projection audit

Feasibility alone is insufficient. At every frozen checkpoint audit:

- fresh R61 raw/candidate all-row feasibility;
- contact box and trust radius;
- nonnegative finite multipliers and exact working-set ownership;
- complementarity `lambda_i * (c_i + a_i^T s)`;
- deterministic reprojection `project_box_ball(s + p_C) == s`;
- the Dykstra stationarity invariant
  `s - target + A^T lambda + p_C = 0` under a predeclared sparse-transpose
  forward bound;
- inertia and strict model reduction against the cached-normal endpoint;
- full-vector roots and exact work/rollback.

The stationarity identity follows directly from the dual corrections and is
preserved by both update types. It distinguishes an arbitrary feasible point
from the best-approximation solution sought by the TRQP.

## Exploratory probe and frozen-stage candidates

Probe fixed checkpoints `8,16,32,64,128,256,512,1024,2048` from one unchanged target, with
no tolerance stop. Record working-set size/additions, positive rows,
stationarity, complementarity, joint correction, inertia and direct sparse
work. The later frozen contract must select one depth before its nominal run.

Predeclared scientific outcomes are:

```text
DYNAMIC_ALL_ROW_PROJECTED_TRQP_CANDIDATE
DYNAMIC_ACTIVE_SET_DEPTH_REQUIRED
DYNAMIC_ACTIVE_SET_EXPANSION_REQUIRED
DYNAMIC_ACTIVE_SET_KKT_ALIGNMENT_REQUIRED
DYNAMIC_ACTIVE_SET_MODEL_REDUCTION_REJECTED
```

R65 executes only the linearized convex projection. Nonlinear density,
switching/filter acceptance, runtime publication, timing and production remain
out of scope.

## Exploratory outcome

The [probe evidence](nonlocal-nsr3b4e2d7r19r65-dynamic-all-row-probe-evidence-2026-08-25.md)
shows that monotone ownership stabilizes at 4680 rows before checkpoint 8, but
ascending Hildreth still has maximum raw `9.17e-12` after 2048 cycles. Fresh
most-violated ordering is worse. `omega=1.5` reaches `2.77e-12` but does not
change the algorithmic conclusion. Stationarity, complementarity, joint
reprojection and model reduction remain coherent. Stop depth/order/omega
tuning and research an SHQP/active-face block polish over the stabilized dual
face.
