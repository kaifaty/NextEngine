# R8b native V7 lift/return integration

Status: `IMPLEMENTED / NATIVE_CONTROLS_PASS / TRAINING_NOT_YET_STARTED`.
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
- Native adapter and paired controls above pass. Linux host-check and final
  formatting/static checks must close before the first generation freeze.
- NOT RUN: V7 learned walking evaluation, Isaac correspondence and export/runtime.

Next freeze clean commit/profile/executable/descriptor/dependencies under
`generation-01`, then run its sole `runs/TRAIN-1`: 10,000 updates (40.96M
transitions), 14,400 s ceiling, diagnostic updates 999/3999, final model 9999.
This remains a learnability experiment, not a successful walking model. Keep
the original safe horizon, 3 m, alternating support and stopping requirements.
