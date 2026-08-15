# TRAIN-4 R130 projected-schedule research — 2026-08-15

| Field | Value |
| --- | --- |
| Scope | Optimizer-free causal research after the sole R130 stopped before inverse dynamics |
| Status | `R130_INVALID / R130_RC1_COMPLETE / R131_FORMULATION_NEXT` |
| Confirmed boundary | `Numerically valid tangent projection creates an invalid fixed-PD actuator schedule` |
| Supported hotspot | `The four largest corrections cluster at two right-forefoot contact exits` |
| Claim ceiling | Static report-only research; no exact row/event identity, retry, projection, dynamics, candidate, PhysX or training |

## Immutable R130 result

The sole R130 process ran from clean commit `4dbd0ac`, passed all six
validations (`325/325` lab, `56/56` motor and full `host-check`), and consumed
its one execution authority. It executes exactly one `3200`-collocation
tangent-projected fixed-PD schedule derivation and stops
`INVALID / STOP_INVALID_EVIDENCE_WITHOUT_RESTART` with transition
`R130_CONSUMED_INVALID_NO_RETRY`.

The projection itself passes all `3200/3200` rows: `560` flight identities,
`324` single-point projections and `2316` flat-foot projections. All `2640`
active systems are assembled, factorized and solved exactly once. Maximum
active-point speed after projection is `1.2948e-14 m/s`, maximum scaled KKT
residual is `1.2490e-13`, maximum generalized-velocity correction is
`21.700328`, and kinetic energy never increases.

The recomputed controller schedule fails before the first inverse-dynamics
system:

| Schedule fact | R121 affine baseline | R130 projected velocity |
| --- | ---: | ---: |
| Hard-ROM violations | `0` | `0` |
| Target soft clamps | `0` | `0` |
| Target slew | `1`, DoF `6` | `1`, DoF `6` |
| Maximum target lag | `1672 µrad` | `1672 µrad` |
| Velocity violations | `0` | `4`, DoF `11` |
| Empty effort envelopes | `0` | `2`, DoF `11` |
| Static effort clamps | `0` | `7`, DoF `11` |
| Power clamps | `0` | `11`, DoFs `3,11` |
| Effort-rate clamps | `641` | `1169` |

Descriptor DoF `11` is `joint.right-ankle-roll`. Its authored maximum speed is
`8 rad/s`; the recorded excess reconstructs a peak absolute projected speed of
`21.712253 rad/s`. Its fixed actuator has `K=300`, `D=30`, `±140 N·m` effort,
`1400 N·m/s` rate and `500 W` power bounds. R130 therefore demonstrates an
actuator-input conflict, not inverse-dynamics cone infeasibility: it performs
zero inverse-dynamics assemblies, SVDs, particular solutions and force-gauge
classifications and emits no solver cache.

Canonical/file/profile SHA-256 is
`acd92a581a732c293cc9440d4215702fdf356f614c150ec4886d82a2fb197955` /
`422a9b54d44ca10297927ccccb54eb541b4fc23cc2c778c34e98d5b55d07ea72` /
`f10f557dd93d47f192f20d321052d309455cf54ba657bafc2829e7700fd2825f`.
The canonical hash independently recomputes exactly. The run used one process
and one observed OS thread, `30.714 s`, `939544576` bytes peak RSS, no restart
and no manual intervention.

## Hotspot audit and evidence boundary

R130 stores every row's projection scalars and state hashes, but deliberately
does not store projected vectors or controller-event row indices. R130-RC1 can
therefore rank existing corrections and classify adjacent contact transitions;
it cannot reconstruct a projected state or claim an exact row-to-event match.

| Rank | Collocation | Interval/substep | Mode / point | Next interval | Max correction | Active speed before |
| ---: | ---: | --- | --- | --- | ---: | ---: |
| 1 | `307` | `76/3` | `[0,2]`, right forefoot `3` | flight `[0,0]` | `21.700328` | `1.478900 m/s` |
| 2 | `2979` | `744/3` | `[0,2]`, right forefoot `3` | flight `[0,0]` | `18.336225` | `1.157680 m/s` |
| 3 | `306` | `76/2` | `[0,2]`, right forefoot `3` | flight `[0,0]` | `14.557628` | `1.010665 m/s` |
| 4 | `2978` | `744/2` | `[0,2]`, right forefoot `3` | flight `[0,0]` | `12.829865` | `0.798604 m/s` |

The next-largest correction is only `7.483665`. Thus the four largest values
form two repeated pairs at substeps `2/3` immediately before right-forefoot
liftoff. This is strong localization evidence, not proof that those exact four
rows are the four velocity violations.

The smallest current causal model is that R121 first derives a hybrid `60 Hz`
knot velocity, including a one-sided contact-exit stencil, but then lifts q and
v independently and affinely to four `240 Hz` substeps while retaining the
left-knot contact mode. Near liftoff, that mixes a flight-side knot velocity
into rows still declared sticking. The tangent projection must then make an
increasingly large last-moment correction under the old mode, and the fixed-PD
actuator envelope is not part of its objective. R130-RC1 supports this model at
the hotspot level but does not yet prove it by an explicit alternative lift.

## Research basis and alternatives

- [Gear, Leimkuhler and Gupta](https://doi.org/10.1016/0377-0427(85)90008-1)
  establish projection as a constrained-DAE consistency mechanism. That does
  not imply preservation of separate actuator inequalities.
- [Escande, Mansard and Wieber](https://gepettoweb.laas.fr/uploads/Publications/2014_escande_ijrr.pdf)
  show why conflicting whole-body equalities and inequality regions need an
  explicit hierarchy or joint optimization.
- [Posa, Cantu and Tedrake](https://groups.csail.mit.edu/robotics-center/public_papers/Posa13.pdf)
  treat contact creation and breakage as hybrid trajectory dynamics rather than
  one globally smooth interpolation assumption.
- [Wang et al.](https://arxiv.org/abs/2006.01987) explicitly account for
  contact-induced velocity and torque jumps in a hardware-feasible QP.
- [Kinodynamic Motion Retargeting](https://arxiv.org/html/2603.09956) reports
  that kinematic retargeting can leave contact and velocity artifacts and uses
  trajectory-level q/v/a, torque and contact-force constraints as the stronger
  fallback family.

| Repair family | What it can answer | Current decision |
| --- | --- | --- |
| Contact-side one-sided `240 Hz` state lift, then conformed tangent projection | Directly tests the two exit-boundary hotspots while keeping q, modes, gains and limits fixed | Primary R131 report-only formulation |
| Instantaneous box/actuator-constrained tangent projection | Can include a joint-speed inequality in one row | Secondary: it does not by itself close cross-mode `qdot`, effort-rate, power or work coupling |
| Full kinodynamic retargeting over q/v/a/effort/contact force | Jointly closes integration, contact and actuator feasibility | Strong fallback after the smaller discriminator |
| Sliding/compliant/impact successor mode | May describe the physical transition more honestly | Requires a separate contact-semantics/native-correspondence decision |
| Raise limits, reduce damping, loosen the gate or retry R130 | Hides or changes the frozen safety/controller contract | Rejected permanently |

## R130-RC1 contract and result

Clean report-only R130-RC1 at commit `74275e3` binds the immutable R121/R130
reports and source identities, the current descriptor, the four predeclared
hotspots and the exact evidence boundary above. It performs one static audit
and one descriptor-channel lookup. It reconstructs or solves no projection or
dynamics system and performs zero candidate, PhysX, optimizer or training work.

All twelve predeclared discriminators pass. The official finding is
`CONFIRMED_PROJECTED_SCHEDULE_ACTUATOR_CONFLICT_WITH_HYBRID_EXIT_HOTSPOTS` and
the gate decision is
`PERMIT_SEPARATE_REPORT_ONLY_R131_HYBRID_CONTACT_EDGE_STATE_LIFT_FORMULATION_ONLY`.
All six validations pass (`330/330` lab, `56/56` motor and full `host-check`).

Canonical/file/profile SHA-256 is
`edc8f978e6ee068c32637fe2000495c066daafdd15458eb3809365d69446dd3d` /
`68166bcf90d9d77d82e92faad159ee3653259da56bf66fe3020a927c174abc2b` /
`7794c56283e9703f710682ec56b48689db299cbe7788b5ccbdd5653b631cd7eb`.
Research-module/tool SHA-256 is
`15d4bea0dd4d81cd48d1682a6b6b460b942250bf987a93256fc387775e983573` /
`284535f5f644b55c45cf88217b398f4f6c127239728edfcf5838d1422fc32fa7`.
The canonical hash independently recomputes exactly.

## Next bounded action

R131 must be a separate report-only formulation. It must compare the current
affine cross-mode lift, a contact-side one-sided hybrid lift, an instantaneous
actuator-constrained projection and full kinodynamic fallback; define the exact
q/v/`qdot`/mode ownership at entry and exit edges; freeze fail-closed generated
tests and identities; and choose the smallest falsifiable successor. It may
not execute a state projection or inverse-dynamics system, rebuild R130,
construct a candidate, run PhysX or start training.
