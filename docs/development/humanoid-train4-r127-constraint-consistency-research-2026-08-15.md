# TRAIN-4 R127 contact-state consistency research — 2026-08-15

| Field | Value |
| --- | --- |
| Scope | Optimizer-free causal research after the sole R127 stopped before its first gauge/cone classification |
| Status | `R127_INVALID / R127-RC1_COMPLETE / R128_FORMULATION_NEXT` |
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
