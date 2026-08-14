# TRAIN-4 native-dynamics research decision — 2026-08-14

| Field | Value |
| --- | --- |
| Scope | Optimizer-free research after complete-clip V9 fresh-scene rejection |
| Status | `R97_FAIL_1_OF_2 / R98_NATIVE_TRACE_NEXT` |
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
| R96 direct emitted-acceleration candidate | canonical/file SHA-256 `94981a8b14b48c6ad01c331676fee8a4a24e9aa764a1cbdf13c090d524aaa2ae` / `d4fe0ac9aec9ba56b7ffed6ac097ba205f1ee8afb3c38eb7570ec7034189d46c` | Offline `PASS`; case `10` acceleration `-42.70%`, jerk `-54.67%`, modes and unselected channels byte-identical |
| R97 two-case fresh discriminator | canonical/file SHA-256 `63d1331529905bd25354884331973bfed4578eb061283c40f7639b31a2f4bfcd` / `6b7ac6a1f2dad49f2c85ebbe5139859ab855549c905e86af2bdfcce4b6b0ac62` | `FAIL 1/2`; contact case passes, derivative case regresses to a new joint-velocity reason; `gate_decision=STOP_AND_RESEARCH` |

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
| H22 | V9 satisfies pose/velocity geometry but asks the fixed PD plant for dynamically infeasible acceleration or effort | R96 V11 lowers the selected emitted acceleration/jerk; R97 no longer reaches the old tick-10 ROM event before termination | Only partially supported: a new left-ankle-roll velocity event terminates at tick `9`; R98 traces native state/effort at every physics substep |
| H23 | Near-zero offline collider clearance does not predict the full PhysX contact manifold and impulse | R95 isolates ordinal `2`: active foot `1539 µm` above surface with no derivative amplification; R97 reserve case passes all `11` ticks and lowers peak left-foot impulse `6440089 -> 4466405 µN·s` | Supported for the selected case only; broader contact cases remain untested |
| H24 | Complete-clip corrections are nonlocal and regress safe controls while repairing old failures | R97 changes the failing channel from right ankle-pitch ROM to left ankle-roll velocity although contact modes and the direct left-ankle-roll reference are effectively unchanged | Supported as cross-chain coupling; R98 must distinguish contact transfer, effort limiting and substep velocity overshoot |
| H26 | Motor-frame pose/derivative summaries hide the causal physical substep | R97 reports the maximum substep velocity excess but stores only an all-joint maximum at the motor boundary, not the failing channel's physical-substep path | R98 captures all action-channel state, targets, requested/published effort and cumulative contact impulse at each 240 Hz substep |
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

### Direct emitted acceleration: qualified offline

Clean commit `03175f9cb0fef2afed9f80289231496063217dec` adds the exact
sparse acceleration operator induced by the same frozen hybrid velocity
stencil used by the emitted reference. The profile SHA-256 is
`80334fd709d25054d5e6e7bb61606d26e89cec0ed685e974e4dec5e4bb43ddbf`;
its single predeclared dimensionless coefficient is `1.0`, with no sweep.
The full lab suite passes `173/173`.

The clean `cmu16` complete-clip build passes in two SQP iterations with zero
contact-point deletion. It retains maximum residual `4914 µm`, finite
tangential/normal steps `1981/984 µm`, analytic steps `1980/997 µm`, collider
minimum `+49 µm`, joint velocity `2500 bp` and root vertical velocity
`199770 µm/s`. On exact case `10`, peak emitted acceleration falls
`78001200 -> 44697600 µrad/s²` (`-42.70%`) and jerk falls
`9073080000 -> 4112424000 µrad/s³` (`-54.67%`). Contact modes, contact bytes
and unselected joint channels are byte-identical. The case artifact SHA-256 is
`4cac5ffc36c227fd71d5245d282c218cfd948ecdfe632e516ab92cac35a00b63`.

This satisfies the predeclared offline causal metric and admits only case `10`
to the two-case R97 discriminator. It is not dynamic-feasibility proof.

## R97 fresh-scene result

Commit `590bb9b5cf1b9848b37420b4b254600482b660d2` freezes the two exact
counterfactual artifacts in a hash-closed bundle. Its canonical/file SHA-256 is
`02d502877e94e1d9d5402fd29fb624c6e5545e94d1fc141934d585abe558466d` /
`69d5a29010754935fbdb8f221cf5e36000c2c4ff33ae35d1b690c07df958b58c`.
The R97 probe profile SHA-256 is
`1d91f7c63f7bf98108175ae2b62365ba5fe6e577854bc780253db82ddd272cd6`;
the run uses two independent fresh worker processes from clean commit
`3b0840110c72c74eaa1da25c2c0b8558d9724b4c`. Indexed partial reset is
`NOT_RUN`; optimizer steps and training runs are zero.

The contact-reserve case `cmu05@25` passes all `11` motor ticks. Its maximum
left-foot impulse is `4466405 µN·s`, below the unchanged hard limit and below
R94's tick-2 failure `6440089 µN·s`. Reference clearance stays
`487..505 µm`; observed support contact is present throughout. This supports
H23 for this one selected case but does not authorize the other R94 contact
failures.

The emitted-acceleration case `cmu16@238` fails at tick `9`, frame `247`, on a
new `joint_safety/joint_velocity` event in `actuator.left-ankle-roll` with
`775377 µrad/s` excess. The old right-ankle-pitch hard-ROM event occurred at
tick `10`, so the earlier termination does not prove that it would remain
closed. Reference contact modes and observed contact bytes match the R94 V9
run through the shared prefix, yet the actual left ankle-roll state diverges
strongly before the flight transition. The direct left-ankle-roll target is
nearly unchanged, making this evidence of whole-chain physical coupling rather
than a local target violation.

R97 therefore rejects the merged candidate and authorizes neither all-17,
full V19 nor training. Its explicit decision is `STOP_AND_RESEARCH`.

## R98 bounded native-trace contract

R98 is report-only instrumentation, not another trajectory variant. It replays
the exact failed case with zero residual and unchanged fresh-scene lifecycle,
controller, limits and artifacts. For every 240 Hz physics substep it records
all action-channel positions/velocities, reference and slew-limited targets,
canonical requested and published effort, effort-envelope state and cumulative
contact impulses. A repeated passing contact case remains the instrumentation
non-regression control. The trace must be hash-bound to a clean commit and must
not authorize a merge, all-17, V19 or training even if a replay happens to pass.

The discriminator is causal order: determine whether the left-ankle-roll speed
crosses its limit before or after contact transfer, whether requested/published
effort is clipped, and which upstream target/state divergence precedes it.
Only then may one smallest hypothesis-specific construction be proposed.

## Decision

Freeze R92–R97, retain the contact result as bounded support for H23, and reject
the R96 V11 candidate as a merged/full-corpus direction after its R97 control
regression. Do not tune another derivative coefficient or begin training.
Instrument and run R98 under the contract above, then choose the smallest
counterfactual that its causal ordering discriminates. All-17 remains blocked
until a future bounded candidate passes every selected fresh control without
changing controller semantics, safety limits, fresh-scene authority or the
exact-zero gate.
