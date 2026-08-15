# TRAIN-4 R130 projected-schedule research — 2026-08-15

| Field | Value |
| --- | --- |
| Scope | Optimizer-free causal research after the sole R130 stopped before inverse dynamics |
| Status | `R130_INVALID / R130_RC1_COMPLETE / R132_PASS / R133_PASS / R134_COMPLETE / R135_CONFORMANCE_NEXT` |
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

## R131 formulation result

Clean report-only R131 at commit `9f54f3a` audits all `3200` stored R130
collocation metadata rows and freezes the contact-edge inventory: `799`
boundaries contain `770` unchanged modes, `9` contact entries, `9` complete
contact exits and `11` active-mode changes. No boundary changes both feet at
once. Intervals `76/744` are confirmed as two of the five right-forefoot exits.

R131 selects `exit_mode_owned_left_velocity_trace_hold` as the smallest causal
discriminator. Only on the nine complete exit intervals, substeps `1/2/3`
replace affine endpoint weights `[4-s,s]/4` by `[4,0]/4`; substep zero and the
next interval's exact knot remain unchanged. Thus at most `27/3200` base-
velocity rows change before the already-conformed projection. Configuration,
SO(3) interpolation, modes, points, targets, gains and limits remain exact.

This is explicitly a diagnostic, not a production integrator: `qdot=v`, a
discrete integration constraint and release impulse are not claimed. The
instantaneous actuator-constrained projection remains secondary because it
does not close sequential rate/power/work state; full trajectory-level
kinodynamic retargeting is the stronger fallback if the local discriminator
fails.

All six validations pass (`336/336` lab, `56/56` motor and full `host-check`).
R131 performs one metadata inventory and one formulation, with zero state-lift
evaluations, matrix/Jacobian assemblies, projections, controller schedules,
inverse dynamics, candidates, scenes or training. Canonical/file/profile
SHA-256 is
`480d28d4b9c90552cc0b42091a26f354118d56fe6b9292777c9813aa8b490696` /
`4a8e6c1bece5269c608a6da3b901ebd868a1a8de0b7f525835706d76f66a9a3d` /
`005b1cddee646064cb4c89a076ac7fe929c1efb84b6dbaf7ef104e7f550543cf`.
Formulation-module/tool SHA-256 is
`1d696a4e6a8408d93b18e869679b4736f6977ffef70be7fc0ea734709a3506af` /
`2a1c6075fe919a0dcb5a32f3e49b7b952efe0e9b69c96f7c3488b4eeded28779`.
The canonical hash independently recomputes exactly.

## R132 conformance result

Clean report-only R132 at commit `5c4cb55` evaluates exactly the predeclared
`9 × 4 = 36` exit rows. It preserves configuration, mode, active point and the
position-only target lineage, replaces exactly the expected `27` substeps
`1/2/3` by the left-knot velocity, and passes all `36/36` rank-three R129
projection guards. Maximum active-point velocity after projection is
`5.551e-16 m/s`, maximum scaled KKT residual is `1.762e-14`, and maximum
idempotence error is `3.553e-15`.

The causal discriminator is positive. Every one of the `27` changed rows has
a strictly smaller generalized-velocity correction than its row-matched R130
baseline. Across the bounded set, the maximum falls from `21.700328` to
`1.950192 rad/s`. At the late hotspot rows `307/2979`, the correction ratios
are `0.02856/0.10636`; projected right-ankle-roll speeds are
`0.619858/1.950192 rad/s`, both below the authored `8 rad/s` limit. All
`36` selected rows have zero descriptor joint-velocity violations.

This is evidence for cross-mode flight-velocity contamination as the cause of
R130's late correction, not yet proof of controller feasibility: R132 does not
carry sequential effort-rate, power or work state. All six validations pass
(`343/343` lab, `56/56` motor and full `host-check`). R132 performs exactly
`36` state-lift evaluations, mass/Jacobian assemblies and projections, with
zero full-schedule projection, controller derivation, inverse dynamics,
candidate, PhysX or training work. Canonical/file/profile SHA-256 is
`9afd566adaa81de573462bf083948f8b1f49b0fc023c76726afb403e906e19d6` /
`657ed7216f7fcf59a0f1e5b06e6068f4383fe1425418922ed3e115c36517d2c5` /
`87b09fdf857c8b096898536f34909f5ba8c8eff3ffdb42f8baa61e6afa35937a`.
Conformance-module/tool SHA-256 is
`510a84a8dc46b6af5ec7e03f4419b9c6a08043c0ed3b67f0bdffc874e59f151b` /
`95a2e75e091075b81d7c789537c28f7a48ad27730281ccfd9a4ff244301c7a4c`.
The canonical hash independently recomputes exactly.

Clean report-only R133 at commit `03f8e0b` consumes that authority and passes.
All `3200/3200` rows pass (`2640` contact projections, `560` flight
identities); the exact R132 anchor hashes reproduce, and the full fixed-PD
schedule has zero hard-ROM, joint-velocity, static-effort, power,
positive-work or empty-effort-envelope violations. Its `1011` row-addressed
events are exactly `1010` admissible effort-rate clamps and one target slew.
Canonical/file/profile SHA-256 is
`f1fad2ca3c7abd49acaefd9fcd37873d02fb1d289a3081fd2039b0b1192fd3b6` /
`2ddef1cfae193eb0e32f4d001aba6a8b744056b700fa2ecbe533434bc8e5f7c2` /
`20cccf2ffd413ae56148b95ca4cbd67c616f1ee396f2bee995a078ab1685a6a7`.
R133 performs zero inverse-dynamics, kinodynamic, candidate, PhysX or training
work. Clean report-only R134 at `5a4085b` then freezes the exact R133 projected
q/v/effort plus R126 gauge-aware `29/32/35` composition. It accounts for
`107668` equality rows, `4956` point cones and `2316` force gauges while making
R120 acceleration scale/diagnostic-only. All validations pass (`354/354` lab,
`56/56` motor, host); every real state/projection/schedule/ID/downstream counter
is zero. Canonical/file/profile SHA-256 is
`17d696166a05a24bd50a545ca31e4aabd15a72ed49444a09983c472ff6b13783` /
`0f7ccdcdcb4fbcf1b3e73e1e8f5565b0387c15a4bd1cfa3c50fbd6754ba176be` /
`7fafc5e874f5b5cd23e5e7720133640e1f84d7dad8c233f4ba384fc97d3cb4b9`.
Only report-only R135 implementation conformance is authorized; R136 numeric
inverse dynamics and every downstream action remain forbidden.
