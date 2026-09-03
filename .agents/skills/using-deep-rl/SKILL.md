---
name: using-deep-rl
description: Select or implement a deep-RL algorithm when the algorithm family or RL formulation is the actual task. Do not use for supervised learning, ordinary model fitting, or running/diagnosing an existing NextEngine humanoid TRAIN profile; use the project training runner or diagnostics skill for those.
---

# Using Deep RL

Route one concrete RL question to one relevant reference. The outcome is a
working experiment, implementation or diagnosis—not a survey of every family.

## Scope gate

- Existing NextEngine run preparation/resume/evaluation:
  `nextengine-training-runner`.
- Existing NextEngine PPO failure or metric diagnosis:
  `nextengine-training-diagnostics`.
- CPU/Isaac mirror parity: `nextengine-isaac-correspondence`.
- Fixed input/target data with no sequential reward: this is supervised or
  offline prediction, not RL; do not use this pack.

## Minimal characterization

Establish only the facts that affect selection: action space, online versus
fixed data, single versus multi-agent, simulator cost, reward density and
evaluation budget. Ask the user only when these cannot be discovered locally
and a wrong assumption would change the experiment.

Then read at most one primary reference:

| Immediate problem | Reference |
| --- | --- |
| Concepts or MDP formulation | [rl-foundations.md](rl-foundations.md) |
| Discrete actions | [value-based-methods.md](value-based-methods.md) |
| PPO or policy gradients | [policy-gradient-methods.md](policy-gradient-methods.md) |
| SAC/TD3 or continuous off-policy control | [actor-critic-methods.md](actor-critic-methods.md) |
| Learned dynamics/planning | [model-based-rl.md](model-based-rl.md) |
| Fixed transition dataset | [offline-rl.md](offline-rl.md) |
| Multiple learning agents | [multi-agent-rl.md](multi-agent-rl.md) |
| Exploration failure | [exploration-strategies.md](exploration-strategies.md) |
| Reward design | [reward-shaping-engineering.md](reward-shaping-engineering.md) |
| Training is not learning | [rl-debugging.md](rl-debugging.md) |
| Gym/Isaac/environment API | [rl-environments.md](rl-environments.md) |
| Evaluation methodology | [rl-evaluation.md](rl-evaluation.md) |

Load a second reference only after evidence shows a genuinely cross-cutting
failure.

## Execution guard

- Debug environment, observations, actions and rewards before replacing the
  algorithm.
- Start with the smallest run that can falsify the choice, then produce its
  evaluation artifact.
- Compare a new algorithm against the existing baseline under the same useful
  budget. Do not create a family tournament when one comparison answers the
  request.
- Keep experimental claims separate from production or roadmap promotion.
