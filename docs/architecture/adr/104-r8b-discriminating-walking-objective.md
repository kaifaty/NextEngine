# ADR-104: Discriminating R8b walking objective and one bounded successor run

| Field | Value |
|---|---|
| ID | ADR-104 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-04 |
| Last verified | 2026-09-04 |
| Normative dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-065](065-curriculum-flat-command-locomotion-profile.md), [ADR-102](102-biomechanics-neutral-self-clearance-successor.md), [ADR-103](103-r8b-rd-only-walking-discriminator.md) |
| Supersedes | ADR-103 only for its prohibition on a second walking budget and only for the exact V2 discriminator below. V1 remains immutable negative evidence; correspondence and promotion gates are unchanged. |
| Superseded by | [ADR-105](105-r8b-dense-tracking-walking-counterfactual.md), only for one exact dense-tracking V3 counterfactual after V2 failed |

## Context

The single ADR-103 run is a useful negative discriminator: optimization is
finite and improves safe episode length, but the policy remains almost
stationary. The V1 environment pays 64% planar tracking at a representative
`0.5 m/s` stationary error and adds independent positive posture rewards. Its
random foundation schedule also cannot prove a final stop. The bounded
[research report](../../development/r8b-walking-training-research-2026-09-04.md)
finds the objective/evaluation boundary earlier than PPO tuning.

## Decision

Admit exactly one separately identified `R&D_ONLY` successor run:

- environment `nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v2`;
- descriptor `nextengine.isaac.humanoid-biomechanics-forward-start-stop.v2`;
- training profile `nextengine.isaac-rsl-rl.rtx3080-biomechanics-forward-start-stop.v2`;
- seed `44`, `128 × 32 × 250 = 1,024,000` transitions;
- unchanged 84-channel observation, 23-channel residual action, BodySchema V3,
  procedural-standing parent, safety/contact termination, network and PPO
  hyperparameters;
- model-weights-only initialization from the exact admitted standing
  `model_249.pt`; optimizer, counters, seed, generation and run root are fresh.

V2 freezes one deterministic first lesson: ticks before 120 command zero,
then a rate-limited `0.5 m/s` local-forward target, ramp-down from tick 991 and
exact zero commands at ticks `1021..1200`. The reward uses Q16
`square(max(0, 1 - absolute_error / normalization))` with `0.5 m/s` planar and
`0.5 rad/s` yaw widths. Planar/yaw tracking and command-conditioned one-sole
support are positive; tilt, height error, vertical/roll-pitch velocity,
effort, applied-target rate, slip and fall are costs.

Training may begin only after exact Rust/Torch goldens show stationary motion
at no more than 10% of the ideal moving total, integrated command of at least
`3 m`, and 180 final zero commands, and canonical CPU zero-action plus standing
parent controls fail the motion gate.

The final checkpoint must run without exploration on GPU and canonical CPU.
Walking acceptance requires all five CPU episodes to reach the 1,200-tick
timeout with no safety terminal, at least `3 m` signed forward travel per
episode and final-stop forward-speed MAE `<=0.15 m/s`. GPU-only success is not
acceptance. No broader command curriculum, export or runtime authority follows;
failed `MODEL-MIRROR-P1` remains an independent hard promotion gate.

## Product checks and rollback

Focused Rust/Python tests cover profile closure, CPU/Torch reward goldens,
command distance/final stop and native CPU reset/step. The generation and run
remain hash-closed in the external training store. Failure retires V2 as
negative evidence and returns to environment/body diagnosis; it does not
weaken the gate or authorize PPO tuning by default.

V2 failed as specified: final GPU inference survives five horizons but travels
only `0.123/7.258 m`; no saved canonical CPU checkpoint walks. ADR-105 records
the resulting zero-gradient compact-kernel diagnosis and the sole successor.
