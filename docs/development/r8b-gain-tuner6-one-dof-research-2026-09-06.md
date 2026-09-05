# Installed Gain Tuner 3.5.2 — one-DOF comparison

External tool-only GAIN-ORACLE-02 revision1, repository b9fd63d9. No body,
controller, training environment, installed vendor files or driver changed.
Continues the [5.1 oracle](r8b-gain-tuner-one-dof-research-2026-09-05.md)
after the [isolated6 installation](r8b-isaac6-install-2026-09-05.md).

## Observed result

The installed6 force-revolute path passes the three axial-inertia comparisons
and live stiffness comparisons. Live damping remains **180/pi =57.2957795**
times its requested SI value. Independently reviewed: C1/C3
`SUPPORTED_BOUNDED`, C2 `REFUTED` (empirical/numerical/correspondence evidence).

| Rotor principal moments, kg m² | Expected X inertia | Reported inertia | Live K / expected K | Live D / expected D |
| --- | ---: | ---: | ---: | ---: |
| 0.1,0.1,0.1 | 0.1 | 0.10000000149 | 0.99999997058 | 57.29577951308 |
| 0.2,0.2,0.2 | 0.2 | 0.20000000298 | 0.99999997058 | 57.29577951308 |
| 0.1,0.15,0.15 | 0.1 | 0.10000000149 | 0.99999997058 | 57.29577951308 |

For J0.1kg m², requested2Hz/ratio1: expected K15.791367N m/rad,
D2.513274N m s/rad. PhysX reports K15.79136658 and D144.0. The UI continues
to report2Hz/ratio1; the actual-gain-derived damping ratio is57.29578.
These are measured gains, not an inferred frequency from a video.

## Protocol and discrimination

Same three physically realizable, world-fixed, centered X-revolute fixtures as
the old test. No gravity, colliders, friction, armature, angular body damping,
randomness or policy. Identity principal/joint frames, rotor mass1kg.
CPU implicit PhysX, TGS16/4, f32 physics and f64 NumPy2.3.1/SciPy1.17.0.
Two resolutions240/960Hz; four conditions each; all24 full trajectories retained.
Known-Ix control is a separate UI input, never substituted into real mass queries.
The installed6 native schema helpers and tensor mass API are used without the
old discovery shim. Current JointListEntry and dictionary inertia-provider API
replace the old UI signature; no vendor computation is replaced.

Frozen C1/C2 thresholds remain1e-5 relative error for inertia and live K/D.
Independent critical-step oracle: q=.05*(1-(1+w*t)*exp(-w*t)), w=4*pi,
q(0)=v(0)=0. Control max position error/.05 <=3% at240Hz, <=1% at960Hz,
strictly improved by refinement. All analytic controls pass:0.10310–0.13827%
coarse,0.03388–0.04484% fine. Raw gains before and after each trace are saved.

The counterfactual changes ONLY USD damping by pi/180 after known-Ix UI
calculation. Live gains then match analytic SI; fine response errors are
0.02719–0.03388%, versus93.37978% with unmodified tool gains. This supports
the missing damping-unit conversion, not a whole-body calibration guarantee.
At240Hz the J0.1 unmodified tool also differs from its own live-gain ODE by
6.00817%; at960Hz this is about0.000116%. Do not attribute every coarse
trajectory discrepancy solely to the gain mapping or claim solver exactness.

## Source correspondence

Installed `gains_tuner.py` now projects the accumulated inertia onto the joint
axis and distinguishes fixed sides. `gain_tuner_drive_math.py` converts K to
degree-based units but returns D in SI; `JointItem.on_update_damping` writes
that raw D to USD. The known-Ix discriminator isolates this boundary.
[NVIDIA6 rigging documentation](https://docs.isaacsim.omniverse.nvidia.com/6.0.0/robot_setup_tutorials/tutorial_rig_legged_robot.html)
explicitly requires multiplying both K and D by pi/180 before authoring USD.
Source checked2026-09-06. The positive inertia result is limited to aligned,
centered single-DOF fixtures; rotated tensors, offsets and coupled/free/contact
bodies are not covered.

## Evidence and next action

External directory `/home/kaifaty/NextEngine-training/gain-oracle6-MrAjmH`:
`contract.md`, `probe.py`, `summarize.py`, `240/`, `960/`, both launch logs,
`summary.json`. Each resolution has fixture USD, result JSON and12 NPZ traces.
Run with `/home/kaifaty/NextEngine-training/isaacsim-6.0/python.sh probe.py --hz
240` or `--hz 960` in a fresh copy/output directory; existing evidence is immutable.

- Contract SHA256 `8395c7b80d1e75a42b8f7832798bf16bb16ebf8c2103b47e6d5f99d06aec908f`.
- Probe SHA256 `ef97a5af19038a4a91c286a95a73a2e868f980c0164ac3e9df41b9bec36fce2b`.
- Summary SHA256 `1f6504d11162585f36b5b0ae7fc67daaf93ad2964fee4510e03dd7ae42617161`.
- 240Hz result `b2cc2bc25f390b7d50643e0696ac18fbf4e2efd4d352a29c2c43c637cc4a64ec`.
- 960Hz result `ec8b89d1e1c751f8e6f0c934c74bf61a13e0487963a8b1eb582e27b2fea81494`.

Do not select unmodified frequency/ratio auto gains. Smallest next implementation
is a scoped unit-boundary correction with reverse UI conversion and regression
coverage, then a body-level waveform check. The damping-only counterfactual is
not an installed fix. No new full tuner, anatomy edit or training follows from
this test alone. Old5.1 conclusions and evidence remain intact.

Independent review verified all39 sealed hashes and all24 reference arrays
(14,424 samples) using a separate DOP853 integration, difference <=5.36e-14rad.
Vendor/UI mapping and controls support the bounded damping-boundary verdict.
Sequential-condition history and the coarse live-ODE mismatch remain explicit
limits; no simulator rerun or load-bearing repair needed. Review artifact:
`/home/kaifaty/NextEngine-training/gain-oracle6-MrAjmH/independent-review-r1.md`,
SHA256 `0e67bd5232afd507f10b78eae73629392508ac63910d5f429ad66c772c4ea764`.

Checks: script compilation, two bounded executions, summary controls and
independent review PASS; unmodified live damping comparison FAIL. Repository
diff/local links PASS. Native ProductChecks NOT_RUN: external tool experiment
and documentation-only repository changes. No simulator process remains active.

## Upstream research — 2026-09-06

Read-only web/source investigation requested after the measured result; no
new numerical experiment, vendor patch, training or upstream submission.
Competing explanations: missing angular-unit conversion; legitimate API-specific
units; force/velocity saturation or imported-asset problems; backend-specific
motion. The existing isolated PhysX fixture discriminates the latter two from
the measured gain mapping, without ruling them out for other robots.

### History and current public source

[NVIDIA changelog](https://github.com/isaac-sim/IsaacSim/blob/045ca8b59622b99a408092124377c66346e8d9c2/source/extensions/isaacsim.robot_setup.gain_tuner/docs/CHANGELOG.md):
3.1.5 (2025-12-16) explicitly repaired axis inertia, fixed chains and radians
to degrees for stored stiffness. 3.5.1 (2026-05-07) repaired damping-ratio
readback and natural-frequency damping updates to use radian-equivalent
stiffness. 3.5.3 (2026-06-09) lists lint/docstring/documentation changes, not a
damping-unit repair. These are related historical fixes, not confirmation that
NVIDIA has acknowledged our exact reproducer.

Public Isaac6.0.1 commit `045ca8b59622b99a408092124377c66346e8d9c2`
(2026-06-22) still has the inconsistent mapping in
[gain_tuner_drive_math.py](https://github.com/isaac-sim/IsaacSim/blob/045ca8b59622b99a408092124377c66346e8d9c2/source/extensions/isaacsim.robot_setup.gain_tuner/isaacsim/robot_setup/gain_tuner/gain_tuner_drive_math.py):
`stiffness_stored_and_damping_from_natural_frequency_revolute_position`
multiplies K by DEG_TO_RAD, but returns SI D without that factor. Its
damping-ratio setter has the same mismatch. The corresponding
[UI writer](https://github.com/isaac-sim/IsaacSim/blob/045ca8b59622b99a408092124377c66346e8d9c2/source/extensions/isaacsim.robot_setup.gain_tuner/isaacsim/robot_setup/gain_tuner/ui/joint_table_widget.py)
stores D directly in USD. This is source inspection, NOT a6.0.1 runtime test.
The official rigging documentation linked above requires pi/180 for BOTH
angular USD gains; tensor API gains use SI. Therefore the conversion belongs
at that API boundary, not in all BodySchema gains or prismatic drives.

### Why existing tests need not catch this path

In the same pinned [upstream tests](https://github.com/isaac-sim/IsaacSim/blob/045ca8b59622b99a408092124377c66346e8d9c2/source/extensions/isaacsim.robot_setup.gain_tuner/isaacsim/robot_setup/gain_tuner/tests/test_gain_tuner.py),
`_measure_pd_step_response` computes gains with test helpers, converts BOTH
K and D with `_revolute_drive_stiffness_damping_si_to_usd`, and directly authors
USD. This motion harness does not exercise the faulty UI conversion path.
Separately, `_assert_oscillation_ok` conditionally skips a severe frequency
mismatch when readback matches design and at least three peaks exist. The
changelog describes that skip as PhysX triage. A coverage gap is a supported
explanation for how this particular test can miss the bug; the actual CI
history or reason the defect shipped has NOT been established. Do not claim
the skip caused our damping error or that every upstream test bypasses UI.

### Similar reports, different causes

- [Issue104](https://github.com/isaac-sim/IsaacSim/issues/104), opened2025-08-04:
  tracking failed despite tuning. Maintainer suggested checking maximum force
  and joint velocity; the reporter confirmed resolution on2025-08-20. Not an
  angular damping-unit reproducer.
- [Issue681](https://github.com/isaac-sim/IsaacSim/issues/681), opened2026-06-23:
  initially blamed Newton and suggested gain retuning. The author's
  [July3 correction](https://github.com/isaac-sim/IsaacSim/issues/681#issuecomment-4875411716)
  identifies an old SolidWorks-derived USD asset; reconversion with official
  urdf-usd-converter0.2.0 restored original gains on both backends. The author
  explicitly retracts the earlier retuning workaround. Maintainer closed the
  issue on2026-08-07. Do not reuse its early workaround as a confirmed fix.

Search covered NVIDIA documentation/forums, IsaacSim GitHub gain/damping/unit
issues and the pinned implementation/changelog; issue comments were read via
GitHub API. No exact public180/pi damping reproducer or confirmed ready-made
fix was found in this bounded search. One final API query returned403, so this
is not an exhaustive issue-index claim.

Conclusion: documentation, source and our existing isolated measurement support
a mixed SI/USD-unit defect, consistent with incomplete earlier repairs. No
evidence here calls for another anatomy edit or an untested upgrade. Retain
the scoped unit-boundary correction as the next proposed action; validate the
real UI-to-live-gain path and reverse readback, then body-level response. Revisit
if NVIDIA publishes a repair or an end-to-end control contradicts this mapping.

## Local patch implementation — 2026-09-06

User authorized the patch and optionally an upstream PR. Angular damping-unit
boundary is now `ROOT_CAUSE_FIXED` in an isolated extension copy, not in the
original pip installation or any NextEngine body/controller. This supersedes
the earlier "next proposed action" for that boundary only.

Source checkout `/home/kaifaty/NextEngine-training/IsaacSim-gain-tuner-fix`,
branch `codex/fix-gain-tuner-angular-damping`, commit
`c02aaa1caf9c20233b5d9f63ce55f06c01527b70`, upstream base
`987015050efebfd0cd5d3736ae47fffe5adee308`. Changes are five files: UI boundary,
math unit documentation, regression tests, extension version and changelog.
The math helpers retain their existing API (stored K, SI D); only non-mimic
revolute / rotational D6 UI callers convert D on write and readback. Manual
USD gains, linear paths and mimic parameters receive no new scaling.

Deployment is a backport to the installed3.5.2 extension copied into
`/home/kaifaty/NextEngine-training/isaacsim-6.0/local-exts/isaacsim.robot_setup.gain_tuner`.
Its local3.5.4 metadata/title identify the patch; this is NOT an NVIDIA release
or an installation of Isaac6.0.1. The original39 sealed files, including vendor
sources and original launch script, still match their hashes. A separate
launcher opts in via Kit's extension search path:
`/home/kaifaty/NextEngine-training/isaacsim-6.0/isaac-sim-gain-tuner.sh`.
Use the original `isaac-sim.sh` to run without this overlay; no uninstall needed.
Tests assert the actual imported module path, not just the requested extension.

### Regression evidence

External `/home/kaifaty/NextEngine-training/gain-patch-validation-ceOW4Y` retains
source patch, harnesses, logs, raw NPZ and result JSON. Five new async UI tests
cover frequency edits, ratio edits, absolute USD gains, manual damping readback,
row reconstruction without stage mutation, force/acceleration, rotational D6,
linear non-regression, velocity mode and mimic parameters. Against stock3.5.2:
ten angular subcases fail, no errors/skips; non-angular controls pass. Against
both the source branch and deployed backport: all five plus eight existing
upstream drive-math tests pass (13 tests each, no failures/errors/skips).

The original three physical fixtures and force-mode thresholds are reused:
24 traces at240/960Hz, four conditions (direct SI control, tuner inertia,
known inertia, frequency edit3Hz->2Hz). All live K/D relative errors are
2.94208e-8; expected D2.513274 now reads2.513274, not144. Errors against the
critical-step solution are0.10310–0.13827% at240Hz and0.03388–0.04484% at960Hz,
with strict refinement for every condition. All six original direct-SI control
time/q/v arrays remain bit-exact. No post-UI corrective setter is used.

Acceleration-mode extension of the fixture passes all12 live K/D comparisons
(max relative error6.72067e-8), but motion error1.30277–2.55630% FAILS the frozen
1% fine-step criterion, including the direct-SI control BEFORE applying tuner
gains. Initial run stopped on that control; a collection-only rerun retained
all traces and records `motion_criterion_pass=false`, without loosening the
threshold. This limits dynamic calibration claims, not the observed unit fix.
Do not diagnose its mechanism or mark acceleration dynamics calibrated here.

The first UI harness launch hit Python package attribute shadowing (`ui`);
using `importlib.import_module` repaired only harness loading, before collecting
the baseline. `probe-v1.py` retains the exact script used for the force tests;
the later collection variant saves negative motion results before closing Kit.
JSON outcomes and completion markers, not Kit process exit status alone, are
the validation authority.

Summary SHA256 `4af96360d457cd02262c52d161a6a4127eedd638cf71ee15469d3ff80309a0c0`.
Export `0001-Fix-angular-damping-conversion-at-Gain-Tuner-USD-bou.patch` SHA256
`ef212df5f8bba95b75635cb683470012871a21b6205692bd574178bb3ae6720e`.
The summary seals deployed files, harnesses and outputs; exact source is the
external Git commit. Generated evidence is not committed into NextEngine.

### Limits and handoff

- Existing USD gains are NOT automatically migrated. Re-enter the intended
  damping ratio for gains previously authored through the broken path; do not
  divide arbitrary BodySchema/direct-SI gains by57.3.
- Linear natural-frequency math already reuses revolute stiffness formulas.
  This patch preserves that path, not certifies it. Universal-morphology
  calibration, coupled/contact response and NextEngine transfer remain untested.
- Next scoped action is a body-level force-drive waveform check. No training,
  anatomy changes, mirror admission or new physics backend follows from this fix.
- GitHub account access is available, but current upstream
  [CONTRIBUTING.md](https://github.com/isaac-sim/IsaacSim/blob/987015050efebfd0cd5d3736ae47fffe5adee308/CONTRIBUTING.md)
  says contributions are not accepted and directs bug reports to the forum.
  No PR, public fork, issue or forum message was created. Patch remains ready
  locally for a developer-requested submission or separately authorized report.

Checks: Black/isort, focused Ruff, shell syntax, Git diff/local paths PASS;
new UI and existing drive-math tests PASS; force physics PASS; acceleration
gain units PASS but acceleration motion criterion FAIL (control also fails).
Full upstream suite and NextEngine native ProductChecks NOT_RUN: bounded external
Python extension patch, documentation-only NextEngine changes. No validation
process remains active; unrelated host GPU processes were left untouched.
