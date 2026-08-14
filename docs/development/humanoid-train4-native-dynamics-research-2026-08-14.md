# TRAIN-4 native-dynamics research decision — 2026-08-14

| Field | Value |
| --- | --- |
| Scope | Optimizer-free research after complete-clip V9 fresh-scene rejection |
| Status | `R94_FRESH_FAIL / STOP_AND_RESEARCH / R95_DIFFERENTIAL_AUDIT_NEXT` |
| Acceptance authority | Fresh scene under ADR-070 |
| Claim ceiling | Research and generated-test design only; no corpus admission or training |

## Why this cycle exists

R93 proves a narrow but useful fact: one deterministic V9 trajectory exists for
each selected complete clip and satisfies every frozen *offline kinematic*
constraint. R94 then disproves the stronger hypothesis that those trajectories
are dynamically reproduced by the accepted PhysX environment and its frozen
zero-residual fixed-PD controller.

This is a stage boundary, not a reason to weaken the gate. The work returns to
bounded research before another solver change or expensive native run.

## Immutable evidence

| Evidence | Identity | Result |
| --- | --- | --- |
| R49 V7 fresh all-17 | report file SHA-256 `6b977a870c50b25545c2b73bc371575b38b40f6a914d1638ab19a3c7a56b5f0c` | `PASS 17/17`, required safety `0`, passing-control regressions `0` |
| R93 V9 complete clips and exact slices | canonical/file SHA-256 `7ee041b7710302b9fae909b0cb34c0de3a6c2df257032cd0e1c7dcaf16afefeb` / `53984129cc45d295ced9ef4f49ea8532d0b224c3ccc19980e8b986cee7604d70` | All three clips and all 17 slices pass offline; `390` arrays over `30` overlap pairs agree byte-for-byte; no contact point is deleted |
| R94 V9 fresh all-17 | canonical/file SHA-256 `0d429faff356e3240a7f2906115566b565fc0800043dd7ffa04544b2edb38925` / `4aab74888d50fae2ab644445597d3f7f7a64217f078ceca4b4f8045ed34f0929` | `FAIL 7/17`, `gate_decision=STOP_AND_RESEARCH` |

R94 is bound to clean repository commit
`5cedc41d23958023f7b4d7dcee46c34f2f230b73`, R93, the unchanged source
corpus, descriptor, reference-tracker profile, gate report and USD. It starts
17 separate worker processes and creates one fresh scene for every case.
Indexed partial reset is `NOT_RUN` and remains report-only. Optimizer steps and
training runs are both zero.

## R94 result

R94 fails seven required cases:

| Ordinal | Slice | First failing tick | Required-safety reason |
| ---: | --- | ---: | --- |
| 2 | `cmu05@25` | 2 | left-ankle hard impact, `6440089 µN·s` |
| 7 | `cmu139@626` | 1 | right-ankle hard impact, `6978856 µN·s` |
| 10 | `cmu16@238` | 10 | right ankle-pitch hard-ROM excess, `13193 µrad` |
| 11 | `cmu16@239` | 9 | right ankle-pitch hard-ROM excess `11805 µrad` plus right-ankle impact `6144072 µN·s` |
| 12 | `cmu16@241` | 8 | right ankle-pitch hard-ROM excess, `12113 µrad` |
| 14 | `cmu16@407` | 8 | left ankle-roll joint-safety/velocity excess, `985768 µrad/s` |
| 15 | `cmu16@409` | 8 | right ankle-pitch hard-ROM excess `11126 µrad` plus right-ankle impact `6003242 µN·s` |

The aggregate event counts are hard impact `4`, hard ROM `4`, joint safety
`1` and joint velocity `1`; every other required category is zero. Four cases
that were passing controls in the source discriminator regress: ordinals
`2`, `7`, `10` and `14`. Targeted failure counts therefore do not strictly
decrease for impact (`3 -> 4`) or ROM (`4 -> 4`); velocity improves only
partially (`2 -> 1`).

At the authoritative post-reset observation, the worst initial-state mismatch
over all 17 workers is only `1 µm` root position, `0 µm/s` root linear
velocity, `1 µrad/s` root angular velocity, `0 µrad` joint position and
`0 µrad/s` joint velocity. The minimum root-orientation dot is
`1073741823` in Q1.30. This rejects reset-state authorship mismatch as the
explanation for R94.

The failure timing also rejects one uniform first-tick contact explanation:
two impacts happen at ticks `1/2`, while ROM, velocity and additional impacts
appear at ticks `8..10` after the physical state has diverged from the
reference.

## Controller and architecture boundary

The accepted zero-residual path sends the current reference joint position
through slew limiting and applies the fixed explicit law
`K(q_des - q) - D*q_dot`. It has no desired-velocity term and no inverse-
dynamics or gravity feed-forward term. This matches the Isaac Lab description
of [position control with fixed impedance](https://isaac-sim.github.io/IsaacLab/v2.2.0/source/overview/core-concepts/motion_generators.html#position-control-with-fixed-impedance):
the simple controller assumes zero desired joint velocity, while a dynamics-
aware formulation additionally uses the inertia matrix and gravity vector.

This observation does **not** authorize a controller change. SPEC-35 and
ADR-070 freeze the current reference-tracking path for this gate, and the
earlier causal matrix already rejects feed-forward and velocity-zeroing as
primary fixes. The reference must be feasible for the accepted controller, or
the architecture must be changed separately through its own decision process.

## Primary-source research synthesis

The external literature supports treating this as a kinodynamic-reference
problem rather than another purely geometric tolerance problem:

- [KDMR](https://arxiv.org/abs/2603.09956) reports that purely kinematic
  retargeting can leave physically inconsistent artifacts and adds rigid-body
  dynamics, contact complementarity and ground-reaction-force information.
- [SPARK](https://arxiv.org/abs/2603.11480) progressively applies kinematic
  trajectory optimization, inverse dynamics and full kinodynamic optimization,
  producing both dynamically consistent state trajectories and torque
  profiles.
- [Multi-Contact Motion Retargeting](https://arxiv.org/abs/2206.00542) couples
  whole-body kinematics with sequential force equilibrium to obtain physically
  viable multi-contact motion.
- [DeepMimic](https://arxiv.org/abs/1804.02717) is evidence that a learned
  physics controller can correct example motion, but it is not authority to
  bypass TRAIN-4 or start PPO while the reference gate is open.

The inference for NextEngine is deliberately smaller than those methods: first
measure whether V9 introduced acceleration, implied-effort or contact-transition
changes that distinguish the R94 failures from R49's passing V7 trajectories.
Only then choose a bounded construction change.

## Ranked hypotheses

| ID | Hypothesis | Current evidence | Discriminator |
| --- | --- | --- | --- |
| H22 | V9 satisfies pose/velocity geometry but asks the fixed PD plant for dynamically infeasible acceleration or effort | R94 starts from the correct state yet diverges into late ROM/velocity failures; the offline solver has no equation of motion or effort proxy | R95 V7↔V9 position, velocity, acceleration, jerk and frozen-PD implied-effort audit over all 17 slices |
| H23 | Near-zero offline collider clearance does not predict the full PhysX contact manifold and impulse | Tick-1/2 impacts occur despite offline collider/contact PASS; case 2 already shows actual foot height drifting through the reference surface | R95 contact-entry/clearance deltas, followed only if needed by one or two fresh counterfactual cases with extra geometric reserve inside unchanged public bounds |
| H24 | Complete-clip corrections are nonlocal and regress safe controls while repairing old failures | V9 repairs source failures at ordinals `1`, `4`, `5`, `8` but creates control failures at `2`, `7`, `10`, `14` | R95 frame-local correction magnitude and derivative comparison around every selected window |
| H25 | Fresh-scene initialization is responsible | Initial state is exact within quantization in every worker | Falsified by R94; do not repeat partial-reset experiments |

## R95 bounded differential-audit contract

R95 is report-only and must not run PhysX, mutate a trajectory, build V19 or
start training. It compares the hash-bound R47 V7 and R93 V9 exact-slice
artifacts for the same ordered 17 cases and records, per frame and case:

1. root/joint position and velocity deltas;
2. maximum joint/root speed, acceleration and jerk;
3. target-step and initial damping-load proxies for the frozen fixed-PD law;
4. contact-entry and foot-collider-clearance changes;
5. grouped summaries for R94 passes, old failures repaired by V9, persisting
   failures and newly regressed controls.

The report must bind all inputs by SHA-256 and separate direct measurements
from proxies. A proxy can select a small native discriminator; it cannot prove
dynamic feasibility or change an accepted requirement.

## Decision

Freeze R92–R94. Do not retune V9 blindly and do not begin training. Execute R95
first. After R95, choose the smallest one- or two-case fresh discriminator that
separates H22 from H23/H24. A full all-17 rerun is allowed only after that
counterfactual passes without changing controller semantics, safety limits,
fresh-scene authority or the exact-zero gate.
