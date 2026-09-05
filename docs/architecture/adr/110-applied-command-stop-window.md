# ADR-110: Applied-command stop window

| Field | Value |
| --- | --- |
| ID | ADR-110 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-05 |
| Dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-109](109-observable-sole-lift-and-return.md) |
| Supersedes | ADR-109's unchanged-command requirement only for the separately identified V8 schedule repair; V7 and all old gates/runs remain frozen |
| Superseded by | none |

## Evidence and decision

The [native diagnostic](../../development/r8b-native-lift-return-2026-09-05.md)
reproduces an impossible V7 stop gate. The 1,200 actions consume schedule
indices 0..1199; integer ramp-down leaves 20 um/s at index 1020, so only
179 final applied commands are zero. The old test counts the unacted next
observation at index 1200. No controller can fix a command supplied by the task.

Add `nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v8`.
It preserves V7's body, reset, 88 observations, 23 Q1.30 actions, controller,
reward formula and coefficients, phase, safety and 1,200-step horizon. Its
sole behavioral change is the command schedule: start ramp-down at action
index 990 instead of 991. Retain the integer maximum delta 16,666 um/s/tick;
indices 1020..1199 are exactly zero, and index 1200 remains a zero next command.
Warm-up, 0.5 m/s target and command indices 0..989 remain byte-identical.
The distinct command V3 hash binds all 1,201 entries and their index meaning.

A separate descriptor/environment/correspondence identity prevents old V7
evidence or checkpoints from silently acquiring corrected-command semantics.
No force/contact classifier, contact bit, safety limit, observation dimension,
network or reward tuning is part of this correction. V7 continues unchanged
to its predeclared final checkpoint; never overwrite its executable mid-run.

## Checks and scope

Require exact old V7 descriptor bytes, command-vector old/new comparison,
rate-bound checks over every adjacent command, an oracle over the **applied**
0..1199 window, descriptor/manifest closure and production native runner
command observations. Identical action tapes must preserve physical states,
joint targets, classified contacts and terminal outcomes; only command-derived
observations/rewards and profile-dependent roots may change. Zero/left/right
controls retain their full horizon, not a shorter passing prefix.

This admits the native environment and no-optimizer diagnostic controls only.
The full-tick load investigation already demonstrates that raw contact presence
is not loaded support, but its report-only metric is not promoted here. A
corrected learned evaluation must separately predeclare its support semantics,
source final checkpoint, compatibility and exact five-episode matrix. It must
retain 1,200 safe ticks, >=3 m travel, velocity MAE <=0.2 m/s, 180 exact final
zero commands, stop speed MAE <=0.1 m/s, >=8 continuous support ticks per side
and >=2 qualified switches. No new optimizer, resume, checkpoint selection,
Isaac or runtime/export admission is granted.

Use focused motor/headless checks and native controls, plus Linux host-check
for the new profile routing boundary. Rollback retires V8 without editing V7.
The alternative of rounding the delta upward violates the existing exact
per-tick rate bound; shortening the stop criterion hides the defect. Moving
the ramp one tick earlier is the smallest physical-task correction. Both
schedule construction time and storage remain O(episode length), 1,201 entries.
