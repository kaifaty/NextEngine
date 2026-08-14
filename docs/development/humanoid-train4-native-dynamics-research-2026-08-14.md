# TRAIN-4 native-dynamics research decision — 2026-08-14

| Field | Value |
| --- | --- |
| Scope | Optimizer-free research after complete-clip V9 fresh-scene rejection |
| Status | `R96_CONTACT_QUALIFIED / DERIVATIVE_MECHANISM_REJECTED` |
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
| R95 V7↔V9 differential audit | canonical/file SHA-256 `ec453b347e9323819ba706daa40949a91924e7a359c9225ab7ce031dcf60bd6f` / `8287e3ef751d22a46cc312d1dc7a79f3245302a2fa4d3da965cd7e5efd652f76` | `COMPLETE`; selects ordinals `2` and `10` for at most two fresh counterfactuals |

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
| H22 | V9 satisfies pose/velocity geometry but asks the fixed PD plant for dynamically infeasible acceleration or effort | R95 finds `8.3576x/32.7269x` acceleration/jerk amplification in new ROM regression `10` | R96 derivative-only construction, then fresh ordinal `10` |
| H23 | Near-zero offline collider clearance does not predict the full PhysX contact manifold and impulse | R95 isolates ordinal `2`: active foot `1539 µm` above surface with no derivative amplification; ordinal `7` flight foot is only `49 µm` high | R96 contact-reserve-only construction, then fresh ordinal `2` |
| H24 | Complete-clip corrections are nonlocal and regress safe controls while repairing old failures | R95 finds four new regressions and derivative hotspots shared across overlapping windows | R96 locality audit plus fresh ordinal `10` |
| H25 | Fresh-scene initialization is responsible | Initial state is exact within quantization in every worker | Falsified by R94; do not repeat partial-reset experiments |

## R95 differential-audit result

R95 ran from clean commit
`6491d241748cfd1a5080f7866186507f37e7c9bc`. Profile SHA-256 is
`11e5eb851f1a5cecb29f9d7796dee7e8b5f7e51104a1e9ed00b3c65285b7fbba`.
It is report-only: PhysX runs, trajectory mutations, optimizer steps and
training runs are all zero.

The four new control regressions have median V9/V7 joint-acceleration ratio
`6.8865x` and jerk ratio `21.8469x`. The three persisting/changed source
failures are higher still at `8.3367x` and `34.3599x`. Stable passing controls
also become rougher (`1.7454x` / `3.3520x` median), so amplification alone is
not sufficient; failure timing and contact state remain required discriminators.

Two cases isolate the competing mechanisms:

- ordinal `2`, `cmu05@25`, has no acceleration amplification (`1.0000x`) and
  only `1.0375x` jerk, yet its declared flat left support begins `1539 µm`
  above the collider surface; passing V7 begins at `-2 µm`. R94 impacts that
  left ankle at tick `2`. This is the contact-geometry discriminator.
- ordinal `10`, `cmu16@238`, has no impact and fails hard ROM at tick `10`.
  V9 reaches `78001200 µrad/s²` right-hip-pitch acceleration at frame `245`
  (`8.3576x` V7) and `9073080000 µrad/s³` jerk at frame `246` (`32.7269x`).
  This is the derivative/nonlocal-correction discriminator.

Ordinal `7` independently shows both mechanisms and is therefore not the first
counterfactual: its right flight foot starts only `49 µm` above ground versus
`13513 µm` in V7, while acceleration/jerk also rise `5.4956x/11.6804x`.

R95's fixed-PD quantities remain explicitly labelled proxies. They rank
experiments but are neither native torque traces nor feasibility proof.

## R96 independent offline results

Both frozen counterfactual profiles and their solver support were implemented
at clean commit `14c322ab7db44bdc9dad2e7f5b5e2e3b9476ae09`. The complete
lab suite passed `172/172`; optimizer steps, training runs and PhysX runs were
zero. The variants were evaluated independently and are not a bundled fix.

### Contact reserve: qualified offline

The contact-only profile has SHA-256
`289133123584fc1f228839731e5df405aacfb0deb6c7aca6cc692dbbe96c8a58`.
It changes only the internal normal-residual margin from `100` to `4500 µm`;
the public residual remains `5000 µm`, the collider floor remains `-2 µm`, and
all contact, ROM, velocity and controller identities remain unchanged.

The clean `cmu05` complete-clip build passes in two SQP iterations. Its
canonical/file manifest SHA-256 is
`376fedff6f8a6a327c2f99f6700d82a338e7117e310d26c36e92b154f9c691eb` /
`08b28cea1ebd69ebfc5bff88f6f17936cb98caa86bc8fb4661b1a4e2cc13fab9`.
The complete clip retains zero contact-point deletion, maximum normal residual
`510 µm`, finite tangential/normal steps `1982/426 µm`, analytic
tangential/normal steps `1970/998 µm`, collider minimum `+49 µm`, joint
velocity `2500 bp` and root vertical velocity `199770 µm/s`.

On ordinal `2`, the left active-support clearance moves from `1539` to
`496 µm` (`67.8%` reduction), maximum normal residual is `506 µm`, and the
contact modes and contact bytes are unchanged. This variant qualifies for its
single fresh-scene R97 discriminator, but that run is held until the independent
derivative variant also qualifies.

### Derivative regularity: two rejected mechanisms

An exploratory first-difference-only run changed the existing coefficient from
`0.01` to `1.0`. It passed the unchanged offline bounds, but case `10`
acceleration increased from `78001200` to `80047800 µrad/s²` (`+2.62%`) while
jerk fell only to `7404696000 µrad/s³` (`-18.39%`). It emitted no frozen
candidate/report and is non-promotable; no coefficient sweep followed.

Read-only localization then found that the right-hip-pitch correction falls
from `47446` to `505 µrad` across frames `244 -> 245` and rebounds to
`4398 µrad` at frame `246`, exactly when the right forefoot changes from
sticking to flight. This produces the R95 `-78001200 µrad/s²` acceleration at
frame `245` and `9073080000 µrad/s³` jerk at frame `246`.

The clean second-difference profile has SHA-256
`8d7a9dbc8f3d080891a8ec0ba4db38dd5ae73274888b559ff26e1303fbcd129e`.
It adds one dimensionless correction-curvature coefficient of `1.0`, equal to
the frozen correction-magnitude regularization, with no sweep or other change.
The `cmu16` complete clip and exact case `10` slice pass every unchanged
offline bound with zero mode/contact disagreement and zero changes in
unselected joint channels. Canonical/file manifest SHA-256 is
`ac7b180fcd429edc4360b4187837002472202717c39d2b7fbebf6f9490af2931` /
`0c61860517cfe6c6f87507203b4ec8251c193b514d88e9134b429149b8bd4ae0`.

It nevertheless fails the causal metric: jerk improves to
`6254388000 µrad/s³` (`-31.06%`), but acceleration worsens to
`79126200 µrad/s²` (`+1.44%`). The correction is smoother in pose space, yet
the frozen emitted-velocity stencil changes from backward difference on the
last contact frame to centered difference in flight. The objective therefore
still optimizes a proxy rather than the emitted acceleration seen by fixed PD.
Ordinal `10` is not authorized for R97 from this artifact.

The next derivative experiment must operate directly on the first difference
of the emitted hybrid-stencil velocity, using one predeclared dimensionless
coefficient and no grid search. It must strictly reduce both acceleration and
jerk, preserve byte-identical modes/unselected channels and pass all unchanged
offline limits before fresh PhysX is considered.

## R96/R97 bounded counterfactual contract

R96 may construct two independent report-only variants, not one bundled fix:

1. contact reserve only for ordinal `2`, moving the declared active support
   toward physical contact inside the unchanged `5000 µm` public residual and
   `-2 µm` collider bounds;
2. derivative regularity only for ordinal `10`, now acting on emitted hybrid-
   stencil acceleration after correction-curvature regularization was rejected,
   without changing velocity, ROM or PD limits.

Each variant must pass the unchanged offline audit and show that its intended
metric decreases without silently changing contact modes or unrelated
trajectory channels. R97 may then run exactly those two fresh cases. It is a
causal discriminator, not all-17 acceptance or corpus evidence.

## Decision

Freeze R92–R95 and both clean R96 artifacts. Retain the qualified contact
variant, reject first- and second-difference-only derivative tuning, and do not
begin training. Construct one direct emitted-acceleration variant for ordinal
`10`; only if it qualifies may R97 run ordinals `2` and `10`. A full all-17
rerun is allowed only if both causal counterfactuals pass without changing
controller semantics, safety limits, fresh-scene authority or the exact-zero
gate.
