# TRAIN-4 R139 kinodynamic solve-method decision — 2026-08-15

| Field | Value |
| --- | --- |
| Scope | Research/roadmap choice after clean R138 graph conformance |
| Status | `R139_FORMULATION_AUTHORIZED / REPORT_ONLY / NO_SOLVE_AUTHORITY` |
| Selected method | Fixed-mode sparse multiple-shooting sequential convexification with OSQP QP subproblems and an exact quantized acceptance oracle |
| Immediate gate | One clean, hash-closed R139 solve formulation with zero real reconstruction, graph evaluation, assembly or solve |

## Decision

Use the exact R137/R138 fixed-mode graph as the accepted problem and a
continuous sparse SQP/SCvx model only to generate trial steps. The candidate
generator may linearize the manifold dynamics, sticking constraints, fixed
controller branch and descriptor bounds inside a normalized trust region. It
may also use temporary elastic variables and a finite polyhedral friction-cone
model so that a QP remains constructible from an infeasible anchor.

None of those approximations has acceptance authority. Every trial is rounded
ties-to-even into the signed integer command domain, replays the complete
R133-order fixed-PD limiter graph, re-evaluates the nonlinear rigid dynamics,
integration, anchors, activation impulses, circular force/impulse cones and
descriptor limits, and is compared by the exact normalized feasibility funnel.
An approximate controller effort, QP slack, polyhedral cone or floating trial
state can never establish feasibility or enter an output cache.

## Why this method

Primary-source review supports the pieces but does not prove this particular
Next Engine problem feasible:

- Posa, Cantu and Tedrake formulate direct rigid-body trajectory optimization
  through impacts and Coulomb contact, including an explicitly specified mode
  sequence as the conventional baseline. This supports the direct hybrid
  transcription already frozen by R137, not a contact-mode change
  ([paper](https://groups.csail.mit.edu/robotics-center/public_papers/Posa13.pdf)).
- Crocoddyl/FDDP treats a predefined contact sequence with sparse analytical
  derivatives and a multiple-shooting feasibility gap. It supports exploiting
  temporal structure and accepting an initially inconsistent trajectory, but
  its torque-control and equality-focused solver is not the exact quantized
  controller contract here
  ([paper](https://cmastalli.github.io/publications/crocoddyl20icra.pdf)).
- Successive-convexification work uses multiple shooting, exact penalties,
  virtual control and trust regions to globalize nonlinear optimal-control
  steps. Those mechanisms fit the broad R136 infeasibility better than another
  pointwise repair
  ([SCvx](https://arxiv.org/abs/1608.05133),
  [prox-linear successor](https://arxiv.org/abs/2404.16826)).
- TrajOpt and CRISP demonstrate sparse sequential convex subproblems, trust
  regions and exact/penalty merit evaluation in large robotic and contact-rich
  problems. They motivate the candidate generator and restoration funnel, not
  a relaxation of the final gate
  ([TrajOpt](https://rll.berkeley.edu/~sachin/papers/Schulman-IJRR2014.pdf),
  [CRISP](https://computationalrobotics.seas.harvard.edu/CRISP/)).
- OSQP solves convex QPs in the required sparse `l <= A x <= u` form, exposes
  residuals and infeasibility statuses, and is already pinned and exercised by
  the repository's earlier bounded trajectory work
  ([paper](https://arxiv.org/abs/1711.08013)).

The chosen method is therefore a bounded implementation fit, not a claim of
global convergence for the nonsmooth accepted graph.

## Rejected alternatives

### Monolithic sparse NLP / Ipopt

Ipopt is a credible large sparse smooth-NLP solver with a filter line search
([official documentation](https://coin-or.github.io/Ipopt/)). It is not
selected for R139 because the accepted controller contains ties-to-even
quantization and stateful min/max limiter switches, while the repository has
no pinned Ipopt/indefinite-linear-solver lineage. Using it would still require
the same continuous-surrogate/exact-replay split while adding a new numerical
stack and a much larger factorization risk.

### Mixed-integer or global MINLP

Mixed-integer contact planners are useful when gait/contact choice is the
research variable and often use reduced centroidal or convexified dynamics
([example](https://arxiv.org/abs/1904.04595)). R137 has already fixed every
contact mode, while exact command quantization would introduce `18400` integer
variables plus nonlinear full-body dynamics and limiter state. General MINLP
methods rely on branch-and-bound/outer approximation
([SCIP framework](https://optimization-online.org/2016/05/5433/)); that search
does not fit one single-threaded bounded discriminator and would answer a
broader global-optimality question than TRAIN-4 needs.

### DDP/FDDP/ALTRO replacement

Structure-exploiting shooting solvers are attractive for smooth torque-driven
optimal control. Adopting one now would require a new contact/impact,
inequality and quantized-controller implementation before its own conformance
could be assessed. R139 instead reuses the already conformed R137 indexing and
the pinned sparse-QP stack.

### Derivative-free exact search

The exact replay is nonsmooth, but black-box search over `311780` primary
scalars discards the known sparse dynamics structure. It is retained only as
the exact acceptance oracle; it is not a viable bounded step generator for
this dimension.

## Frozen R139 design

### Source reconstruction and endpoints

A future execution must start from a byte-exact R120 cache and independently
reconstruct the R133 projected schedule. Before the first real graph call it
must reproduce the R120 report/cache, R133 report, R133 projected-velocity,
applied-target and applied-effort hashes, the R131 mode hash and the R138 graph
index/event hashes. R136 cache, acceleration and force witnesses are forbidden
inputs.

The initial configuration is exact R120 and the initial generalized velocity
is exact R133 projected row zero. Those two values are immutable equalities.
Interior R120/R133 state and source command rows are deviation references only.
The terminal state is not byte-fixed or periodic: it remains subject to the
same terminal fixed mode, rigid anchors, dynamics, integration, controller and
descriptor bounds, with an explicit higher terminal tracking cost. This avoids
turning an unproved source endpoint into a hidden feasibility assumption.

Acceleration is initialized only from the reconstructed q/v reference by the
frozen discrete map. Continuous forces and activation impulses initialize to
zero. No pointwise inverse-dynamics witness is imported.

### Objective hierarchy

Hard feasibility is lexicographically above every tracking term. At each major
iteration the candidate generator uses three ordered QPs over the same local
model:

1. minimize the maximum normalized elastic violation;
2. hold that optimum within `1e-8` and minimize mean normalized elastic
   violation;
3. hold both feasibility optima within `1e-8` and minimize the normalized
   tracking/regularization objective.

Temporary elastic values are absent from emitted state. The tracking
tie-breaker uses mean squared deviations from R120/R133 for configuration,
velocity and commanded target with relative weights `1`, `1/4` and `1/2`;
terminal q/v terms receive `16x`; acceleration variation, force magnitude/
variation and impulse magnitude use weights `1/100`, `1/10000` and `1/10000`.
All terms use fixed physical scales from the R139 profile. The first exact
all-hard-constraint PASS terminates; no extra quality polishing is authorized.

### Trust region and globalization

Local variables use componentwise physical scales and one dimensionless
radius. The initial/minimum/maximum radius is `1`, `1/32` and `2`. A future
formulation freezes the exact per-group scales for root/joint q, v, a,
commands, forces and impulses.

Each major step audits fractions `1, 1/2, 1/4, 1/8, 1/16, 1/32, 1/64` in that
order. Exact PASS has priority. Otherwise a trial may be retained only when
its exact `(maximum, mean)` normalized violation decreases lexicographically
by the frozen minimum and its actual/model merit-reduction ratio is at least
`0.1`. Ratio below `0.25` contracts the radius by `1/2`; ratio at least `0.75`
with an active trust boundary expands it by `2`, capped at the frozen maximum.
No accepted trial means contraction and complete restoration of the previous
exact anchor. Failure at the minimum radius is a valid `STOP_AND_RESEARCH`,
not permission to tune tolerances or restart.

### Exact-versus-smooth boundary

The continuous controller model is an explicitly labelled candidate-only
fixed-branch surrogate. It removes integer rounding, differentiates only the
currently selected clamp/limiter branches and records a branch-address hash.
At an exact branch equality it uses the frozen zero derivative and records the
ambiguity count. It never emits applied effort as a free variable.

Every trial discards the surrogate output, rounds command scalars ties-to-even,
and derives applied targets and efforts with the exact R133-order controller
from the trial q/v observations. A changed floating command that rounds back
to the same integer row supplies no command progress. Exact controller array,
event-address and branch hashes are required diagnostics at every accepted
anchor.

The QP starts with `32` fixed angular half-spaces per circular force/impulse
cone. After an accepted exact anchor, each exact cone violation contributes one
deterministically ordered separating direction for the next model. The exact
circular cone is always the acceptance test; the polyhedral rows and their
elastic variables have no witness authority.

### Numerical and resource boundary

R139 must freeze float64 evaluation, the existing R136 rank gap
`null <= 1e-12` / `retained >= 1e-10`, expected `0/3/5` contact rank and the
known flat-foot force gauge. Any extra nullity, value inside the rank gap,
non-finite coefficient, unordered bound, disagreement between norm and
squared-cone classification, or solver residual/status ambiguity stops
invalid without restart.

The exact acceptance tolerances are frozen separately from OSQP termination:
scaled dynamics and momentum residual, manifold integration, anchor position,
sticking velocity/acceleration, circular cone and all existing integer
descriptor/controller gates. Passing a QP tolerance never substitutes for
passing one of them.

The future execution envelope is one clean process, one observed thread, seed
zero, at most six major iterations, eighteen lexicographic QPs, `100000` OSQP
iterations per QP and forty-two exact audits; wall time is four hours, peak RSS
is `16 GiB`, randomized/warm restart and manual intervention are forbidden.
Within-process factorization reuse is transient and hash-counted; R136 cache is
never loaded.

## Progression boundary

R139 itself performs zero real reconstruction, controller graph evaluation,
dynamics/Jacobian assembly, QP setup, factorization, solve, exact trial audit,
cache build, candidate build, PhysX scene or training run. Its exact
`COMPLETE` transition may authorize only a separate report-only R140
implementation/numerical-conformance increment using synthetic and
metadata-only discriminators.

Only an exact R140 PASS plus an explicit roadmap update may authorize one
bounded R141 execution under the frozen envelope. R139 cannot authorize that
solve directly. TRAIN-4 remains reopened; contact semantics, all-17/V19,
candidate publication, PhysX, corpus admission, visual/exhaustive gate and
every learned optimizer remain blocked.
