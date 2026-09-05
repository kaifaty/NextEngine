# R8b V8 applied stop-window correction

Status: `IMPLEMENTED / NATIVE_CONTROLS_PASS / HOST_CHECK_PASS / NO_LEARNED_EVALUATION`.
Authority: [ADR-110](../architecture/adr/110-applied-command-stop-window.md).
Cause: [V7 exact diagnosis](r8b-native-lift-return-2026-09-05.md).

## Change and non-regression boundary

V8 uses command V3, beginning ramp-down at action index 990 instead of 991.
The integer rate limit stays 16,666 um/s/tick. Its final 180 applied commands
(indices 1020..1199) are zero; index 1200 is the unacted next command. A shared
pure schedule constructor preserves all V2 vector bytes and the old V7
descriptor SHA-256 `6d0e9f4b3e2a6d27831d965632d50af5033d4b94e62032d3c2eeb4a3023ee083`.

V8 preserves V7's complete body, controller, observation/action schemas,
reward formula/coefficients, phase, safety, reset and horizon. Only environment,
command and correspondence identities change. The descriptor declares the
applied index range explicitly. Native profile routing accepts V8 through
ordinary create/reset/step/close; Python optimizer/adapter code and profiles
are untouched. The active V7 executable remains
`ec7887363c5d92bf98cbeb58cd3bbe84545d66337949030103fe40dc7e0d820e`.

## Native same-action controls

External root:
`/home/kaifaty/NextEngine-training/r8b-walking-stop-window-v8/evidence/`.
The existing 1,200-action diagnostic-3999 tape and three 105-action
zero/left/right controls are copied with only the explicit V8 profile selection
and added diagnostic provenance. No policy inference or optimizer is run.

All 1,515 physical frames, classified substeps, raw contacts, applied joint
targets and final safety/terminal outcomes match the corresponding V7 traces.
Expected changes in the full tape are exactly action indices 990..1020,
observation channels 80/84 and reward components 0/9/11/12. Profile-bound
step roots differ, as required. The three shorter reachability controls have
identical observations/rewards as well. An independent integer Python schedule
checks every action command and next-observation command in the native trace.
The full tape has exactly 180 final applied zeros and the prior command is
`[0,20,0]` um/s. This repairs the environment, not the frozen V7 evaluation.

| Artifact | SHA-256 |
| --- | --- |
| `descriptor-v8.json` | `d4e43b3e4ad0d08fc69a0327cd086d133375926de1eb4d9c296ef7987e6c560e` |
| `diagnostic-3999-native.json` | `7d816572f9ee40da4d1b22917d5911329a54016be9dc2de7014dec4d7b729874` |
| `reachability-native.json` | `a6252e7dad41bc27b175658be1da6a7255f92ff9e0cd34c6da142fe493785687` |
| `native-comparison.json` | `df27f286cf7daa958fe479226c9a8335179d574b36203aa72eb5aea0bf568a72` |

The comparison closes both original/new trace and input-tape hashes. Each new
trace also closes the native executable and exact tape hash. Reproduce by
running `export_biomechanics_canonical_walking_v8`, then
`audit_biomechanics_action_tape` on each retained input into fresh output files.
Compare all physical/contact fields exactly, not tolerances or visual similarity.

## Checks and next action

- PASS: 126 feature-enabled motor tests, including old/new applied-window and
  rate oracle, exact descriptor/manifest delta, native reset schedule and
  unchanged zero-action physical control through its actual terminal.
- PASS: six native headless protocol tests, including V8 create/reset/step/close.
- PASS: feature-enabled motor/headless all-target clippy, Rust formatting,
  exact native controls above and diff/link checks.
- PASS: full Linux host-check for implementation `f7efceb5`; external
  `host-check.log` SHA-256 is
  `bcaaf499a6614793689f66b5cbf00268d9032d1187a295c8f8b3bc45e70058e1`.
- NOT RUN: V8 policy inference, corrected learned acceptance, new optimization,
  Isaac correspondence or runtime/export promotion.

Finish the current frozen V7 run and inspect only final model 9999. The
full-tick loaded-support result is diagnostic, not a silent gate replacement.
Predeclare corrected support semantics and final-weight compatibility before
any corrected V8 learned evaluation; do not assume another optimizer is needed.
Rollback retires V8. Never replace the active V7 executable while its learner
can still invoke it for evaluation.
