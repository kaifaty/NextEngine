# ADR-055: Mamba-2 physical motion foundation profile

| Field | Value |
|---|---|
| ID | ADR-055 |
| Status | Proposed |
| Version | 1.0 |
| Decision date | 2026-08-09 |
| Last verified | 2026-08-09 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-28](../28-skeletal-animation-retargeting-and-ik.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-009](009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-013](013-self-contained-physical-avatar-boundary.md), [ADR-027](027-physics-motor-and-animation-layering.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-053](053-engine-native-model-training-and-immutable-artifact-boundary.md) |
| Supersedes | none while `Proposed`; current Accepted SPEC-14/SPEC-27 wire semantics remain in force |
| Superseded by | none |

## Context

The physical layer already accepts a universal foundation plus conditioned,
residual and exclusive routes, explicit recurrent state, engine-owned safety
clamps and a procedural fallback. What is missing is a concrete learned
foundation hypothesis that can retain longer motion context without moving
physics authority into a model runtime.

The research basis is:

- [Mamba-2](https://arxiv.org/abs/2405.21060)
- [MaskedMimic](https://arxiv.org/abs/2409.14393)
- [PHC](https://arxiv.org/abs/2305.06456)

These works motivate the profile and curriculum; they do not prove target
portability, deterministic parity, runtime budget or gameplay quality.

## Decision candidate

### Scope of the architecture choice

Mamba-2 is mandatory only inside the proposed learned universal foundation
profile. It is not mandatory for `ResidualSkillAdapter`, `ExclusiveExpert`, a
future learned reference generator, or the deterministic procedural fallback.
The Accepted ADR-009 foundation/conditioned/residual/exclusive composition
remains unchanged.

One intent/reference-conditioned foundation controller covers the declared
subset of locomotion, balance, transitions, hit reaction and recovery for one
morphology family. It runs at an exact manifest-bound rate in the 30–120 Hz
range and receives only the accepted SPEC-27 observation plus validated
`PhysicalAvatarIntent`/reference features. It does not read quest, biography,
free-form memory or raw tactical model state.

Strategic and Tactical policies remain upstream. Tactical selects a semantic
affordance/mode; the Motion Controller alone produces joint-level action, and
Physics remains the authority for contacts and outcomes.

### First action and safety profile

The first allowed Mamba-2 route emits bounded joint position and velocity
targets. Engine-owned fixed PD gains convert those targets through the current
SPEC-27 safety profile; torque, power, velocity and per-tick rate limits remain
engine-owned and are applied before the physics commit.

Adaptive stiffness/damping output requires a separate quality and safety gate.
Direct torque is admitted only as a research `ExclusiveExpert` route with its
own action schema and safety evidence; it is not the first foundation route.
Existing position/velocity, stiffness/damping and torque action semantics are
not changed by this Proposed decision.

A learned reference generator is a later upstream module. The first mandatory
chain consumes authored/procedural or dataset reference features; it does not
require a learned generator to run.

### Explicit Mamba state

All convolution and state-space caches are flattened into fixed-width segments
of the generic `PolicyStateSchemaV1`. Segment offsets, widths, numeric
descriptors, initial values and reset/handoff rules are content-addressed. The
complete fixed-point `PolicyStateRecordV1` is the only authoritative state.

Evaluator-session cache, dynamic sequence history, implicit warm state,
provider KV/SSM state and state reconstructed from arrival order are forbidden.
The one-step graph is a pure function:

```text
(observation_t, explicit_state_t) → (joint_targets_t, explicit_state_t+1)
```

### Training and portable export

Training MAY use fused selective-scan kernels and framework-native sequence
execution. Export must lower the profile to one portable fixed-shape ONNX step
graph using standard operators, explicit state input/output and no custom
Mamba/selective-scan op. The outer manifest remains vendor-neutral under
ADR-053; ONNX Runtime is only a private reference adapter.

Promotion requires trainer↔export↔runtime one-step and rollout parity on
Windows x86_64 and Linux x86_64, no hidden cache, fixed-PD bounds, transition,
perturbation, fall/recovery and performance suites, plus multi-seed held-out
comparison against procedural and GRU comparators. Failure of portability,
parity, safety, quality or budget leaves the profile `Proposed` and uses the
procedural route.

### Manifest evolution boundary

SPEC-14 currently requires ONNX-specific manifest wording, while SPEC-27 is
technology-neutral. This ADR proposes a future manifest field
`evaluator_format_profile_id` whose first value is the portable ONNX profile.
Because ADR-055 is `Proposed`, it does not supersede or silently change the
current Accepted wire schema. The field replacement occurs only in a future
consumer-backed promotion ADR/SPEC/schema change.

## Alternatives considered

- GRU foundation as the only profile — retained as the mandatory comparator,
  but not selected as the intended universal learned foundation hypothesis.
- Transformer with an unbounded context window — rejected for v1 because its
  runtime state/resource envelope is harder to bound and persist.
- One end-to-end Strategic/Tactical/Motion network — rejected because it
  collapses ownership, cadence, safety and fallback boundaries.
- Direct torque as the default foundation action — rejected due to the larger
  safety and transfer burden.
- Custom ONNX selective-scan operator — rejected from the first portable
  profile because it makes the artifact provider-specific.

## Consequences

- Mamba-2 becomes explicit in the physical R&D roadmap without becoming a
  shipping requirement.
- The accepted generic state/action/safety contracts carry the profile; no
  Mamba type enters public gameplay APIs.
- State width and one-step lowering may be expensive; measured target parity
  and performance decide whether the profile can be promoted.
- Procedural R5 remains independently closable.

## Promotion boundary

ADR-055 remains `Proposed` until a production Motor consumer, portable
standard-op export, explicit-state parity on both shipping targets, safety and
performance suites, and pre-registered multi-seed held-out quality gates all
pass. A framework checkpoint, fused training kernel or isolated ONNX smoke is
not a promoted physical policy.
