# Whole-body Gain Tuner calibration screen

External tooling continuation at NextEngine `27950223`, following the
[isolated angular-unit patch](r8b-gain-tuner6-one-dof-research-2026-09-06.md).
User authorizes relevant calibration/checks without repeated questions.
No anatomy, native controller, selected body, environment or weights changed.

## Result and ceiling

104 captured four-second trajectories: 100 single-channel comparisons plus
four combined-channel tests. All recorded joint angles and velocities remain
inside source ROM +10 microradians and speed +0.001 rad/s; all finite. This
does **not** check native effort-rate, work, contact impact or balance safety.

Eight literal damping candidates improve the simultaneous response in both
zero gravity and gravity9.81 with fixed pelvis and no ground. Stiffness and
the other17 damping channels are unchanged. This is an implicit-force-drive
tracking result, not calibrated standing or native explicit-PD admission.

| Bilateral channel | Source D | Candidate D, N m s/rad | Combined gravity RMSE reduction |
| --- | ---: | ---: | ---: |
| Ankle pitch | 40 | 4.87304544 | 54.1–54.2% |
| Ankle roll | 30 | 1.80627084 | 90.9% |
| Shoulder yaw | 0.875 | 0.40439922 / 0.40442291 | 49.3% |
| Elbow | 14 | 6.41633892 | 50.7% |

Exact per-side values are in the external summaries, not rounded table values.
Maximum tracking error also decreases in every selected channel. No assertion
of optimality or physical critical damping: these are empirically screened D.

## BODY-GAIN-01 r2: isolated-channel discovery

V8 descriptor SHA256
`0fe0252fe8e1c63caaf323bc4c555b4efacf81765db0a542ac851e116e07cf7f`,
26 bodies/25 actuators,75.337kg. Isaac6.0.0.1, patched Gain Tuner overlay3.5.4,
CPU implicit PhysX,240Hz,TGS16/4, f32 simulation/f64 analysis. Each of25 joints
has source and ratio1 candidate, preserving K and changing only that joint D;
two gravity boundaries. Replicas separated3m. First1s zero hold,2s raised-cosine
pulse of0.05rad toward greater ROM, last1s zero hold.961 full-channel samples.

r1 stopped before recorded motion: the apparatus compared full body-frame
inertia tensors to principal-frame diagonal moments. r2 compares independently
decoded source tensor A I A^T and sorted eigenvalues, without widening2e-6
relative/absolute tolerances. Failed r1 script/contract/log/fixture preserved.
[NVIDIA tensor API](https://docs.omniverse.nvidia.com/kit/docs/omni_physics/107.3/extensions/runtime/source/omni.physics.tensors/docs/api/python.html)
defines inertia at COM in rigid-body-prim axes, not principal axes; current
installed articulation wrapper forwards the corresponding tensor call.

Independent review verifies all sealed inputs and independently recomputes all
100 traces. Same eight provisional positives, including with previous-command
error alignment. `SUPPORTED_BOUNDED` for recorded post-initialization metrics;
`INCONCLUSIVE` for literal cold-neutral and gain-only causal interpretation:

- World.reset internally steps; gravity initial |q| reaches0.004236rad.
- Pair offsets introduce floating-point/pose differences; source precommand
  q spread reaches0.0001547rad across replicas.
- q[t] precedes the next interval's target[t]; sample-time tracking and prior
  applied-command error must not be confused.
- Tuner D was recomputed at different post-reset poses in each gravity case.
- Initial seal included widget but not all helper/backend source dependencies.

Source review also finds general-pose COM/tensor-frame handling remains suspect:
`_accumulate_link_inertia` composes link_pose * Translate(COM) under Gf row-vector
conventions and projects the unrotated body-frame tensor onto the robot-frame
axis. Do not claim reliable arbitrary-morphology critical-damping estimation.
This was not repaired or treated as an explanation of our standing failures.

## BODY-GAIN-02 r1: matched-origin simultaneous check

Successor explicitly removes pair placement and boundary-dependent gain changes:
one articulation per fresh process at world origin, four baseline/candidate x
zero/gravity cases. All eight selected joints move simultaneously. Candidate D
is the fixed literal eight-value vector from01 zero-gravity records. Independent
inertia recalculation is not used to change this vector.

Initial post-reset q/v and complete target arrays match **exactly** within each
baseline/candidate pair. Unselected gains and all stiffnesses match exactly;
live candidate D matches prescribed SI values. Mass/full inertia/principal
moment checks pass. Criterion: each of eight channels in each gravity condition
must reduce sample-time RMSE on1..4s by>=10%, not increase maximum error, and
all25 observed channels must stay inside the original angle/speed margins.
All16 channel/boundary comparisons pass. Initial state is explicitly post-reset;
no fictitious claim of zero physics steps or gravity equilibrium.

Unchanged gains do not imply unchanged coupled response: nonselected MTP
tracking RMSE under gravity worsens by about5%. The declared check constrains
all-channel ROM/speed, but tracking improvement only on the selected eight.
Therefore this is not a Pareto improvement of every joint or a foot-servo fix.
The legacy scalar RMSE in each result.json refers to the placeholder left hip;
use the independently recomputed eight-channel summary, not that scalar.

Fixture USD is exported **before** candidate gain edits: it is the construction
fixture, not a ready calibrated asset. Actual candidate identity comes from
the fixed vector, live gains, script and raw arrays. Dependency hashes include
all overlay Python files and relevant articulation/reset implementations.

## Evidence and commands

- 01: `/home/kaifaty/NextEngine-training/body-gain-calibration-8Cq1uK`.
  `SHA256SUMS` hash `7f92fb5f23293e5db3aae3148127aa41cc0fff525f460fa0c19ebb8212510262`;
  summary `53cfa3c677cc90b28372e16c9ada1fe2175108d952f1694818dcb7053c345902`;
  independent review `449a5179a652842a00e5063cdbec964deedcdedbf62bc495e26c96cefa188cb2`.
- 02: `/home/kaifaty/NextEngine-training/body-gain-combined-LMzvPR`.
  `SHA256SUMS` hash `7237c1784dbe84ac0c2170368420b428088259bdc9b1d0868d664cac2a808b79`;
  summary `09afa5068ba1f10a8430016c1008ea9e06dcfa4bb680d77a51737d5cb21231a6`;
  probe `e922357e27bda2f749bd07ea789cfabaf2b6c601cea2c7fa88478a3c68e39be6`;
  contract `afae27a2e58e082f3b9cbff33fe0fc929b0c1e5a1050e4da817aa959c836d601`.

Use `/home/kaifaty/NextEngine-training/isaacsim-6.0/python.sh probe.py --gravity
0` and `--gravity 9.81` for01;02 also requires
`--variant baseline` or `--variant candidate`. Run in fresh copied output
locations; do not overwrite evidence. Analyze with installed env/bin/python
`analyze.py`. Simulator exit0 alone is insufficient: require result JSON,
completion marker, assertions and sealed raw traces.

## Decision and next boundary

Keep V8 nominal baseline and preserve all historical failures. Reject applying
ratio1 automatically to all25 channels: multiple channels tracked worse.
Keep the eight-value vector as an external candidate only. The smallest next
consumer test is that vector under canonical240Hz explicit PD and unchanged
full safety, with a new diagnostic identity and exact unchanged V8 control,
then loaded foot transfer if it passes. Never substitute implicit-drive success
for this test or reuse old weights on a changed body.

For a morphology-general tuner, first add a rotated, offset-COM one-DOF
counterexample and independently known tensor/parallel-axis answer before any
additional vendor patch. Do not sweep body gains to conceal a frame defect.
Standing robustness, canonical transfer and training are NOT_TESTED here.

## Independent review and checks

Fresh02 review: `SUPPORTED_BOUNDED`, no load-bearing defect. Manifest and28
live dependency hashes verified. Separate scalar math.fsum RMSE and sin² pulse
reconstruction confirm all16 selected-channel conditions, all4 full-channel
ROM/speed checks and exact paired initial states. No simulator rerun needed.
Review file `independent-review-r1.md` in02 has SHA256
`7e4e8c805294390204283fcbf408cbe562dbba033bd046a64881770ec25c8061`.
It preserves the nonselected-joint, fixture-export and scalar-label limitations.

PASS: Python compilation, six completed bounded launches after the preserved
r1 apparatus failure, raw-array recomputation, both independent reviews within
their stated ceilings, repository diff and changed local reference checks.
Native ProductChecks/host-check: `NotRun(NoExecutableChange)`; repository diff
is documentation only, external experiment is not runtime validation. Both
simulator experiment processes are closed; no optimizer was started. Canonical
V8 identity, installed stock vendor extension and previous evidence preserved.
