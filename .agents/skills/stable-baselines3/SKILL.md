---
name: stable-baselines3
description: Implement or debug an experiment that explicitly uses Stable-Baselines3 with a Gymnasium-compatible single-agent environment. Do not use for generic RL algorithm selection or NextEngine humanoid/Isaac TRAIN runs, which use the project-specific training skills.
license: MIT license
allowed-tools: Read Write Edit Bash
metadata:
  version: "1.1"
  skill-author: "K-Dense Inc."
  compatibility: Requires Python 3.10+, PyTorch >= 2.3, and stable-baselines3 2.8+. Gymnasium environments; optional extras for TensorBoard and Atari (ale-py).
---

# Stable-Baselines3

Use this library skill only after SB3 is selected by the user or existing code.
The primary artifact is a working training/evaluation result, not a generic RL
stack.

## Minimal workflow

1. Inspect the environment's observation/action spaces and existing dependency
   versions. Do not install or upgrade SB3 unless the requested run requires it.
2. Validate a custom environment with `check_env`.
3. Choose only among SB3 algorithms compatible with the action space:
   PPO/A2C are general on-policy baselines, DQN is discrete-only, and SAC/TD3
   are continuous off-policy choices.
4. Run the smallest smoke/overfit experiment that proves environment and
   training plumbing, then evaluate on a separate fixed episode set.
5. Save normalization state alongside the model when `VecNormalize` is used.
6. Report the actual evaluation result and failure modes; training return alone
   is not sufficient.

Use the provided resources only for the requested feature:

- [references/algorithms.md](references/algorithms.md) for a real SB3 algorithm
  choice;
- [references/custom_environments.md](references/custom_environments.md) for a
  custom Gymnasium environment;
- [references/vectorized_envs.md](references/vectorized_envs.md) when profiling
  shows environment throughput is limiting;
- [references/callbacks.md](references/callbacks.md) when a concrete monitoring
  or stopping behavior is required;
- [scripts/train_rl_agent.py](scripts/train_rl_agent.py),
  [scripts/evaluate_agent.py](scripts/evaluate_agent.py) and
  [scripts/custom_env_template.py](scripts/custom_env_template.py) as optional
  starting points.

Do not add callbacks, checkpoints, vectorization, video, TensorBoard,
hyperparameter search or alternate algorithms by default. Add one only when it
answers the current experiment or fixes an observed bottleneck.

For NextEngine TRAIN profiles, stop and use
`nextengine-training-runner`/`nextengine-training-diagnostics`; SB3 examples
must not replace the project's admitted generation, Isaac or evaluation
contracts.
