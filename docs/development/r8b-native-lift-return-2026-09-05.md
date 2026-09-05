# R8b native V7 lift/return integration

Status: `IMPLEMENTED / NATIVE_CONTROLS_PASS / V7_TRAINING_RUNNING`.
Authority: [ADR-109](../architecture/adr/109-observable-sole-lift-and-return.md).
Predecessor: [report-only discriminator](r8b-lift-return-discriminator-2026-09-05.md).

## Implemented boundary

V7 appends two schema-derived whole-foot box heights to V6's 86 observations
and two per-foot Q16 height-error costs. The sum matches the tested candidate;
all eleven V6 terms remain. The first integration test correctly rejected a
single component above Q16 unity. Splitting by foot preserves the existing
protocol and exact objective; no public bound or safety limit was widened.
The runner caches only immutable sole geometry. Physics, action/reference,
actuators, reset and safety have no new owner or override.

The canonical adapter supports 88-channel reset and terminal observations.
The learner records combined moving height cost, performs predeclared
diagnostic evaluations without checkpoint selection, restores policy training
mode, and closes nested diagnostic artifacts in the final manifest. Final
walking gates are unchanged.

## Native evidence

External root: `/home/kaifaty/NextEngine-training/r8b-canonical-walking-v3/`.

| Artifact | SHA-256 / result |
| --- | --- |
| `evidence/nextengine-v7-descriptor.json` | `6d0e9f4b3e2a6d27831d965632d50af5033d4b94e62032d3c2eeb4a3023ee083` |
| `target/release/next_headless` (repository binary, preserved by run closure) | `ec7887363c5d92bf98cbeb58cd3bbe84545d66337949030103fe40dc7e0d820e` |
| `evidence/final-tape-comparison.json` | `d1ee2403dd561cd7a4bd25bfbe559c621fe7570c4bfcd62feb7d0e48b12b4593` |
| `evidence/reachability-comparison.json` | `9ae154e4e08442e0ca9aa3bf37abf46f322dedeef814ca97887e9b9010911a5f` |
| `adapter-control-01/run-manifest.json` | all 5,120 native transitions exact, 399 terminals/resets |

The V6 final policy's exact Q1.30 tape reproduces all 382 native physics
frames and the same terminal under V7. The three zero/left/right 105-tick
controls add 315 exact paired frames. Observed integer heights agree with
independent collider geometry within 0.999597 um (floor rounding); every new
cost exactly matches the independent Python phase oracle applied to raw native
heights. First 86 observations also match in the new paired reachability trace.
The original V6 descriptor still hashes `fb5276f5…10dca` byte-for-byte.

The initial comparison against the older V5 reachability JSON rejected missing
contact positions: that recording predates that field. Physics roots were
equal. A fresh V6 replay of the identical tape supplies full positions; the
strict V6/V7 comparison passes without dropping contact checks. Both old and
new recordings are retained, not repaired in place.

Reproduce via `python -m lab.scripts.verify_walking_lift_return` with explicit
source/successor traces, descriptor and fresh external output. A separate Rust
test compares V6/V7 zero-action states, all legacy reward/observation fields,
applied targets, safety checkpoints and terminal through the actual terminal.

## Checks and next action

- PASS: 123 feature-enabled motor tests and five headless protocol tests.
- PASS: 25 focused Python tests, including corruption rejection, independent
  phase/geometry controls, 88-channel timeout/reset and diagnostic-mode restore.
- PASS: native adapter and paired controls above; Linux host-check, Rust fmt,
  feature-enabled native clippy, Ruff and changed-document local links.
- FAILED: the two predeclared V7 diagnostic matrices; final evaluation is pending.
- NOT RUN: renewed Isaac correspondence and export/runtime.

## Launched run

Clean code `f0c15bd4ae73df4fbd5b7e95b97b39db3a2d7bc0` freezes generation-01.
Generation manifest SHA-256:
`0c7d54ff302ab0fd7cb31be98ae8a5b37e7e286ceeae5ed77f074e1224a1dfa1`.
Profile SHA-256: `4d39a4e07694f34900d93c5d81008dab55606a91ded802aaca4f605c053f5c37`.
The clean freeze repeats all 5,120 exact native control transitions. Its sole
`generation-01/runs/TRAIN-1` is running with finite PPO updates confirmed;
`evidence/training-01.log` and append-only `metrics.jsonl` are live projections.
The run's output hashes close only when it terminates. Later documentation-only
commits do not relabel the frozen implementation or alter its running inputs.

Budget: 10,000 updates (40.96M transitions), 14,400 s ceiling, diagnostic updates
999/3999, final model 9999. Next monitor the existing process and predeclared
milestone, not launch another run. This remains a learnability experiment, not
a successful walking model. Keep the original safe horizon, 3 m, alternating
support and stopping requirements.

## Predeclared diagnostic 999 — failed, not a stopping rule

At 4,096,000 transitions, all five nominal seeds again produce the same
trajectory: 406 ticks, 0.165737 m forward, velocity MAE 0.311684 m/s, zero
single-support runs/switches, and `terminal.joint-safety`. No final gate passes.
These identical nominal resets are not five independent robustness trials.
The run has continued past this diagnostic as required; no weights, settings,
budget, training inputs or acceptance conditions were changed.

Exact source artifacts under `generation-01/runs/TRAIN-1`:

| Artifact | SHA-256 |
| --- | --- |
| `model_999.pt` | `7e358c9e470cfe24588210e924fccb1c7d976197255fa2daf3ef9db145ae8295` |
| `diagnostic-999/evaluation.json` | `7586a7e6d2296d67f292bba4038181f8b0e9bcc9b48b26378f53e99a13912c9b` |
| `diagnostic-999/evaluation-1001.npz` | `1ac591d042747c1d9c02f7519d9d1d987b6784f4063d98e5ccfbf2383e20a14f` |

Replay `evidence/diagnostic-999-actions.json` with the existing native example.
Its input closes the source checkpoint/evaluation/generation hashes. Output
`evidence/diagnostic-999-native.json` hashes
`4105346b24a31c611124735b9a269c801965b2d07612f89eca4df3a3ba483224`.
The existing contact-audit `analyze` function verifies every recorded root
pose/velocity, ordered joint position, command and contact observation exactly.
`evidence/diagnostic-999-report.json` records the measurements and source hashes.

The first recorded hard-ROM breach is right ankle pitch at tick 406:
q=-698,656 urad versus lower limit -698,132 urad, excess 524 urad, above the
unchanged 10-urad observation tolerance. Native diagnostics report
`MOTOR_SAFETY_HARD_ROM_VIOLATION`, with four actual physical substeps completed.
This is an actual physical joint excursion, not action-order or replay mismatch;
it does not establish why the learned controller chooses the unsafe trajectory.
Do not infer a safety-envelope defect or relax the limit from this observation.

Complete-box clearance peaks at 6.945 mm left / 9.350 mm right; neither foot
reaches 20 mm. Small clearance with contact presence is possible within the
existing contact margin, but does not satisfy the original support/step gates.
The stochastic training rollout's occasional larger lift is therefore not
evidence of a learned deterministic gait. Next inspect only the already
predeclared diagnostic 3999 and final 9999, retaining this fixed negative result.

## Predeclared diagnostic 3999 — geometric progress, failed matrix

All five nominal resets again reproduce one trajectory: 1,200 safe ticks,
6.135033 m forward, velocity MAE 0.10568467 m/s, final-window planar speed MAE
0.080391342 m/s. Contact-presence-based longest single support remains 15/3
ticks with zero qualified switches. Final-zero-command, bilateral support and
alternation gates fail. This is a report-only checkpoint, not final acceptance
or five independent robustness trials.

Source artifacts relative to the same external root:

| Artifact | SHA-256 |
| --- | --- |
| `generation-01/runs/TRAIN-1/model_3999.pt` | `606cc8101f1cc299d0be1ff3b952c751ac16953592116e19022f497db53e90c3` |
| `generation-01/runs/TRAIN-1/diagnostic-3999/evaluation.json` | `850bbf29b1fc49ba4d27f2f864cd043db4040c84701e11388e051cb01a6597f5` |
| `generation-01/runs/TRAIN-1/diagnostic-3999/evaluation-1001.npz` | `fa106d6d714ac54a2ae89d1c8e80846c07f68dcd496c86edbfa99efdf5f2cf50` |
| `evidence/diagnostic-3999-actions.json` | `1cd5905140c5087de504e477daeb8f438c90c3a280b180415577afa5d798087a` |
| `evidence/diagnostic-3999-native.json` | `6dd04b3dc9a92c07103609d11d5b032552b5d4fbc7e9a7b2a7d0e9680a752708` |
| `evidence/diagnostic-3999-contact-audit/report.json` | `4c7fe6a6a85e7dd89b85da02f6ae515fedc8748ced13e4d989b3a6ddb2150f72` |

The retained tape is replayed with the existing native executable, without
rebuilding or modifying the live learner. The independent contact audit passes
exact root pose/velocity, ordered joints, command and contact equality on all
1,200 frames. The tape's running-manifest hash is a snapshot, not final output
closure. Derived figures are in `evidence/diagnostic-3999-contact-audit/`.

### Actual feet versus contact-presence qualification

Unlike V6 and diagnostic 999, complete feet repeatedly lift and return, reaching
41.632 / 76.827 mm. There are 139/152 moving frames above 20 mm. This falsifies
the explanation that this checkpoint only unloads grounded feet. It does not
establish acceptable style, full-tick loaded support or final quality.
The same frames include 257/174 cases of contact presence with zero vertical
impulse in the last physical substep; 214/120 also combine contact presence
with clearance above 5 mm. These are separate counts, not their intersection.

`biomechanics_standing_runner.rs::contact_flags` checks existence of the
foot/ground pair without an impulse threshold; the evaluator qualifies support
from those bits. Therefore zero qualified switches cannot be interpreted as
zero physical alternating lifts. PhysX explicitly generates contact points
before touching and without necessarily applying an impulse; see its
[5.4.1 collision documentation](https://nvidia-omniverse.github.io/PhysX/physx/5.4.1/docs/AdvancedCollisionDetection.html),
rechecked 2026-09-05. A full-tick force/support discriminator is still needed;
do not substitute last-substep forces or new thresholds into the frozen gate.

### Independent command-schedule defect

The episode applies schedule indices 0..1199. The last nonzero command is at
1020: `[0,20,0]` um/s, followed by only 179 zeros (1021..1199). The function
`biomechanics_forward_start_stop_command_schedule_v2` uses integer increment
`1_000_000/60 = 16_666`; thirty reductions from 500,000 leave 20 um/s. Its
1,201-entry vector has 180 zero entries at 1021..1200, but entry 1200 is the
next observation's command and receives no action in this horizon. The old
test explicitly accepts this vector while the descriptor advertises 180.

This is an Environment schedule/indexing mismatch, not a learned failure to
stop. The evaluator correctly rejects the actual 180-action tail. No policy
can pass that gate with the current fixed schedule. Retain the old failed
identity; do not weaken the requirement to 179 or silently relabel V7.

Decision: finish the already frozen run unchanged as a learning experiment,
then diagnose its predeclared final model. Before any successor run, correct
the schedule in a separately identified successor and test applied action
indices, ramp-rate bounds and 180 exact zeros. Independently resolve full-tick
support semantics against geometry and load controls before changing gait
assessment. No new optimizer run, reward edit or safety relaxation is performed
by this investigation. PASS: native exact replay and five contact-audit tests;
the earlier V6 geometry result was independently reproduced again.
