# R8b explicit known-candidate reuse

Status: `BOUNDED_WALKING_VERIFIED / EXPLICIT_SELECTION_BIAS / NO_RUNTIME_AUTHORITY`.
Authority: [ADR-113](../architecture/adr/113-explicit-known-walking-candidate-reuse.md).

## Observation, decision and rejected alternative

The user wants a model that walks, not a successful last checkpoint. The old
predeclared model 3999 diagnostic already completed 20 s and 6.135 m; exact
native controls confirm real alternating whole-foot release. The old V7 final
model 9999 failed, and its separately corrected V8 final matrix also failed.
Those results remain failed and cannot be reinterpreted.

The blanket ban on any later reuse of an intermediate was unnecessarily
restrictive for the practical artifact objective. Admit one explicitly selected
known candidate with disclosed selection bias. This is not an unbiased test of
PPO, not a new claim about the old experiment, and not a checkpoint scan.
Reject requiring another full training budget before even checking an available
candidate; retain the strict physical outcome rather than a last-state rule.

Freeze `lab/profiles/known-walking-candidate.v1.json`, source model 3999 SHA-256
`606cc8101f1cc299d0be1ff3b952c751ac16953592116e19022f497db53e90c3`,
the exact completed source run/generation and V8 descriptor/executables.
The separate wrapper validates the entire source through the unchanged
final-source validator, then the specific candidate name/iteration/hash.
It does no optimization or normalization update. Evaluate fresh **closed-loop**
V8 behavior, not the prior open-loop action tape; require all five complete
ADR-111 physical outcomes and exact native replays. Retain full video for
visual confirmation before claiming the user's bounded walking outcome.

Closed-loop V8 stopping and the complete nominal behavior now pass, as below.
Remaining uncertainty is robustness beyond this nominal lesson, natural style,
broader commands and runtime portability; these are not established by repeated
identical physical starts. No other old checkpoint was scanned or substituted.

## Execution surface

Entry point: `lab/scripts/evaluate_known_walking_candidate.py`.
Completed external output (clean code `18890c1a`):
`/home/kaifaty/NextEngine-training/r8b-known-walking-candidate-v1/evaluation-01`.
The separate fresh optimizer/session **23866**, PID **2309530**, was stopped
through SIGINT only after all candidate gates and video verification passed.
It exited 130, closed all native workers and its artifact hashes, and retained
all checkpoints. It is explicitly interrupted, not a successful experiment.

## Completion audit: the requested bounded walking artifact

Every seed 1001..1005 produces the same nominal physical result:

| Required physical outcome | Observed |
| --- | --- |
| Complete episode without safety events | 1,200 contiguous ticks / 20 s; 4,800 actual physics substeps; timeout, null safety error |
| Forward travel >=3 m | 6.145660 m |
| Forward velocity MAE <=0.2 m/s | 0.1064100617 m/s |
| Final 180 applied commands exactly zero | Exact all-zero vector verified from all five NPZs and native command oracle |
| Stop planar-speed MAE <=0.1 m/s | 0.0832490704 m/s; final planar speed 0.0124608907 m/s |
| >=8 consecutive single-support ticks on each side | 32 left / 23 right, classified load in all four substeps plus actual whole-foot release |
| >=2 qualified support switches | 24 |
| Learned closed-loop policy, not a manual action tape | Fresh policy inference generated each trajectory; independent exact-action native replay follows |
| Complete visual evidence | 1,200-frame / 60 fps / 20 s / 1200x800 MP4, no interpolation or omitted prefix; full one-second contact sheet and mid/final frames inspected |

Visual inspection confirms alternating lifted feet, forward progress and a
two-foot final stance. Motion remains angular, arms are raised, and the world
trajectory drifts sideways; neither natural style nor world-heading precision
is claimed. The observed stop moves another 0.2281 m over the final three
seconds while decelerating, within the frozen speed criterion. Do not describe
it as instantaneous stopping or perfect posture.

| Exact evidence | SHA-256 |
| --- | --- |
| `evaluation-01/run-manifest.json` | `a8678933d1b934109b5c50d991e4228910ebd4c642dbe7b4c1d193cf24e1884c` |
| `evaluation-01/evaluation.json` | `fa75928accedb57f8737dede50930e9ec3e44b7b0311f3e35eb93a335340442d` |
| `video-01/video-manifest.json` | `ee013ac0acd73bbc8703a40da4ef1749a7145be8fd3d32beb70749001589160f` |
| `video-01/walking.mp4` | `93fb8df5124ff8ef140ef5c44257bbde80dd3a3188fb312afd6e8d5029dc5669` |

PASS: every declared candidate/video artifact hash; exact checkpoint identity;
five complete native traces with no safety error; independent applied-zero and
duration checks; ffprobe frame count and video inspection. The new source
wrapper plus existing source/adapter/contact/validation checks pass 52 tests;
four video tests pass, including old-final compatibility and corrupted-frame
rejection. The final combined boundary suite passes **70 tests**. Ruff,
formatting, diff and changed-document local-link checks pass. No physics/public-
contract code changed; the prior native host-check is not rerun for this work.

The practical user request for a first walking model is satisfied by this
explicitly selected artifact, not by claiming old V7 convergence or the later
fresh run's success. Do not restart training merely because the old final model
failed. A new user request for style, heading, long-duration balance, disturbed
starts, terrain, arbitrary commands or runtime integration defines further work.
