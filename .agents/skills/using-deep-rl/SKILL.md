---
name: using-deep-rl
description: Use when training, debugging, or selecting a deep-RL algorithm — value-based (DQN/Rainbow/BBF), policy-gradient (PPO/GRPO), actor-critic (SAC/TD3/REDQ), model-based (DreamerV3/TD-MPC2), offline (CQL/IQL/Decision Transformer), multi-agent (MAPPO/IPPO), exploration, reward shaping, or counterfactual credit assignment. Routes to the matching specialist sheet by problem type and algorithm family.
---

# Using Deep RL

Apply the [shared execution guidance](../astra-guidance.md) once per task alongside this skill; it governs process defaults in the references too.

Select the reference that addresses the user's actual RL task. Establish action
space, online/offline data access, objective and compute constraints from the
request, environment and existing code. Ask only about a missing fact that
would change the next decision; do not interview the user about facts already
available or require a foundations lesson before implementation.

## Respect the active workflow

For NextEngine, use its training runner for authorized runs, diagnostics for run
analysis, and architecture skill for contracts. Treat generic RL recipes as
method references; they do not replace frozen profiles, select a new engine
trainer or authorize additional compute. For an explicitly requested algorithm,
preserve the choice and explain concrete incompatibilities before substituting.

## Route by problem

| Concern | First reference |
| --- | --- |
| Requested explanation of MDPs, value or policy | rl-foundations.md |
| Discrete value-based control | value-based-methods.md |
| PPO, TRPO, REINFORCE or general GRPO | policy-gradient-methods.md |
| Continuous off-policy control | actor-critic-methods.md |
| Learned dynamics or planning | model-based-rl.md |
| Fixed dataset without new interaction | offline-rl.md |
| Multiple interacting learning agents | multi-agent-rl.md |
| Sparse exploration or intrinsic reward | exploration-strategies.md |
| Reward definition or invariance | reward-shaping-engineering.md |
| Causal credit, HER or counterfactual evaluation | counterfactual-reasoning.md |
| Flat reward, divergence or unstable learning | rl-debugging.md |
| Environment API, reset, wrappers or vectorization | rl-environments.md |
| Evaluation, variance or checkpoint comparison | rl-evaluation.md |

Standard DQN needs a discrete action formulation. Fixed-data learning needs
an offline-compatible method and evaluation; do not assume an online recipe
applies unchanged. Diagnose environment, data and optimization evidence before
an algorithm switch. Read only the next relevant sheet; routing never means
ending the task or waiting for a specialist to be installed.

## Finish the requested work

Use a bounded smoke or targeted experiment before a large run when it resolves
an actual uncertainty. Preserve agreed compute budgets, seeds, checkpoint
selection and evaluation criteria. Do not launch training for a diagnosis-only
request or turn a pipeline check into a statistical quality claim.

Keep the answer focused on the result, evidence and smallest unresolved issue.
LLM-specific trainer recipes and deployment workflows require suitable tools
when needed; references to other packs are not proof that they are installed.

## Topic references

Read only the reference relevant to the next decision. Paths are relative to this skill.

- [rl-foundations.md](rl-foundations.md) - MDP formulation, Bellman equations, value vs policy basics
- [value-based-methods.md](value-based-methods.md) - Q-learning, DQN, Double DQN, Dueling DQN, Rainbow
- [policy-gradient-methods.md](policy-gradient-methods.md) - REINFORCE, PPO, TRPO, policy optimization
- [actor-critic-methods.md](actor-critic-methods.md) - A2C, A3C, SAC, TD3, advantage functions
- [model-based-rl.md](model-based-rl.md) - World models, Dyna, MBPO, planning with learned models
- [offline-rl.md](offline-rl.md) - Batch RL, CQL, IQL, learning from fixed datasets
- [multi-agent-rl.md](multi-agent-rl.md) - MARL, cooperative/competitive, communication
- [exploration-strategies.md](exploration-strategies.md) - ε-greedy, UCB, curiosity, RND, intrinsic motivation
- [reward-shaping-engineering.md](reward-shaping-engineering.md) - Reward design, potential-based shaping, inverse RL
- [counterfactual-reasoning.md](counterfactual-reasoning.md) - Causal inference, HER, off-policy evaluation, twin networks
- [rl-debugging.md](rl-debugging.md) - Common RL bugs, why not learning, systematic debugging
- [rl-environments.md](rl-environments.md) - Gym, MuJoCo, custom envs, wrappers, vectorization
- [rl-evaluation.md](rl-evaluation.md) - Evaluation methodology, variance, sample efficiency metrics
- [multi-skill-scenarios.md](multi-skill-scenarios.md) - Common problem routing sequences
