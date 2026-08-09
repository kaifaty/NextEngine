# ADR-053: Engine-native model training and immutable artifact boundary

| Field | Value |
|---|---|
| ID | ADR-053 |
| Status | Proposed |
| Version | 1.1 |
| Decision date | 2026-08-09 |
| Last verified | 2026-08-09 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-32](../32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [SPEC-33](../33-behavior-policy-training-evaluation-and-deployment-lifecycle.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-005](005-offline-first-ai-process-boundary.md), [ADR-009](009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](048-direct-exact-project-lock.md) |
| Supersedes | none while `Proposed` |
| Superseded by | none |

## Context

For Strategic and Tactical roles this is an optional R8 quality track under
ADR-056, not a dependency of deterministic R4 or v1. Motor use remains an
independent optional profile behind the procedural R5 baseline.

Next Engine needs one reproducible path from production-shaped simulation to
trained Strategic, Tactical and Motor artifacts. The current repository has a
small isolated motor-learning smoke experiment, but no production environment
protocol, shared trajectory schema, dataset lineage, run lifecycle, promotion
boundary or runtime evaluator integration. Treating an experiment, a model
file or a framework-specific trainer as a platform would bypass the engine's
ownership, replay, package and provenance rules.

The first need is first-party R&D, not a public creator product. A creator SDK,
stable training CLI and UI would add compatibility obligations before one
production lane has proved the data plane. The engine must nevertheless use
the same public gameplay observations, proposals, validators and committed
outcomes that a future creator route would use.

## Decision candidate

### Product boundary

The first version is an internal first-party training platform. It provides
versioned manifests, environment adapters, trajectory capture, evaluation,
export and immutable artifact publication. It does not promise a creator SDK,
interactive training UI, hosted service or general experiment platform.

The platform has three independent lanes:

```text
Strategic behavior → Tactical behavior → PhysicalAvatarIntent
                                      → Motion controller → Physics
```

They share the SPEC-34 data plane but do not share one network, optimizer,
hidden state or runtime authority. Each lane has its own observation, action or
candidate, recurrent-state, reward and evaluation profiles.

### Canonical environment and accelerated mirrors

Production `headless` is the canonical environment. It runs the production
stage order, command validation, persistence/replay and subsystem owners.
Labels and rewards read immutable projections, receipts and committed events;
the trainer cannot mutate ECS, RPG, physics, candidate masks or success facts.

Isaac Lab or another GPU simulator MAY be used only as an accelerated mirror.
It is a private replaceable adapter and must pass a versioned correspondence
corpus against production `headless`, followed by final evaluation in
production `headless`. A correspondence mismatch blocks promotion of every
artifact trained from the affected mirror/configuration. Faster simulation is
not authority.

### Determinism and statistical claims

The environment data plane uses canonical schemas, exact seeds and named RNG
streams. Given an exact environment manifest, initial closure and recorded
inputs, `headless` reset/step/trajectory/reward records and authoritative
decision roots must replay byte-exact where their governing SPEC requires
exactness. Parallel workers use stable episode/slot order; completion order,
wall time and worker count cannot change an episode identity or record.

Stochastic optimizer execution, especially on GPU, is not described as
byte-exact training. Candidate quality is accepted by a pre-registered
statistical evaluation manifest containing the exact seed set, train/held-out
split, episode count, evaluation mode, sample budget, metrics, confidence or
effect-size method, practical thresholds and baseline identities. All runs and
failures in that declared set are reported; a selected best seed cannot stand
for the set.

### Immutable artifacts and activation

Runtime model weights are immutable, content-addressed package artifacts.
Offline training, retraining, consolidation or dreaming always creates a new
candidate bundle. It never mutates an active bundle, world, project lock, save
or replay in place.

Publication validates the complete model/schema/provenance/license/evaluation
closure and atomically creates a new immutable revision or rejects it. An
active session never hot-swaps that revision. Activation requires an explicit
exact project-lock change and a new session; save compatibility and recurrent
state disposition validate before world creation. Absence or rejection of a
learned route preserves the declared deterministic planner/procedural fallback.

### Portable evaluator artifact

The public artifact manifest is vendor-neutral. It records an
`evaluator_format_profile_id`, exact model bytes/hash, engine-owned schema
hashes, required closed evaluator capabilities, target/resource bounds and
fallback. Framework, simulator, training kernel, provider session and device
types stay private.

The first proposed runtime format profile is portable ONNX using standard
operators, fixed bounded shapes and explicit recurrent state inputs/outputs.
It admits no custom Mamba/selective-scan operator, hidden session cache,
stochastic inference operator or provider-owned sampler. ONNX Runtime is a
private reference adapter, not a public gameplay type. If export, target parity
or runtime performance fails, that profile remains unpromoted and gameplay
uses the deterministic fallback.

## Artifact lifecycle

```text
Environment closure
  → trajectory/dataset closure
  → training run + candidate checkpoint (outside Git)
  → deterministic export
  → immutable candidate bundle
  → schema/correspondence/parity/quality/retention checks
  → explicit project-lock selection for a new session
```

Datasets, checkpoints, optimizer/replay-buffer state, run directories,
credentials, generated captures and model outputs remain outside Git. The
repository MAY contain bounded engine-owned schemas, tiny CC0/generated golden
corpora and validation fixtures.

## Alternatives considered

- A framework-owned public API (PyTorch/Gym/Isaac/MLflow types) — rejected
  because it reverses dependencies and couples gameplay contracts to a tool.
- GPU simulator as the canonical environment — rejected because production
  ownership, command semantics and replay live in `headless`.
- Runtime online learning or active-session hot swap — rejected because it
  creates mutable hidden authority and invalidates save/replay identity.
- ONNX as the outer public bundle contract — rejected; ONNX is the first
  proposed evaluator profile behind a vendor-neutral manifest.
- Creator SDK/UI in v1 — deferred until a first-party production consumer and
  stable schemas prove the workflow.

## Consequences

- SPEC-34 owns the common environment/trajectory/dataset/run/export records.
- SPEC-33 specializes the data plane for Strategic and Tactical policies.
- SPEC-14/SPEC-27 specialize it for Motor policies without changing their
  Accepted foundation/expert composition or procedural fallback.
- Training infrastructure can change without changing runtime gameplay
  contracts, provided exported artifacts pass the same closures and checks.
- Strategic/Tactical use of this data plane is optional R8 quality work and
  does not block deterministic R4 or v1. A learned Motor profile remains
  optional quality work and does not block procedural R5.

## Promotion boundary

ADR-053 and SPEC-34 may become `Accepted` only with a production consumer that
uses the common data plane end to end, rejects malformed provenance before
use, reproduces the canonical headless corpus, passes required mirror
correspondence and runtime export parity, and activates one immutable bundle
through an exact project lock. Documentation, a trainer scaffold or an
isolated smoke model is insufficient under ADR-046.
