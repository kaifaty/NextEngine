# R8b prospective validated walking run

Status: `IMPLEMENTED / PREFLIGHT_PENDING / NO_LEARNED_QUALITY_CLAIM`.
Authority: [ADR-112](../architecture/adr/112-prospective-validated-walking-training.md).

## Decision

The [closed numerical controls](r8b-final-policy-regression-research-2026-09-05.md)
support testing fixed learning rate 1e-5. Keep the V8 physical task, seed 44,
network and other PPO settings unchanged. This is one fresh run, not a resume,
retrospective selection or an equal-budget proof that the schedule alone caused
the old regression. The repaired stop window and evaluation protocol are also
explicit differences from the old V7 run.

Save and natively validate after every 100 updates. Select the first checkpoint
passing all five complete original physical-task gates and stop. Otherwise stop
at 10,000 updates / 40.96M transitions or the 14,400-second wall ceiling, retaining
failed evidence. Fixed LR may learn too slowly; the local probe establishes no
fresh-training convergence promise. Another unchanged retry is not admitted.

The callback checks that saved weights equal the evaluated policy, preserves
model/normalizer state and restores mode plus Python/NumPy/Torch CPU/CUDA RNG.
It retains full inference and native replay trajectories, exact action/seed/
executable identities and corrected loaded-support/whole-foot-release evidence.
No evaluation samples enter PPO storage. Passive update diagnostics are enabled
only in the new profile; old profile execution remains unchanged.

## Checks before generation freeze

- PASS: 65 focused tests covering new callback/selection, canonical adapter and
  timeout bootstrap, corrected evaluation and geometric/contact oracles, old
  V7 milestone behavior, passive observer and gradient diagnostics.
- PASS: Ruff check, format check and `git diff --check` for the changed code.
- PASS: new callback on the unchanged closed model 9999 reproduces every episode
  dictionary of the previous corrected final evaluation exactly: five failures
  at tick 152. This is a no-optimizer negative non-regression control, not a new
  trained candidate. Evidence is outside Git under
  `/home/kaifaty/NextEngine-training/r8b-canonical-walking-v4/evidence/validation-control-01`.
- NOT RUN yet: new generation freeze/native multi-slot control and optimizer.
- No new Rust/public/physics contract: retain ADR-110's prior passing native
  routing and Linux host-check; broad host-check is not repeated for private
  Python callbacks.

## Inputs and next action

Profile: `lab/profiles/canonical-rsl-rl-walking.v4.json`.
Descriptor: `/home/kaifaty/NextEngine-training/r8b-walking-stop-window-v8/evidence/descriptor-v8.json`.
Executables: the immutable `evaluation-tools-01/next_headless` and
`evaluation-tools-01/audit_biomechanics_action_tape` beneath that V8 root.
The profile closes all three hashes. Freeze the new clean-commit generation at
`/home/kaifaty/NextEngine-training/r8b-canonical-walking-v4/generation-01`, then
launch its sole `runs/TRAIN-1`. Preserve the process handle and inspect it rather
than restarting on an observation timeout. No runtime activation or held-out
robustness claim follows from this nominal lesson.
