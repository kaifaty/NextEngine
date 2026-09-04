# ADR-103: R8b R&D-only walking discriminator before mirror repair

| Field | Value |
|---|---|
| ID | ADR-103 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-04 |
| Last verified | 2026-09-04 |
| Normative dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-065](065-curriculum-flat-command-locomotion-profile.md), [ADR-102](102-biomechanics-neutral-self-clearance-successor.md) |
| Supersedes | ADR-102 only for requiring passing `MODEL-MIRROR-P1` before any walking optimizer starts. All correspondence thresholds and promotion gates remain unchanged. |
| Superseded by | none |

## Context

The V3 standing checkpoint completes one contact-correct Isaac episode and five
canonical CPU PhysX episodes at the full 3,600-tick bound. Paired mirror work
then falsified trajectory correspondence: joint and root-velocity RMSE exceed
ADR-058 limits, and either plane loses the other plane's exact action tape
before 600 ticks. Repairing or replacing the GPU mirror remains necessary for
promotion, but it does not answer the cheaper product question of whether the
accepted body/controller can acquire the first visible forward skill at all.

## Decision

Permit exactly one separately identified `R&D_ONLY` forward start/stop
discriminator before mirror repair:

- environment `nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v1`;
- observation `nextengine.motor.observation.humanoid-biomechanics-forward-start-stop.v1`;
- action `nextengine.motor.action.humanoid-biomechanics-forward-start-stop-residual.v1`;
- descriptor `nextengine.isaac.humanoid-biomechanics-forward-start-stop.v1`;
- training profile `nextengine.isaac-rsl-rl.rtx3080-biomechanics-forward-start-stop.v1`.

The profile uses biomechanics BodySchema V3, the same procedural-standing
residual and complete safety/contact path, a root-local 84-channel observation,
the 1,200-tick limit and only ADR-065's foundation command distribution:
120-tick zero warm-up, 240-tick segments, 40% stop and 60% forward
`0..0.75 m/s`; strafe, yaw and backward commands remain absent. The bounded
eleven-component ADR-065 reward is evaluated with descriptor-derived effort
and applied-target-rate normalizers.

The exact standing `model_249.pt` may initialize model and observation-
normalizer weights only. The walking run creates a new optimizer, counters,
seed, manifest and run root; it is not a resume. The profile binds the source
profile, generation and checkpoint hashes.

This exception grants no runtime, export, policy-quality or mirror authority.
The final checkpoint must be evaluated without exploration on the GPU and then
on canonical CPU PhysX. A GPU improvement that fails CPU safety or the bounded
forward/start/stop criterion is rejected and receives no expanded budget.
`MODEL-MIRROR-P1` remains an unchanged hard gate before any runtime promotion,
broader curriculum or second walking training budget.

## Product checks and rollback

Focused Rust/Python checks cover descriptor closure, command schedule, reward,
model-weights-only lineage and one native CPU reset/step. The run must be
hash-closed in the external store. Rollback retires this complete profile/run
as R&D evidence and returns to mirror repair; it never relabels the standing
checkpoint or weakens ADR-058 thresholds.
