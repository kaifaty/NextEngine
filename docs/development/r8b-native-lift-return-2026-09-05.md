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
- NOT RUN: V7 learned walking evaluation, Isaac correspondence and export/runtime.

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
