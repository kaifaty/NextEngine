# TRAIN-4 R127 contact-state consistency research — 2026-08-15

| Field | Value |
| --- | --- |
| Scope | Optimizer-free causal research after the sole R127 stopped before its first gauge/cone classification |
| Status | `R127_INVALID / R129_PASS / R130_INVALID / R133_PASS / R134_COMPLETE / R135_CONFORMANCE_NEXT` |
| Primary cause | `Frozen q/v is incompatible with two-point FlatSticking acceleration closure` |
| Claim ceiling | Static research and generated-test design only; no R127 retry, feasibility claim, candidate, scene or training |

## Observed boundary

The sole R127 process ran from clean commit `44b536b`, passed all six
validations (`300/300` lab, motor and full `host-check`), and consumed its one
execution authority. At collocation `0`, right heel and forefoot are active on
the same `body.right-ankle-roll`. The reduced `35 × 35` system has expected
rank `34`, nullity `1`, analytic gauge residual `1.11e-16` and analytic/SVD
projector error `8.35e-13`. Gauge handling therefore conforms.

The Moore–Penrose equality particular has scaled residual
`6.204465308984932e-5` against the frozen `1e-9` bound and backward error
`1.3821783580600247e-5`. R127 stops `INVALID` before a gauge interval or cone
classification. It records one SVD and one particular solution, emits no cache,
and performs zero kinodynamic/candidate/PhysX/optimizer/training work.

Canonical/file/profile SHA-256 is
`255f2dd900f7ca67381fd6853aa42e47a991680f723b17539e9505651d6e7a4e` /
`0bdf21b2e995a3ea16f7670666a10b73379d6f6eda9dede04dea5c5e788e1a2b` /
`0e3537dd36e1a148788d6e4634e4409a01d10136033bf957f7a5d684699976c2`.
The canonical hash independently recomputes exactly.

## Static rigid-body discriminator

Let `d` be the unit world vector from heel to forefoot and `L=0.215 m`. For two
points fixed on one rigid body,

`a_fore - a_heel = alpha × (L d) + omega × (omega × (L d))`.

Taking the dot product with `d` removes the angular-acceleration term and gives

`d · (a_fore - a_heel) = -L ||omega × d||²`.

Therefore two zero world-point accelerations require `omega` to be parallel to
the heel–forefoot line. No generalized acceleration or contact-force split can
change this compatibility identity.

The frozen collocation reproduces:

| Quantity | Value |
| --- | ---: |
| Heel–forefoot separation | `0.215 m` |
| Foot angular velocity | `[-0.000458855, -0.131862991, -0.000290897] rad/s` |
| Perpendicular angular speed | `0.131848743768 rad/s` |
| Observed `d·(Jdot-v_fore-Jdot-v_heel)` | `-0.0037375796151256995 m/s²` |
| Predicted `-L||omega×d||²` | `-0.0037375796151256990 m/s²` |
| Absolute identity error | `4.336808689942018e-19 m/s²` |
| Heel/forefoot speed | `0.0991904304 / 0.1074934511 m/s` |

The nonzero value proves that the full two-point zero-acceleration RHS is
outside the rank-34 matrix range. The force gauge and RHS incompatibility are
distinct: R126 fixed the former correctly; it cannot fix the latter.

## Research context

- [MIT Underactuated multibody dynamics](https://underactuated.mit.edu/multibody.html)
  derives constrained acceleration as `H vdot + Hdot v = 0` and defines a
  sticking contact by zero relative tangential velocity. This supports treating
  q/v compatibility as a prerequisite, not as a force-selection detail.
- [Regularized time-discrete multibody dynamics](https://link.springer.com/article/10.1007/s11044-025-10107-8)
  states that the acceleration-level DAE is exact for initial positions and
  velocities compliant with the constraint, and separately identifies
  redundant constraints and regularization/softening.
- [Ruina and Pratap, Introduction to Statics and Dynamics](https://ruina.tam.cornell.edu/Book/RuinaPratap8-21-13.pdf)
  gives the relative rigid-body acceleration and centripetal term used by the
  no-solve discriminator.
- [Masarati, constraint stabilization](https://journals.sagepub.com/doi/10.1177/2041306810392117)
  describes projection enforcing derivatives of holonomic constraints through
  second order. It is a candidate family, not authority to modify R127.

## Repair families considered

| Family | What it repairs | Cost / risk | Current decision |
| --- | --- | --- | --- |
| Project q/v onto the selected contact manifold, then recompute fixed PD | Restores position/velocity/acceleration DAE consistency | Changes R120 state and R121 effort schedule; needs a new bounded formulation | Keep as primary discriminator |
| Reclassify FlatSticking to single-point/sliding/flight from frozen point velocity | Makes the mode honest for the current state | Changes mode inventory and cone ownership; may expose impacts | Keep as primary discriminator |
| Discrete-time or compliant contact | Accepts off-manifold state without an inconsistent rigid DAE | Changes rigid-contact semantics and must correspond to the native backend | Research, not a local patch |
| Baumgarte/stabilized acceleration RHS | Feeds position/velocity error back into closure | Introduces gains and does not automatically make redundant rows compatible | Do not tune before formulation |
| Delete one closure row while retaining two point cones | Makes the square algebra solvable | Hides an off-manifold velocity and breaks the frozen physical residual contract | Reject |
| Raise residual limits or retry R127 | Masks the incompatibility | Violates the consumed immutable gate | Reject permanently |

## R127-RC1 contract and result

The R127-RC1 implementation at clean commit `9d5cdbf` performs one static,
report-only audit. Its frozen contract requires it to:

1. bind the exact R127 report/profile/module/tool and clean commit;
2. reproduce only collocation `0` q/v, point identity and rigid-body kinematics;
3. compare the observed `Jdot-v` line projection with
   `-L||omega×d||²` under a predeclared tolerance;
4. audit both point velocities and distinguish force-nullspace correctness from
   RHS-range incompatibility;
5. perform zero SVDs, particular solutions, gauge classifications, local or
   kinodynamic solves, candidates, scenes and training runs;
6. stop invalid on any source, identity, nonfinite or tolerance mismatch.

The official audit is `COMPLETE` with finding
`CONFIRMED_FLAT_STICKING_STATE_ACCELERATION_INCOMPATIBILITY`. All six
validations pass (`305/305` lab, motor and full `host-check`), all ten
predeclared discriminators are true, and the rigid-line identity independently
reproduces the preliminary values. It performs one kinematic audit and zero
contact-Jacobian assemblies, local-system reconstructions, SVDs, particular
solutions, gauge classifications, solves, candidates, scenes or training runs.

Canonical/file/profile SHA-256 is
`a1028728e50050747c2167b45d76726aceebd24a7c881bd11bc9eb1e2c8dcc62` /
`a32f84719b72bf826255287209c596c4faba99646e16112f970acc3c4a5b63ba` /
`1e0137b3c1ba37cedd62da9f20e71616b8cc3a52d2f3e36df49ed3613549ec70`.
The canonical hash independently recomputes exactly.

The resulting gate permits only a separate report-only R128 contact-state
consistency formulation. R128 must compare q/v manifold projection, honest
contact-mode correction and explicit discrete/compliant contact, choose the
smallest falsifiable successor, and freeze its identities and acceptance
criteria. It may not execute the formulation, retry R127, build a candidate,
run PhysX or start training.

## R128 formulation result

Clean report-only R128 at commit `0220493` selects
`mass_metric_tangent_velocity_projection` as a Stage-2 diagnostic. At each
active collocation it freezes the unique minimum-kinetic-metric correction

`delta_v = -M^-1 J_A^T (J_A M^-1 J_A^T)^+ J_A v`

so that `J_A(q)(v+delta_v)=0`. It keeps `q`, modes, point identities, gains and
safety limits unchanged, but explicitly invalidates reuse of the old velocity,
`Jdot-v`, bias and fixed-PD effort bytes. Flat contact retains six rows, rank
five and one multiplier gauge; the projected velocity is nevertheless unique.

The alternative audit defers mode correction because the current schema has no
sliding mode, defers discrete/compliant contact until its timestep/material/
PhysX correspondence is explicit, and rejects Baumgarte or residual relaxation
as an R127 repair. R128 claims no position projection, `qdot=v`, integration,
impact, native behavior or candidate authority.

All six validations pass (`310/310` lab, motor and full `host-check`). The
official formulation performs zero state projections, matrix/Jacobian
assemblies, factorizations, solves, inverse-dynamics evaluations, candidates,
scenes or training runs. Canonical/file/profile SHA-256 is
`32a9e278f0c1afe74b646daaf9ef2e446e04e1e56c101caa4e8221cab76cd3d0` /
`15cba3557e487b3af33cb9b9b281501f90aea2fa662151dcbf99277bab46f56e` /
`4a6caece3113ad622ad8a2481bb07849b139eab626015f68511c66cabfe74035`.
The canonical hash independently recomputes exactly.

## R129 projection conformance result

Clean R129 at commit `b81270f` passes all seven frozen anchors. Six contact
projections have their exact predeclared ranks and nullities; maximum
active-point velocity after projection is `9.281e-16 m/s`, maximum scaled KKT
residual is `1.469e-14`, maximum closed-form/KKT disagreement is `3.775e-15`,
and every projection lowers kinetic energy. The three flat-foot anchors retain
one multiplier gauge while changing projected velocity by at most `4.072e-15`
under that gauge.

The collocation-0 rigid-line incompatibility falls from
`-0.003737579615 m/s²` to `-1.917e-18 m/s²`; across all flat anchors its maximum
post-projection magnitude is `2.277e-17 m/s²`. All synthetic rank-gap,
nonfinite, idempotence and same-body gauge cases pass. All six validations pass
(`318/318` lab, motor and full `host-check`). The official audit performs seven
mass assemblies, six contact-Jacobian assemblies/factorizations/solves, and
zero full-schedule projection, inverse dynamics or downstream work.

Canonical/file/profile SHA-256 is
`b34eb4727165e9b16ef82f597138fde33993c12efa8e3177776254237c6fbb99` /
`490b32f67c98d07d9daf0fe9c301372d69b8b85774227658b942b05210531829` /
`d25557c5a07cc243570c1a2b57d9ecb6c8c9ac12bdd31c1ec950f4cfbfc4a376`.
The canonical hash independently recomputes exactly.

R129 is consumed without retry. Its sole R130 successor is also consumed and
immutable `INVALID`: projection passes all `3200/3200` rows, but the recomputed
fixed-PD schedule violates right-ankle-roll speed/effort bounds before inverse
dynamics. Clean [R130 projected-schedule research](humanoid-train4-r130-projected-schedule-research-2026-08-15.md)
confirms two repeated right-forefoot exit hotspots while preserving the exact
row/event evidence boundary. Clean R131 selects a nine-exit mode-owned lift;
clean R132 passes `36/36`, lowers all `27` changed corrections and finds zero
local speed violations. Clean R133 passes all `3200` projections and the full
schedule with zero unsafe actuator categories. Clean R134 freezes its exact
q/v/effort with the gauge-aware reduced system and executes zero numeric
systems. Only report-only R135 implementation conformance is authorized;
R127/R129/R130/R132/R133/R134 retries, numeric R136 ID/kinodynamic execution,
candidate construction, PhysX and training remain forbidden.
