# R8b corrected final walking evaluator

Status: `IMPLEMENTED / CONTROLS_PASS / WAITING_FOR_CLOSED_FINAL_SOURCE`.
Authority: [ADR-111](../architecture/adr/111-final-weight-corrected-walking-evaluation.md).
This is not a learned V8 evaluation result or an optimizer admission.

## Implementation and claim boundary

`lab/scripts/evaluate_corrected_walking.py` accepts only the predeclared matrix
and final V7 generation. Preflight checks the completed source run, generation
hash, complete artifact closure, full metric sequence, final checkpoint name,
source profile and descriptor hashes. It rejects the still-running real source
before loading a checkpoint. Weights load with `weights_only=True`, exact final
iteration and strict model-state matching. No optimizer is instantiated.

Compatibility compares the entire source/target descriptor after accounting
only for ADR-110's exact command and environment identity changes. Body, joint
and actuator order, observation scales, reward, reset, controller and safety
remain identical. V8 is now recognized by the Python adapter, including Q1.30
action conversion and 88-channel observation/terminal normalization. Old V7
profiles and the live learner's loaded code/executable remain unchanged.

The evaluator saves five deterministic inference episodes and their unchanged
raw-contact reports. For each, it replays the exact quantized action tape in
the native auditor, verifies source/executable hashes, root pose and velocity,
ordered joints, contact flags, commands, terminal and horizon, then applies the
exact load-and-release measurement. Physical travel/tracking/stop gates are
recomputed from the trajectory, not inherited from a raw report's booleans.
An independent applied-command oracle rejects the old 179-zero tail.
The fresh external output closes every result and failure artifact. Source
manifest/checkpoint and evaluator executables are rechecked before completion.

## Controls and exact evidence

External root: `/home/kaifaty/NextEngine-training/r8b-walking-stop-window-v8`.
The isolated binaries are copies of existing native builds, not replacements
of the training binary:

| Relative path | SHA-256 |
| --- | --- |
| `evaluation-tools-01/next_headless` | `431b9c22ab6b2185f9fb8446ec1403e79bf8ef9e0c0ec59374dd5f89b8f63624` |
| `evaluation-tools-01/audit_biomechanics_action_tape` | `7dd63846abcc2fb5bc6939c44eee4b5c8fa0e9fc646eb5025d7924520a997e0d` |
| `evidence/v8-python-adapter-control.json` | `da1862caa6748f4af0562ea53b3b4c4b24b95d56ab5831b281a19e80953ced48` |
| `evidence/v8-python-full-tape-control.json` | `c2f863bd47f12fd28e3720687e1dd5ef92cc840619945d0a9f44c4392e02384b` |
| `evidence/v8-corrected-evaluator-control.json` | `9f1baed9ed1c6faff24763f5f48ea050972220d32ef5466792973a4ac973a4c1` |

- Multi-slot control: 5,120 transitions, 399 terminals/autoresets, every raw
  step field exactly equal to independent direct native clients.
- Existing open-loop 1,200-action control: every raw observation/applied command
  equals the V8 native trace; timeout, retained terminal observation and autoreset
  pass. The first probe wrongly expected post-reset ordinal 1; production starts
  at 1 and increments to 2. The corrected check binds actual initial ordinal +1,
  with no engine change. The failed probe produced no result artifact.
- Pure evaluator on that same previously frozen open-loop tape: all task gates
  pass, 6.135033 m, velocity MAE 0.105950817 m/s, stop MAE 0.080391342 m/s,
  32/23 longest support ticks and 24 switches. These are **not** V8 policy
  inference or a newly selected checkpoint. The control hashes the evaluator
  before the later addition of manifest path metadata; gate logic is unchanged.
- PASS: 51 focused Python tests covering old/new adapter paths, exact action
  conversion, source closure/escape/corruption, final selection, compatibility,
  partial/short/safety episodes, 179-zero rejection and unchanged numeric gates.
  Ruff, formatting, CLI import/help and diff checks pass.
- NOT RUN: learned V8 matrix, new optimizer, runtime/export. Broad host-check
  is not repeated for localized Python evaluation/adapter work; ADR-110's native
  routing host-check still covers the unchanged Rust implementation.

## Smallest next action

Wait for existing V7 PID 2071350 to complete and close final model 9999 and all
source artifacts. Use the pinned external Isaac Python, `PYTHONPATH=lab`, and
the clean commit containing this evaluator. Invoke
`python -m lab.scripts.evaluate_corrected_walking check-inputs` then `evaluate`
with these exact paths:

- `--run`: `/home/kaifaty/NextEngine-training/r8b-canonical-walking-v3/generation-01/runs/TRAIN-1`
- `--generation`: `/home/kaifaty/NextEngine-training/r8b-canonical-walking-v3/generation-01/generation-manifest.json`
- `--target-descriptor`: external root's `evidence/descriptor-v8.json`
- `--headless` / `--auditor`: the two isolated binaries above.
- `--output`: external root's fresh `final-evaluation-01` directory.

Verify all five final episodes, their closed hashes and visual motion before
claiming learned walking. Retain the original V7 failed-gate identity. No extra
intermediate evaluations or restarts are justified while that source is live.
