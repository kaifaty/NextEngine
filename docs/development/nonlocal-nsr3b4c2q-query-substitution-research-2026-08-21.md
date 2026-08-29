# NSR3-B4C2Q solver-query substitution research -- 2026-08-21

Status: `COMPLETE / ONE_SUBSTEP_QUERY_LIFECYCLE_SELECTED`

## Question

What must be proven before the joint neighborhood and pressure tape can replace
the all-pairs operator in the complete B4B2 controller?

## Call-graph audit

The B4B2 pressure path has four distinct workspace lifetimes:

1. the clamped macro forecast evaluates one uncommitted position and, only if
   active, reuses its Hessian for 48 Lanczos calls;
2. a KKT solve builds the feasible predicted **current** state and reuses its
   Hessian throughout trust-region CG and predicted-reduction evaluation;
3. every projected **trial** has different positions and must rebuild pair
   membership, density, gradient, CSR, radii and compression before its
   objective can participate in acceptance;
4. acceptance must atomically promote trial position, KKT state, neighborhood
   and tape; rejection must destroy all trial-owned data and leave current
   identity unchanged.

The box KKT projection itself is analytic per coordinate and performs no
neighbor query. Static support is immutable during the substep, but its pair
membership changes with fluid trial positions. Reusing current membership for
a trial is therefore invalid even if the active contact axes are unchanged.

## Why the old B4C2 scope is split

The earlier B4C outline combined one-substep substitution and the full B4B2
controller. Those have different failure localization. A one-substep mismatch
can come from query state, tape lifetime, projection, acceptance or KKT
reaction. A full-trajectory mismatch additionally includes adaptive frame
selection and continuation.

Freeze two gates:

- **B4C2Q:** current/trial/forecast query lifecycle and one-substep result;
- **B4C2T:** complete B4B2 adaptive and fixed-reference trajectories using
  only the selected B4C2Q operator.

B4C3 canonical publication remains after B4C2T. No authority is lost by the
split.

## Selected transaction model

Each private query owns:

```text
canonical positions
  + one-pass joint pair list
  + pressure Evaluation
  + compact CSR/radius/compression tape
  + state digest
```

The candidate path may call `evaluate_joint` while constructing a workspace
and `apply_joint_pressure_tape` for HVPs. It may not call the all-pairs
`evaluate` or `apply_hessian`; those calls belong only to a separately counted
audit oracle. The validation-only nested adjacency remains a B4C4 packaging
issue and receives no nominal memory credit.

An accepted trial moves the complete workspace. It is forbidden to move only
the numerical KKT state while retaining the old tape. A forced-reject control
fully constructs and audits a distinct valid trial, destroys it, and requires
the current digest and arrays to remain exact.

## Selected discriminator corpus

- P1 initial with the selected forecast coarse `dt = frame/21`;
- P1 feasible macro-forecast position with `dt = frame/42`;
- P2 detached initial state with `dt = frame`;
- P1 `0.99` compressed active holdout with `dt = frame/48`;
- P1 and P2 macro-forecast spectrum queries;
- a forced-reject transaction derived from the P1 forecast state.

The corpus is selected from already frozen states and timestep policies, not
from the outcome of the new implementation.

## Decision

Freeze B4C2Q as a bit-exact all-pairs-versus-joint discriminator. PASS may
authorize B4C2T full-controller contract design only. It cannot authorize
canonical continuation, nominal execution, runtime or production use.
