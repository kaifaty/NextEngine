# ADR-105: Dense-tracking R8b walking counterfactual

| Field | Value |
|---|---|
| ID | ADR-105 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-04 |
| Last verified | 2026-09-04 |
| Normative dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-104](104-r8b-discriminating-walking-objective.md) |
| Supersedes | ADR-104 only for its one-run ceiling and only for the exact V3 counterfactual below |
| Superseded by | none |

## Context

The complete V2 run falsifies walking despite finite PPO metrics. Its final GPU
policy survives five full episodes but travels only `0.123/7.258 m`; canonical
CPU checkpoints either fall or survive while moving backward. The compact
tracking kernel is exactly zero at the initial stationary `0.5 m/s` error, so
the policy receives no local task-reward gradient toward forward motion. The
GPU reward decomposition confirms that nearly all planar reward comes from the
zero-command warm-up and stop intervals.

## Decision

Admit one separately identified `R&D_ONLY` V3 counterfactual:

- environment `nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v3`;
- descriptor `nextengine.isaac.humanoid-biomechanics-forward-start-stop.v3`;
- profile `nextengine.isaac-rsl-rl.rtx3080-biomechanics-forward-start-stop.v3`;
- seed `44`, `128 × 32 × 250 = 1,024,000` transitions.

V3 changes only planar/yaw tracking to the exact bounded Q16 kernel
`square(1 / (1 + (error / 0.5)^2))`. It preserves V2's body, command schedule,
reward coefficients, posture/safety costs, controller, observation/action,
standing-weight initialization, PPO and evaluation gates. The stationary
`0.5 m/s` state therefore has a non-zero but suboptimal tracking value and the
reward increases strictly at the exact `0`, `0.25` and `0.5 m/s` control
points.

Training may begin only after Rust/Torch goldens agree, the descriptor and
native CPU runner accept V3, and zero-action plus standing-parent controls fail
the unchanged motion gate. V2 remains immutable negative evidence. Failure
returns to command curriculum or action/body diagnosis; it does not authorize
PPO tuning, weaker safety, runtime promotion or broader commands.

