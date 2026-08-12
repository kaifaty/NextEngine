# Humanoid reference tracker profile V1

| Field | Value |
|---|---|
| Status | Frozen TRAIN-5 input |
| Architecture | [ADR-070](../architecture/adr/070-biomechanics-reference-tracking-training-environment.md) |
| Machine profile | Linux x86_64, RTX 3080 10 GiB |
| Body | `nextengine.body.humanoid-biomechanics-raja-1700.v2@1` |
| Corpus manifest | `e6f53d749292b3c6b0e2a7642711c8e1a9f6cef747d184ebb3a21e5e62ffad43` |
| Canonical JSON | `lab/profiles/humanoid-reference-tracker.v1.json` |
| Canonical JSON SHA-256 | `cdc428435ffc142b1b9cf4de5ebf44b73797dddf1d1e67415384c8a9f6bb2211` |

V1 is the smallest tracker profile that can consume the admitted locomotion
partition without introducing command selection, recovery, a runtime
`PhysicalActionChunk` or privileged critic state. Exact profile fields live in
the canonical JSON; this document records the rationale and claim boundary.

## Fixed choices

The four reference offsets `0/4/8/16` expose current, 67 ms, 133 ms and 267 ms
targets. This covers immediate contact response plus one bounded anticipation
window while remaining small enough for the first feed-forward MLP.

Reset weights start at 70% exact reference, 20% small perturbation and 10%
neutral entry. Exact states establish whether reference/action/reward wiring is
correct; perturbations prevent a pure open-loop solution; neutral entry tests
the actual idle/start boundary. Near-failure sampling stays exactly zero until
nominal tracking passes, so it cannot obscure a basic tracker failure.

The actor and critic both receive 435 ordered integer channels. Keeping the
first critic unprivileged removes a hidden-information variable from the sanity
ladder. A later privileged critic requires a new profile hash and explicit
training-only channel list.

Reward emphasizes joint pose, root/effector/contact agreement, with smaller
velocity terms and low energy/smoothness costs. The tenfold terminal-failure
coefficient is diagnostic only: hard ROM, actuator, impact and forbidden
contact failures terminate independently and cannot be traded for reward.

## Execution ladder

1. Validate manifest/profile/corpus hashes and deterministic clip/phase
   selection without Physics.
2. Run random residual, zero residual and reference-following baselines through
   the canonical CPU environment; reject constant, non-finite, dominating or
   wrong-sign reward components.
3. Prove CPU/Isaac reference-feature and applied-target correspondence on the
   fixed fixture corpus.
4. Overfit one phase/clip with one declared seed.
5. Only then begin the frozen multi-clip PPO curriculum and pre-registered
   multi-seed held-out report.

Isaac Lab/Isaac Sim are not part of the repository profile and are currently
an unavailable external capability until installed. Hardware availability by
itself is not a TRAIN-5 result.

## Claim boundary

This profile authorizes only locomotion reference tracking. Recovery data,
commands, perturbation pushes, portable export and runtime learned-policy
publication remain outside TRAIN-5. Report-only quality targets never turn a
hard safety or visual failure into `Pass`.
