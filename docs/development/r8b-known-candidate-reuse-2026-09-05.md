# R8b explicit known-candidate reuse

Status: `EVALUATION_PENDING / EXPLICIT_SELECTION_BIAS / NO_RUNTIME_AUTHORITY`.
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

Remaining uncertainty: V8 closed-loop stopping and complete behavior of these
exact weights. If the candidate fails, keep that result and the independent
ADR-112 run; do not select a different old checkpoint. If it passes, report
nominal walking only, with no robustness/terrain/style/runtime inference.

## Execution surface

Entry point: `lab/scripts/evaluate_known_walking_candidate.py`.
Proposed fresh external output:
`/home/kaifaty/NextEngine-training/r8b-known-walking-candidate-v1/evaluation-01`.
The separate fresh optimizer remains session **23866**, PID **2309530**;
do not edit its code/profile/inputs while it runs. Stop redundant compute only
after the full candidate outcome and video have been verified, preserving its
handled interrupted manifest and all artifacts as interrupted, not successful.
