# TRAIN-4 R137 fixed-mode controller-reachable kinodynamic decision — 2026-08-15

| Field | Value |
| --- | --- |
| Scope | Research/roadmap choice after valid R136 cone infeasibility |
| Status | `R137_FORMULATION_AUTHORIZED / REPORT_ONLY / NO_SOLVE_AUTHORITY` |
| Selected branch | Fixed-mode rigid-contact trajectory co-design |
| Rejected branch for this increment | Changed contact semantics |
| Immediate gate | Exactly one clean, hash-closed R137 formulation report with zero numerical work |

## Decision basis

R136 closed the pointwise question without closing the trajectory question. All
`3200` systems were numerically valid, yet the exact projected state and exact
fixed-PD effort admitted a rigid unilateral friction-cone witness at only
`782` rows. The failure is broad across contact phases, not isolated to the
nine exits, and therefore cannot be repaired by another pointwise force gauge,
tolerance change or R136 retry.

The roadmap selects the smaller remaining semantic intervention: jointly vary
the state trajectory and controller command while retaining the current contact
model. Sliding, compliance, contact-implicit mode choice, friction changes and
new native contact semantics are deferred. This choice tests whether the
current rigid model is feasible when q/v/a, controller-reachable effort and
contact force are allowed to agree over time; it does not presume that such a
trajectory exists.

## Frozen branch boundary

R137 must preserve all of these facts:

- clip `cmu16-walk-nominal-b`, `800` motor intervals, four physics steps per
  interval, `3200` dynamics intervals and `3201` state nodes at `240 Hz`;
- R131's exact left-owned contact-mode sequence, 29 changed boundaries, nine
  contact entries, nine complete exits and eleven active-mode changes;
- R113 point ordinals, current rigid sticking constraints, current ground
  material and Coulomb coefficient `52429/65536`;
- current descriptor ROM, velocity, effort, effort-rate, power, work and
  collision limits;
- the exact fixed-PD observation, ties-to-even quantization, 60 Hz target hold,
  240 Hz limiter state and event ordering already exercised by R133/R136;
- R136 as immutable failure evidence. Its cache and force witness have no warm
  start, candidate or admission authority.

The branch may not select contact modes, delete points, raise friction, add
sliding, add compliance, soften a physical constraint, substitute free torque
for controller output or reinterpret a continuous controller surrogate as an
accepted result.

## Formulation that R137 must freeze

The transcription has `3201` manifold configuration and generalized-velocity
nodes. Every one of the `3200` intervals has generalized acceleration, exact
controller-derived effort and forces only for points active in the immutable
mode schedule. The `800x23` commanded targets are the only motor command
decisions; applied targets and `3200x23` efforts are derived by exact replay of
the frozen controller and limiter state, never independent decision variables.

For a regular physics interval, R137 must couple continuous rigid-body dynamics
to an explicit `1/240 s` integration map. Active sticking points retain their
source world anchors and satisfy position, velocity and acceleration closure;
every active force satisfies the unchanged unilateral circular Coulomb cone.
At a boundary that activates a point, a rigid activation impulse may change
velocity after continuous integration and must satisfy the same unilateral
friction cone. A release or point deactivation applies exactly zero impulse.
The event order and the inventory of newly activated/deactivated points must be
explicit and independently reproducible from the source mode sequence.

R120/R133 q/v, targets and effort are reference/initialization evidence only,
except for source identity and a frozen initial state. They are not hard
interior trajectory equalities and R136 forces are never imported. A later
candidate may be generated with a differentiable approximation, but acceptance
requires exact quantized controller replay and exact evaluation of every hard
dynamics, integration, contact, actuator and descriptor bound. Approximate
effort reachability has no acceptance role.

## Why contact activation impulse remains inside branch A

The selected model is still rigid, unilateral and fixed-mode. An activation
impulse represents the velocity jump at a scheduled rigid sticking event; it
does not introduce compliance, sliding, penetration or mode choice. Only newly
activated R113 points may carry it, and release impulses are forbidden. Native
correspondence is still unclaimed and remains a later gate before any PhysX
scene authority.

## Bounded progression

R137 is report-only. It may read and audit the exact R131 and R136 reports and
tracked source identities, formulate one symbolic transcription and inventory
its variables/events. It must perform zero state reconstruction, dynamics or
Jacobian assembly, controller schedule derivation, factorization, optimization,
candidate build, PhysX scene or training run.

An exact R137 `COMPLETE` may authorize only a separate report-only R138
implementation-conformance increment. R138 may test indexing, hybrid event
ordering, exact controller reachability and synthetic residual composition, but
may not assemble or solve the real `3200`-interval problem. A real kinodynamic
solve requires a later explicit decision that freezes objective/scaling,
algorithm, numerical tolerances, resource budget, output hashes and stop
conditions after R138 passes.

The pre-R137 transition is
`R137_FIXED_MODE_KINODYNAMIC_FORMULATION_AUTHORIZED_REPORT_ONLY`. TRAIN-4 stays
reopened; every TRAIN-5 checkpoint stays rejected. No R136 retry, kinodynamic
solve, candidate, PhysX, all-17/V19, corpus admission, visual/exhaustive gate,
optimizer or training work is authorized by this decision.
