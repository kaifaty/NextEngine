# TRAIN-4 native-dynamics research decision — 2026-08-14

| Field | Value |
| --- | --- |
| Scope | Optimizer-free research after complete-clip V9 fresh-scene rejection |
| Status | `R101_VECTOR_REJECTED / R102_NATIVE_ROLLOUT_CONTRACT_NEXT` |
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
| R98 V9 native trace | canonical/file SHA-256 `b8e367e873d0384f4a849d25a338751e0a9c56aa7c4143923c1446b63b016372` / `fd6deb633a5b53c4a00b5de6943a5ce12ae59d4b0a232629586807107ef8d77b` | `COMPLETE`, exact R94 outcome reproduced, `40/40` physical substeps captured, trace SHA-256 `b032098868b6bf732155cf8211b4070704363e1f85f070294cb510d116240bcd` |
| R98 V11/control native trace | canonical/file SHA-256 `06727950f3c2b8b6344b1bf15c2c975d52e9d7630a0edcd9503993dfb9f87910` / `5143a7ef5bb068c97594926d226460b657c87c6ba7f9b94a123b0f03c79e31cb` | `COMPLETE`, exact R97 PASS/FAIL reproduced, `44/44 + 36/36` substeps captured; no acceptance authority |
| R99 matched V7 native trace | canonical/file SHA-256 `19aa8ddfce365fadce4146067470e60e6bd48d0c83f80e9064c587fe7e3613db` / `04553001223cdcb638a8cf8029b5f6a453610ca026fa995833418f97ed6b22dd` | Exact R49 case-10 `PASS 11/11` reproduced with `44/44` substeps; trace SHA-256 `552ddd671bc5501500504d54876451e5e93fd4a6979ff1b6b50b5b1649c2b40d`; no acceptance authority |
| R100 one-scalar input | canonical/file/artifact SHA-256 `59d556f5a71943e6bdf60beb3dabbee03d5cc2a7618a341966a6ce3a5d050e78` / `e66cdda52f674d76a7ac1dd70cd60810c02143a9efdefed78804b52751a6a3da` / `427b5e1895519fe83c40189406c3027f14d9b9aa36d50e3b45e300d516f5761a` | Offline proof changes exactly one velocity cell and zero other array elements; PhysX/optimizer/training `0` |
| R100 native trace | canonical/file SHA-256 `024e2251e4518f83c1f5fba4d22142afa83d6a23d8dfdd04e41c40ba1d4210b3` / `afee35e799be7522c434998583175d71f4629cdeeb06e0009fffde9d45629426` | `FAIL` tick `5`, new right-ankle hard impact `6092658 µN·s`; complete `20/20` trace SHA-256 `24489460446a9bee160508b7e16b4125e43cb5ab1f70dbb4440710e754b6eeae` |
| R101 velocity-vector input | canonical/file/artifact SHA-256 `42cc653095911a71a01d6ab3750228e2f47e6190a9fc4a378ce7a19840c183fb` / `096b2f94eaa59a555da82165276c39456cf0ee33cec9123489271548b6deeb2c` / `2dab3cbea1e71f60c328a48b3ec52a3a95af23b6c0214ac05c77c0c3a13ae58a` | Exactly `18` frame-0 joint-velocity cells change to V7; all other array elements remain V9; PhysX/optimizer/training `0` |
| R101 native trace | canonical/file SHA-256 `04b91df1ca6be06c10af9fbf11505457fed8e5a25e2c50abe3f8523bdd684cb6` / `08ef6a379bfc2379ed43b1373f784025760d5b2bdce9bd54ab164376b1afc468` | `FAIL` tick `10`, right-ankle-pitch hard ROM plus remote hard impact `6006560 µN·s`; complete `40/40` trace SHA-256 `364ed05872fb522e6afbab5c945924fda5eaa09fd14d796163255c9ecb83ac31` |

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
- Isaac Lab `2.3.2` distinguishes applied actuator effort from computed effort
  after clipping in its [actuator model](https://isaac-sim.github.io/IsaacLab/v2.3.2/_modules/isaaclab/actuators/actuator_base.html),
  while PhysX documents that
  [`maxJointVelocity`](https://nvidia-omniverse.github.io/PhysX/physx/5.1.2/_build/physx/latest/class_px_articulation_joint_reduced_coordinate.html)
  is enforced with joint-space solver impulses. R98 therefore treats the
  frozen `9000 bp` inner guard as part of the hybrid plant, not as proof that
  the outer velocity terminal cannot be crossed.
- [Trajectory Optimization under Contact Timing Uncertainties](https://arxiv.org/abs/2407.11478)
  shows why nominal contact timing is insufficient: every candidate
  pre-contact state across the uncertain switching region must remain safe.
  This directly matches R98's safe/unsafe touchdown-phase distinction.
- [DynaRetarget v3, 2026-06-10](https://arxiv.org/abs/2602.06827) treats the
  simulator as a black-box dynamics function, rolls out control sequences by
  single shooting, reduces the search to interpolated control knots and grows
  the optimized horizon incrementally. This is the closest published shape to
  the frozen NextEngine plant because it does not require differentiating or
  replacing PhysX contact semantics.
- [DiffMimic v2, 2023-04-26](https://arxiv.org/abs/2304.03274) demonstrates
  simulator-rollout optimization of PD target angles with zero desired
  velocity, but it optimizes a policy and relaxes joint limits for gradient
  propagation. It is therefore evidence that rollout state matching can work,
  and simultaneously a counterexample to importing that method into TRAIN-4:
  learned optimization and weaker limits remain forbidden here.
- The current official PhysX
  [articulation stability guide](https://nvidia-omniverse.github.io/PhysX/ovphysx/latest/guides/articulation_stability.html)
  warns that stiff drives, high one-step angular acceleration and competing
  contacts can destabilize an articulation. It recommends controller/drive
  changes as possible remedies, but those are outside this gate; for R102 the
  bounded inference is only that the reference must be tested against the
  exact clipped/rate-limited plant rather than an open-loop derivative proxy.

The inference for NextEngine is deliberately smaller than those methods. R98
has already rejected contact as the first cause and isolated pre-contact
closed-loop phase divergence. R99 now compares the exact passing V7 trajectory
with the two failed variants before any bounded construction change is chosen.

## Ranked hypotheses

| ID | Hypothesis | Current evidence | Discriminator |
| --- | --- | --- | --- |
| H22 | V9 satisfies pose/velocity geometry but asks the fixed PD plant for dynamically infeasible acceleration or effort | R101 exactly matches V7's initial requested-effort vector, then reaches greater right-ankle-pitch effort debt and the V9 terminal region under unchanged V9 targets | Confirmed as trajectory-wide closed-loop feasibility; boundary state is insufficient |
| H23 | Near-zero offline collider clearance does not predict the full PhysX contact manifold and impulse | R95 isolates ordinal `2`: active foot `1539 µm` above surface with no derivative amplification; R97 reserve case passes all `11` ticks and lowers peak left-foot impulse `6440089 -> 4466405 µN·s` | Supported for the selected case only; broader contact cases remain untested |
| H24 | Complete-clip corrections are nonlocal and regress safe controls while repairing old failures | R100 moves one channel and R101 moves all initial velocities; both preserve V7-like local phase but alter the remote right support chain | Confirmed; end manual substitution and evaluate sequence-level candidates through the native plant |
| H26 | Motor-frame pose/derivative summaries hide the causal physical substep | R98 localizes V11 touchdown to tick `9` substep `2` and overspeed to the immediately following pre-substep state | Confirmed; retain substep traces for every future native discriminator |
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

## R98 physical-substep result

Commit `c8ea848fe359d78b1cf264428ff35fd2f2ba0d4d` adds an explicitly
report-only probe mode. Its bounded acceptance is always `NOT_APPLICABLE`, both
profile dispositions are `STOP_AND_RESEARCH`, and merge/all-17/V19/training
authority is false regardless of the observed outcome. The V9/V11 profile
SHA-256 values are
`4183e55eb9f8e2601a4a0638ff0f8fe18ffa03a1059e0411a01b0ae55022bbd5` /
`1f8fdd1d7aee99a10ec4e6f786485060cbcbc39fe498e0de759ea89ff1568112`.
The full lab suite passes `177/177`.

The V9 worker reproduces R94 exactly: right ankle-pitch hard-ROM excess
`13193 µrad` at tick `10`. The V11 workers reproduce R97 exactly: contact case
`2` passes `11/11`, while case `10` reaches left ankle-roll excess
`775377 µrad/s` at tick `9`. This outcome identity plus complete `40`, `44`
and `36` sample inventories rejects instrumentation perturbation.

The substep order changes the causal conclusion:

1. At the initial physical state, V9/V11 left ankle-roll differs by only
   `70 µrad` position, `600 µrad/s` velocity and `18000 µN·m` requested/
   published effort. The largest action-target difference is only `2791 µrad`
   in right ankle-pitch.
2. By tick `3`, substep `0`, while the left foot is still airborne in both
   runs, V9 left ankle-roll is `+7199121 µrad/s` and V11 is
   `-7198550 µrad/s`. The phase divergence therefore precedes touchdown and
   falsifies contact as the first cause.
3. V9 first observes left-foot contact at tick `9`, substep `0`; V11 does so
   two physical substeps later at tick `9`, substep `2` (`8.33 ms`). At the
   V11 contact step, ankle-roll is `+7196128 µrad/s`, requested effort is
   `-233603940 µN·m`, but the feasible rate-limited published effort is still
   `+1616400 µN·m`.
4. Immediately after that contact response, pre-substep `3` velocity reverses
   to `-8775377 µrad/s`. The outer-limit excess is `775377 µrad/s`; the
   environment publishes zero effort because safety is already blocked.
   Effort-envelope infeasibility remains false and contact impulse
   `1395479 µN·s` is far below the hard-impact limit.

Thus V11 did not simply trade one excessive reference derivative for another.
Small reference changes move a guard/rate-limited closed-loop oscillation to a
different phase; touchdown then exposes it as a terminal event. A third local
pose/derivative smoother would optimize the wrong abstraction again.

## R99 matched passing-control contract

The missing discriminator is the exact V7/R49 `cmu16@238` trajectory: it is a
same-case fresh PASS, whereas both R98 variants fail. R99 may trace only that
existing immutable case under the identical R98 instrumentation and frozen
environment. It must reproduce the R49 PASS, capture `44/44` physical
substeps, remain report-only and change no trajectory, controller, reset or
limit. The comparison will ask which pre-contact state, solver-guard
utilization and effort-slew phase distinguish the successful control. Only
those observed margins may define the next solver locality/native-stability
anchor.

## R99 matched passing-control result

Clean commit `fad451be1ba7da62d41495ee8607a1b6d29f43f7` binds only V7/R47
ordinal `10`; profile SHA-256 is
`4a1ac6899aa22542ee8bf57b4a3d7aeff1a3c4101ea7e1ed220dd2a76dfe8425`.
The lab suite remains `177/177`. R99 reproduces the exact R49
`cmu16@238` PASS for all `11` motor ticks, captures `44/44` substeps and
leaves partial reset `NOT_RUN`, bounded acceptance `NOT_APPLICABLE`, and both
optimizer/training counts at zero. The worker file SHA-256 is
`25f2315c6a16172563a548d3a019c10aec6ca74fa697917b368f4292b178f1d6`.

R99 rejects a tempting but over-broad fix: avoiding the inner PhysX guard is
not necessary for success. V7 left ankle-roll has `18` samples at or above
`7.1 rad/s` and reaches `7.290231 rad/s`, or `9112 bp` of the unchanged outer
limit. It first contacts at tick `10`, substep `2`, with
`-7.185320 rad/s`; the contact response reverses it to `+7.290231 rad/s`,
leaving `0.709769 rad/s` outer reserve. V11 instead contacts at tick `9`,
substep `2`, with `+7.196128 rad/s` and reverses to `-8.775377 rad/s`.
The switching phase and response magnitude, not mere guard use, distinguish
the terminal event.

The matched V7/V9 reference provides a smaller natural discriminator. Their
left-ankle-roll position/target sequence is byte-identical over all `12`
frames. At the first frame only, V7 initializes that channel at
`-40080 µrad/s`, while V9 initializes at `-53280 µrad/s`; the difference is
`13200 µrad/s`, and fixed damping changes initial requested/published effort
from `1202400` to `1598400 µN·m`. By tick `3` the closed-loop phases differ.
This is a boundary-derivative locality effect, not a target-pose effect.

The right support chain shows the downstream cost. V7 right ankle-pitch has
maximum requested-to-published effort debt `24212772 µN·m`, maximum speed
`1629719 µrad/s` and minimum traced position `-602111 µrad`. V9 reaches
`136680080 µN·m`, `3337332 µrad/s` and `-710034 µrad` before its hard-ROM
termination, even though its late reference target is less negative than
V7's. Target geometry alone therefore has the wrong sign as an explanation.

## R100 one-scalar boundary discriminator

R100 may copy exact V9/R93 case `10` and replace only
`joint_velocity_urad_s[0, 5]` (`joint.left-ankle-roll`) from `-53280` with the
matched V7 value `-40080 µrad/s`. The builder must prove that the direct
12-frame joint-position target is byte-identical between V7 and V9, exactly
one scalar changes, and every other array element remains identical to V9.
The resulting case receives one fresh R98-style trace with unchanged
controller, limits, contact modes, pose trajectory and ADR-070 lifecycle.

This is a discriminator, not an admissible hand edit. If it moves the guard
phase and materially improves the native outcome, the next complete-clip
construction may add a boundary-velocity locality anchor derived from that
effect. If it does not, the scalar hypothesis is rejected and the next anchor
must cover the coupled boundary state. Both outcomes remain
`STOP_AND_RESEARCH`; no all-17, V19, optimizer or training is authorized.

## R100 one-scalar result

Clean commit `0bf2a69130b5b8f179064ac38d9ddb2e76bbab96` builds the exact
one-cell input; clean commit `a11158610d2f062b3cf4744a254b789006fd10ca`
binds its only fresh trace. Builder/trace profile SHA-256 is
`15dfb154d76213ce590dc95f5cd7176a47b6537fa27769f54cd3ee185c034763` /
`00542b76d6dad5a4fd10249e43baa27450734b47a1577ab4afce47d85a28ae02`;
the full lab suite passes `178/178`.

The scalar is causal for its local phase. Across the first `20` physical
substeps, R100 left-ankle-roll velocity has RMS distance
`177826 µrad/s` from V7 but `6542737 µrad/s` from V9. At tick `3`, substep
`0`, R100 is `-7197047 µrad/s`, matching V7's negative phase rather than
V9's `+7199121 µrad/s`. Maximum left-ankle speed stays within the unchanged
outer limit at `7199996 µrad/s`.

It is nevertheless unsafe and is rejected as a construction anchor. R100
terminates at tick `5`, frame `243`, on a new
`ground:body.right-ankle-roll` hard impact of `6092658 µN·s`; the matched V9
impulse at the same motor tick is `4521872 µN·s`. The single left-channel
edit therefore increases a remote right-foot impulse by `1570786 µN·s` and
moves termination earlier than either prior failure. Partial reset remains
`NOT_RUN`; optimizer/training remain zero.

## R101 coherent velocity-vector discriminator

R100 proves both sides of the question: boundary velocity controls phase, but
per-channel repair is nonlocal and unsafe. The smallest coherent escalation is
the complete frame-0 joint-velocity vector, not another joint scalar or a pose
smoother. At zero position error, this vector determines the initial damping
request for every actuator under the frozen controller.

R101 may copy exact V9/R93 case `10` and replace only
`joint_velocity_urad_s[0, :]` with the matched V7/R47 vector. The builder must
record the exact changed-cell inventory (`18` expected), prove all other frames
and arrays identical to V9, and prove the 12-frame direct left-ankle target
still byte-identical. One fresh physical-substep trace then asks whether
coherent initial actuator phase removes the R100 remote regression and moves
the original V9 outcome. It remains a report-only discriminator, not an
admissible hand edit.

If R101 does not improve both local phase and remote contact safety, manual
boundary-state substitution is exhausted and the roadmap must move to a
native-rollout/coupled construction or an explicit architecture decision. No
root/pose substitution, coefficient sweep or third scalar is allowed.

## R101 coherent velocity-vector result

Clean commit `da0d6959eb7dfde327437f824261da93a9a5af92` builds the exact
input and clean commit `9bbfbc1e08111fb1223df4e48b68e93b0813b5de` binds its only
fresh trace. Builder/trace profile SHA-256 is
`9214622cb85baba71e09ce88289633eb34555642286a98f98c95a7c9b1aa1b1e` /
`1c0d94663c863564a4c1f144f14df2c358ff2bba701d93e4354c4b88b1b8c786`.
The full lab suite passes `178/178`; initial state is exact, partial reset is
`NOT_RUN`, and optimizer/training remain zero.

R101 proves that coherent initialization is causal but not sufficient. Its
first requested-effort vector is exactly equal to V7 across all `23`
actuators. Across the first `20` physical substeps, its left-ankle-roll
velocity RMS distance is `267756 µrad/s` from V7 and `6461306 µrad/s` from
V9; the all-actuator velocity RMS is `606637` versus `1623104 µrad/s`.
Thus the passing local phase survives the vector replacement.

Every applied joint-position target remains byte-identical to V9 for the
complete shared `40`-substep trace. The remote response improves only relative
to the immediately rejected scalar edit: at tick `5`, right-foot impulse is
`5707869 µN·s`, below R100's `6092658` and the unchanged `6000000` limit, but
still above V9's `4521872`. At tick `10`, substep `3`, it reaches
`6006560 µN·s` and becomes a hard impact. The same motor tick also reaches
right-ankle-pitch position `-709427 µrad`, producing hard-ROM excess
`11295 µrad`. Maximum right-ankle-pitch requested-to-published effort debt is
`186668760 µN·m`, worse than V9's `136680080` and V7's `24212772`.

R101 therefore delays R100's remote failure but neither restores V7 safety nor
closes V9's original support-chain terminal. Its outcome satisfies the
predeclared stopping condition: no third scalar, boundary-vector, root or pose
hand edit is permitted.

## R102 native-rollout construction decision

The smallest architecture-preserving next mechanism is a report-only
simulator-in-the-loop construction contract over the existing fixed PD and
fresh PhysX plant. It is selected over an immediate full kinodynamic NLP
because the observed guard, effort clipping/slew and contact response are the
plant being qualified; an approximate inverse-dynamics model could reproduce
the same proxy mismatch already rejected by R96/R97. It is selected over
DiffMimic or PPO because TRAIN-4 must remain optimizer-free with respect to
learned policy weights.

R102 first builds a hash-closed audit/evaluator, not a candidate search. It
must consume the immutable V7/V9/R100/R101 traces, reproduce the phase, target
identity, effort-debt and remote-contact facts above, and define the future
rollout objective lexicographically: any required-safety event rejects the
candidate before tracking cost, every candidate uses a fresh scene and full
physical-substep evidence, and controller/limits/reset semantics never become
variables. Candidate-generation variables, knot count and search budget remain
`NOT_AUTHORIZED` until that evaluator is reproducible from a clean commit.

If the evaluator closes, the next bounded design may parameterize a complete
future joint-target sequence with low-dimensional time knots and incremental
horizon growth, following the black-box rollout shape of DynaRetarget. Frame-0
state and all frozen safety/controller identities remain fixed. A full
kinodynamic state/torque/contact-force NLP remains the fallback if black-box
rollout cannot produce a deterministic, bounded one-case construction. A
controller change requires a separate architecture decision and is not the
R102 fallback.

## Decision

Freeze R92–R101, retain the contact result as bounded support for H23, and
reject V11 plus every manual boundary-state or open-loop derivative smoother
as a merged/full-corpus direction. Do not tune controller or solver-limit
values and do not begin training. Build only the report-only R102 native-
rollout audit/evaluator before defining a candidate search. All-17 remains
blocked until a future bounded candidate passes every selected fresh control
without changing controller semantics, safety limits, fresh-scene authority
or the exact-zero gate.
