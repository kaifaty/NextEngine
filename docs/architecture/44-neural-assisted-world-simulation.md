# SPEC-44: Proposed neural-assisted world simulation

| Field | Value |
|---|---|
| ID | SPEC-44 |
| Status | Proposed |
| Version | 1.1 |
| Last verified | 2026-08-17 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [SPEC-34](34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-38](38-continuum-material-physics.md), [SPEC-39](39-layered-physical-world.md), [SPEC-40](40-structural-vegetation-physics.md), [SPEC-41](41-world-substrate-composition.md), [SPEC-43](43-thermochemical-material-processes.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-053](adr/053-engine-native-model-training-and-immutable-artifact-boundary.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-080](adr/080-neural-assistance-as-bounded-proposals.md), [ADR-081](adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md) |
| Related research | [Imported world-dynamics source papers](research/world-dynamics-source-papers.md) |
| Candidate revision note | Version 1.1 narrows N0/N1 to report/shadow-only work; runtime solver advice now requires a later certificate-backed Accepted ADR |

## Status and purpose

This SPEC defines report/shadow evaluation of a learned component against an
already selected classical world solver without making it authority, a second
state owner, a runtime advisor or a correctness fallback. It does not claim a
dataset, model, runtime provider, accelerated solver or ProductCheck PASS.

The classical CPU authority and its reference, persistence and production
gates are prerequisites. Neural work is never on the critical path to water,
vegetation, thermochemical, arcane or v1 promotion. The first allowed study is
a report/shadow warm-start proposal for one already promoted classical owner;
production always executes its default path.
No domain is selected until its baseline exposes a measured iterative cost and
a consumer-backed acceleration target.

The standalone [neural world-physics roadmap](../plans/neural-world-physics/README.md)
remains `PLANNED / NOT_ACTIVE` until one owner passes its classical reference,
exact persistence and production baseline gates.

## Authority and trust boundary

The learned component is a stateless proposal producer. It owns no world
field, particle, contact, temperature, phase, topology, resource, command,
event or checkpoint. The target owner remains the sole writer and runs its
unchanged default initialization, canonical solver and publication path. A
separate shadow evaluator may compare a proposal-run result after production
authority has already been selected.

The source paper's trust ladder is narrowed for Next Engine:

| Tier | Allowed status | Meaning |
|---|---|---|
| N0 diagnostics | allowed after baseline instrumentation | Predict residual, relevance or cost for telemetry only; never changes work or state. |
| N1 proposal | report/shadow-only | Suggest bounded initial iterate, preconditioner parameter or active-set ordering to a non-authoritative shadow solve; production never consumes it. |
| N2 learned correction | not admitted | Requires a separate consumer-backed ADR proving exact owner semantics and failure closure. |
| N3 surrogate constitutive law | not admitted | Cannot replace a profile law or material owner. |
| N4 learned region evolution | not admitted | Cannot publish state or choose representation. |
| N5 learned world simulation | rejected | Conflicts with exclusive owners, exact replay and engine-owned laws. |

Training, evaluation and model packaging follow SPEC-34/ADR-053. This SPEC
does not extend current `ModelLaneV1` or add a public physical-model schema.
Exact lane/profile contracts land only with the first production consumer.
Online training, runtime weight updates, self-modifying models and model output
entering a command directly are forbidden.

## Classical baseline first

Before collecting an admissible accelerator dataset, the target domain must
have:

1. a frozen owner/profile and canonical state boundary;
2. a serial/reference corpus with declared error and failure thresholds;
3. a deterministic production solver with exact save/restart;
4. stable instrumentation for iteration count, residual and bounded cost;
5. a representative performance baseline and a named bottleneck;
6. a complete deterministic default initialization path.

A learned proposal cannot compensate for an unspecified law, unclosed numeric
profile, failing solver or missing persistence. Generated data from a changing
teacher lineage is not merged into one dataset generation.

SPEC-38 explicitly excludes warm start from the base water V1 path. Therefore
a water proposal is a separate post-promotion shadow branch and gives no credit to
`CONTINUUM-WATER-REF-P1` through `W6`. The same rule applies to vegetation and
thermochemical base roadmaps.

## Data and artifact lineage

Every sample binds target owner/profile, exact project/content/runtime/numeric
hashes, canonical pre-state root, canonical inputs, teacher solver identity,
proposal target schema, teacher result root, convergence/failure class and
declared metrics. Split membership is fixed before training by stable scenario
identity; near-duplicate trajectories cannot cross train/evaluation groups.

Only canonical committed or independently labelled teacher facts enter the
authoritative dataset manifest. Raw private solver scratch may be recorded as
a bounded training feature only when its producer/profile hash and units are
closed and it contains no protected content or secret. Dataset, runs,
checkpoints and heavy model artifacts remain in the configured external store,
not the repository.

An immutable model bundle binds architecture, weights, feature/normalization
profiles, target proposal schema, teacher/dataset generation, training run,
provider requirements, resource ceilings and quality/performance reports.
Unknown or incompatible lineage rejects the bundle before inference.

## Report/shadow proposal evaluation

One shadow evaluation may consume at most one closed proposal batch for its
declared key:

```text
NeuralSolverProposalV1 {
  proposal_profile_id,
  model_bundle_hash,
  target_owner_id,
  target_profile_hash,
  prior_state_root,
  input_root,
  tick,
  substep,
  proposal_kind,
  bounded_fixed_point_payload,
  proposal_hash,
}
```

This is an illustrative private shape, not a public contract. Proposal output
is quantized and range-checked before the shadow solver sees it; raw tensors,
provider handles and device buffers never cross the owner boundary.

One proposal key is exactly `(target_owner_id, target_profile_hash,
prior_state_root, input_root, tick, substep, proposal_kind)`. A closed batch
admits at most one record for that key. Exact duplicates, conflicting hashes or
multiple model winners reject the shadow batch and record one bounded
diagnostic; arrival, confidence or completion time never chooses a winner and
production work is unchanged.

The fixed flow is:

1. freeze the same canonical input used by the production classical solver;
2. run and publish the owner's unchanged deterministic default path;
3. after authority is selected, optionally evaluate the immutable model under
   bounded report/shadow resources;
4. canonicalize and either reject the proposal or apply exactly one to a
   non-authoritative copy of the same classical solver;
5. compare result roots, failure class and cost, then record only bounded
   telemetry; never publish the shadow result or exchange.

Missing, late, malformed, incompatible, nonfinite or out-of-range model output
only removes that shadow sample. Shadow non-convergence, overflow or invariant
failure is telemetry and cannot reject, delay, retry or otherwise change the
already selected production step.

## Exact-result and failure requirements

A proposal profile is useful research evidence only if its shadow proposals
preserve exact canonical output roots, outcome order and failure classification
versus the default classical path on the predeclared corpus, all supported
targets and the required continuation window. Approximate field similarity is
not enough even for a future authoritative proposal.

The research harness may run shadow default and proposal candidates to measure
this relation. Runtime does not publish or persist the shadow. If exact-root
closure cannot be demonstrated, the model stays N0/report-only regardless of
speed or average error.

Model inference must not choose production timestep, iteration stop,
representation tier, owner set, query result, command/event or gameplay
outcome. An N1 warm start may reduce work only in its shadow solve under a
frozen solver schedule/termination profile. GPU authority, tolerance-based root
acceptance and mid-run classical/GPU switching are forbidden.

Runtime solver consumption requires a later consumer-backed Accepted ADR plus
a mechanically checkable admissibility certificate proving that every admitted
proposal lies in a domain with the same canonical result and failure class as
the default path. Corpus agreement, including every check below, is necessary
evidence but is not that proof.

## Scheduling, persistence and fallback

Inference receives immutable revision-bound data after one target-owner
boundary and cannot hold mutable ECS/backend access across `await`. Worker and
GPU completion order cannot choose production work. A bounded shadow job may
be dropped when late or unavailable; production never waits to race models.

No hidden recurrent state, adaptive normalization or online cache may affect a
proposal. Reconstructible feature/model caches are not saved. The authoritative
checkpoint remains the classical owner's exact segment; shadow bundle identity
may appear in diagnostics but is not required for restore.

The deterministic default is mandatory and uses the same owner, law, numeric
profile and commit. It is not permission for Jolt/Bullet/GPU authority, a
simpler material law or a frozen owner. Under this SPEC a project cannot
require the model for activation, performance policy or gameplay correctness.

## Safety, telemetry and stop conditions

Telemetry records proposal admission/rejection reason, inference cost,
classical iterations/cost, residual class, exact-root comparison in shadow
mode and end-to-end p95/p99. It contains no secrets, protected content, raw
vendor pointers or unbounded tensors.

The track stops after two evidence-backed optimization cycles if the selected
proposal does not improve the predeclared end-to-end bottleneck while
preserving exact roots and failure semantics. Reducing the classical product
gate, accepting tolerance, granting model/GPU authority or increasing the
frame budget requires a separate explicit decision; a faster approximate
result is not promotion evidence.

## Evidence and promotion

| Check | Required result |
|---|---|
| `WORLD-NEURAL-DATAPLANE-P1` | Dataset/model lineage is hash-closed, split-stable, bounded and reproducible from the frozen classical teacher without repository-stored heavy artifacts. |
| `WORLD-NEURAL-SHADOW-P1` | Report-only proposal evaluation records exact-root/failure agreement, residuals and cost over the predeclared held-out and adversarial corpus. |
| `WORLD-NEURAL-NONREGRESSION-P1` | Shadow proposal and default paths produce identical canonical roots, event/outcome order and failure classes across order/worker/save-continuation permutations; disagreement never faults production. |
| `WORLD-NEURAL-CROSS-TARGET-P1` | Windows/Linux shadow proposal bytes and resulting non-authoritative owner roots match for the frozen research profile. |
| `WORLD-NEURAL-PERFORMANCE-P1` | Fixed THOTH shadow runs report projected end-to-end p95/p99 effects with model inference, gate and correction costs included; this is not a production budget PASS. |

N0 diagnostics may begin after the classical baseline. N1 remains report/
shadow-only even after these checks pass. Runtime consumption requires all
checks, affected owner checks, `persistence-replay`, conditional `platform` and
`performance`, a production consumer, the mechanical safety certificate and a
later Accepted successor ADR that explicitly supersedes ADR-081's restriction.

No neural check can close a classical solver, physical coupling, persistence
or world-substrate gate. `WORLD-DYNAMICS-P1` also receives no credit from a
model because the model is not a substrate owner.
