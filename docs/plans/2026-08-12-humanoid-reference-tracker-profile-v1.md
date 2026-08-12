# Humanoid reference tracker profile V1

| Field | Value |
|---|---|
| Status | Frozen TRAIN-5 input |
| Architecture | [ADR-070](../architecture/adr/070-biomechanics-reference-tracking-training-environment.md) |
| Machine profile | Linux x86_64, RTX 3080 10 GiB |
| Body | `nextengine.body.humanoid-biomechanics-raja-1700.v2@2` |
| BodySchema hash | `e2460e7dc4af93538ae4b0b68a9e1bf74b2b7990161e08e441d58687e953c43d` |
| Safety/contact profile | `ad20d7a4abd5cc8b59069ecdb59161499ce7754953cbff2477f2850395adb42c` |
| Corpus manifest | `24629cf222fded5c0a80755f7442922527dde829c49f4369d1f88cddc92d0196` |
| Canonical JSON | `lab/profiles/humanoid-reference-tracker.v1.json` |
| Canonical JSON SHA-256 | `7f1ea0faff1707767b1145070ed3d1ac4b60b287b40ed87865e520e5807079d8` |

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

The profile explicitly lists the 23 observation/reference joints in DoF order,
the six effectors, seven contact flags and 23 action actuators. Because action
channels use canonical actuator-ID order while reference joints use DoF order,
the frozen `reference_joint_dof_ordinal_by_action_channel` permutation is the
only admitted conversion between them.

Reward emphasizes joint pose, root/effector/contact agreement, with smaller
velocity terms and low energy/smoothness costs. The tenfold terminal-failure
coefficient is diagnostic only: hard ROM, actuator, impact and forbidden
contact failures terminate independently and cannot be traded for reward.
Every vector/scalar aggregation, normalization, coordinate frame, Q16 rounding
rule and the authorizing TRAIN-4 gate-report SHA are frozen in the canonical
JSON; changing any of them produces a new profile identity before a run.

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

The canonical native PhysX path is the required first execution target. An
Isaac mirror remains an additional correspondence target and never substitutes
for native safety evidence. Hardware availability by itself is not a TRAIN-5
result.

Current pre-acceptance evidence distinguishes baseline execution from policy
quality. Zero residual completes the admitted idle reference, while dynamic
walk/start/stop/turn clips normally lose tracking or reach a hard contact
terminal without a learned residual. Those baseline failures are retained as
optimizer-free diagnostics; a safety event, hash mismatch or malformed reward
remains blocking, but poor tracking by the zero/random actor is not mislabeled
as a trained-policy result.

The current corpus lineage is the ankle-roll-reserved TRAIN-4 revision. Its
input-closure audit SHA-256 is
`14977226e0d7d86ac6c5c13cb45a9059f230633cb13b454dc1cfd8f7fe8efccc` and
records zero optimizer steps. The baseline matrix and tiny/curriculum results
below bind the superseded corpus and remain historical diagnostics only; none
of their checkpoints may initialize the current lineage.

The closed input/reward audit has SHA-256
`b76fcf1d9ce8ef0b7df2c5d35a2c6b115a7c458c1f8a4a90fcb1898f6d0d1489`.
It covers 84 observation/perfect-reference fixtures, 84 phase-offset corpus
probes, 84 directed cost probes and 84 terminal-penalty probes. No tracking
component is constant; the largest positive component is `2857` basis points
of positive reward scale against the declared `5000` rejection threshold.
The nine-case native matrix has SHA-256
`eaa3c81b3cf120318d3cf62f4aff587fac7f56a5a31f113958f9820a5ab6c9a6`
and decision `AdvanceToTinyDeterministicOverfitOnly`. Both artifacts record
zero optimizer steps and make no learned-policy claim.

## Claim boundary

This profile authorizes only locomotion reference tracking. Recovery data,
commands, perturbation pushes, portable export and runtime learned-policy
publication remain outside TRAIN-5. Report-only quality targets never turn a
hard safety or visual failure into `Pass`.
