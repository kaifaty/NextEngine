# ADR-078: World substrate and arcane physical-interaction track

| Field | Value |
|---|---|
| ID | ADR-078 |
| Status | Proposed |
| Version | 1.3 |
| Decision date | 2026-08-16 |
| Last verified | 2026-08-17 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](../18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-19](../19-rpg-domain-and-narrative-state.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](../25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](../30-presentation-extraction-and-render-content.md), [SPEC-31](../31-autonomous-quest-lifecycle-and-narrative-director.md), [SPEC-38](../38-continuum-material-physics.md), [SPEC-39](../39-layered-physical-world.md), [SPEC-40](../40-structural-vegetation-physics.md), [SPEC-41](../41-world-substrate-composition.md), [SPEC-42](../42-arcane-substrate-and-physical-magic.md), [ADR-008](008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-019](019-canonical-player-actions-and-presentation-authority.md), [ADR-020](020-rpg-domain-authority-and-extension-boundary.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-027](027-physics-motor-and-animation-layering.md), [ADR-034](034-player-targeting-replay-v5-and-mapping-provenance.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-071](071-canonical-physics-material-lineage.md), [ADR-077](077-layered-physical-world-and-living-structures-track.md), [ADR-081](081-world-dynamics-gap-closure-and-promotion-guardrails.md) |
| Candidate revision note | Version 1.3 applies ADR-081 successor-stage, composition, analytical-debit, checkpoint-epoch, capacity, fault-domain, identity and budget guardrails |
| Superseded by | Partially [ADR-081](081-world-dynamics-gap-closure-and-promotion-guardrails.md): it supersedes current-stage access, rollback-to-continue, incomplete debit, implicit fault-domain and legacy-budget clauses. |
| Related Proposed tracks | [SPEC-43](../43-thermochemical-material-processes.md), [SPEC-44](../44-neural-assisted-world-simulation.md), [ADR-079](079-thermochemical-material-process-track.md), [ADR-080](080-neural-assistance-as-bounded-proposals.md) |

## Context

The physical-world research paper correctly converges on one causal world with
specialized owners, typed coupling, representation transitions, persistence
and semantic queries. Most of that requirement is already captured more
strictly by SPEC-39. Copying its proposed `PhysicalWorldCore`, generic domain
traits, global representation manager, multi-rate scheduler, network layer and
large crate tree would create infrastructure before a second production owner.
It also conflicts with current CPU authority, exact active persistence and
camera-independent LOD where it recommends GPU authority, lossy sleep or
visibility-driven relevance.

The arcane paper adds a useful new concern: an explicit source/sink substrate
whose resources can cause real physical work. It also spans fields, ecology,
organisms, artifacts, healing, identity, divine authority and ontological
effects before any product profile or current owner exists. Treating all of
that as one magic system would duplicate RPG, Mechanics and Physical
Embodiment authority and violate consumer-driven contracts.

## Proposed decision

### Use owner composition, not a universal world service

Adopt SPEC-41 as a conceptual `WorldDynamics` composition only. Runtime remains
the schedule/ledger/commit owner; RPG and Mechanics retain their Accepted
stores and public paths; Physical Embodiment retains physical state. New
substrates join as peer owners through revision-bound immutable projections,
typed exchange batches and one declared all-or-nothing transaction.

Do not add a generic domain trait, global quantity map, shared mutable world
database, universal `CouplingGraph` API or `RepresentationManager` contract.
The current twelve-stage profile remains unchanged. The first non-physical
owner uses an explicit successor profile with `WorldDynamicsStep` at stage 8,
a closed runtime-owned owner/edge DAG, one merged PhysX integration and an
all-or-none fail-stop transaction.

### Add one bounded Arcane owner

Create the Proposed SPEC-42 arcane lane. It owns only arcane quantity,
reservations, throughput and active execution state. RPG continues to own
skills, inventory/equipment and current character resources. Mechanics owns
ability/package definitions and validates proposals; it owns no mutable V1
telekinesis reducer state. PhysX remains the only rigid writer. Mana is not
health, physical energy already deposited in another owner, soul, identity or
divine standing.

The candidate authority is checked CPU fixed-point arithmetic/publication.
`f64` is oracle/diagnostic-only for A1-A5 and cannot select a request, debit,
branch or root. GPU is optional presentation/correspondence only. Every
physical effect has a named source, finite debit, conversion/loss receipt and
atomic destination result. No effect receives permission to set transforms,
delete material or emit damage as a substitute for the destination owner.

### Split command start from physical exchange

Do not extend the current command ledger with a receipt pending across stage 5
and stage 8. The ordinary Ingress transaction creates the Arcane-owned active
execution, recast lock and one conservative reservation for the complete
finite declared plan, partitioned into canonical per-substep maximum-debit
slices. It then finalizes a command receipt whose meaning is `execution
started`. Mechanics retains only immutable V1 policy and no mutable
telekinesis reducer/cooldown state.

Each active execution emits bounded fixed-point requests at `WorldDynamicsStep`
without another package callback. Arcane and PhysX build candidate states,
validate work/loss and publish both plus an exchange receipt atomically. Release
and cancel are later `WorldCommand` values. Completion facts use the single
existing stage-9 Outcome batch and never re-enter the current tick.

### Start with one telekinesis vertical

The first production-shaped consumer is one package-authored telekinesis
ability acting on one real dynamic PhysX crate through the ordinary
`PlayerActionFrame`/authoritative targeting, `EffectRequestV1` and
`WorldCommand` path. One authored caster reservoir is sealed, closed and has no
regeneration or energy recovery. `1 AQ` is a one-joule pre-loss maximum
mechanical-work budget. V1 applies force at a point plus optional free torque;
impulse is excluded.

The fixture pins caster and crate in one sealed active region and forbids
streaming, transfer and despawn. A0A architecture closure is complete. A0B must
still freeze integer raws/scales, capacity/throughput values, efficiency and
maintenance coefficients, cadence/duration, exact force curve, analytical
traces, capacities, thresholds and performance budget before code.

The maximum-debit proof is analytical rather than corpus-observed. It includes
linear and angular speed bounds, application-point-to-CoM lever arm, force and
free torque, cadence/duration, maintenance, conversion loss and all rounding
bounds. Insufficient full-plan reservation rejects before freeze; a later debit
above its slice is an unreachable invariant.

The private `ArcaneRigidExchangeV1` compiles into existing
`PhysicsStepInputV1.external_force_requests`; no generic bus or new public
physics-step collection is introduced. Exact command retries stop in the
ledger. Duplicate exchange keys inside a newly built closed batch are fatal
internal invariants. Concurrent executions sort canonically and reserve
sequentially from one Arcane candidate state at start. Physical substeps
account already published current slices in the same order and never reserve
the same work twice.

### Defer general spell graphs and metaphysics

The first ability compiles to a closed typed execution plan. A generic bounded
spell graph waits for at least two production abilities that need shared
composition. Ambient/regional fields, ley networks, ecology, channels as
detailed anatomy, artifacts, runes and anti-magic are later packages with
their own consumers and state/evidence.

Thermochemical, continuum and vegetation coupling can start only after the
destination owner's relevant gate passes. Arcane heat/cooling uses SPEC-43
enthalpy and `ARCANE-THERMOCHEMICAL-P1`; it never directly selects
temperature, phase, ignition or damage. Vital/tissue, soul/identity, divine remote
sources, teleportation, matter creation, transformation, resurrection and
causality/time changes require separate SPEC/ADR. SPEC-31's inert divine-
standing intent does not grant arcane or identity authority.

### Exact active state before summaries

The first Arcane owner stores complete future-affecting active state, recast
locks and exchange receipts in one required owner segment of a successor
current-only composite checkpoint. Arcane and PhysX restore into staging and
publish together; absent Arcane state cannot default to zero. Exact save/
restart precedes regional summaries, field LOD, sleep or lossy ecology
persistence. Representation changes use canonical facts and integer budgets;
camera, visibility, measured frame time, wall clock and GPU completion are
forbidden.

Exact PhysX continuation uses fixed scheduled checkpoint epochs; save waits for
the same canonical rehydration barrier executed by uninterrupted comparison
runs.

## Failure and fallback

Capability denial, insufficient quantity/throughput, ineligible or pre-freeze
missing target and explicit cancel are ordinary per-execution rejections or
terminations; they apply no force and return unused reservation. Worst-case
execution, target, record and aggregate-wrench capacities are also
admitted before freeze; an expressible denial is ordinary. Post-freeze
missing/stale participant, nonfinite/overflow, duplicate key, work mismatch,
PhysX rejection, capacity exhaustion beyond reservation or corrupt persistence
is fatal and rejects the complete participating `WorldDynamicsStep`. The first
primary-gameplay profile faults the whole application session via ADR-081; only
independently provisioned test/training scenes may isolate a narrower slot.

Before activation, projects may omit arcane content or author an ordinary
mechanic. A project requiring unsupported arcane capability fails activation.
After activation there is no free cast, scripted transform/damage, VFX-only
success, frozen owner, GPU switch or retry-to-green.

## Promotion and stop conditions

The [arcane roadmap](../../plans/arcane-world/README.md) remains
`PLANNED / NOT_ACTIVE` while numeric Package A0B is open. After A0B closure the
serial reservoir/transfer oracle may run. Only
`ARCANE-RESERVOIR-REF-P1 = PASS` allows the main R8 row to become an active
research track.

Production promotion additionally requires `ARCANE-MECHANICS-P1`,
`ARCANE-RIGID-COUPLING-P1`, `ARCANE-PERSISTENCE-P1`,
`ARCANE-CROSS-TARGET-P1`, `play`, `content-package`, `persistence-replay`,
`platform`, the declared conditional performance budget and a consumer-backed
Accepted ADR. A2 freezes Proposed schema/schedule detail; public contracts land
only in the coherent integrated A3 consumer checkpoint. Failing the closed
law/coupling corpus keeps the track research-only; changing the source model,
physical budget, target scope or GPU authority requires an explicit new
decision.

Integrated performance authority is the successor mutually exclusive
`world-dynamics-step` row measured across every physical substep, merge,
validation and publication in one gameplay tick; standalone numbers are stop
targets only.

## Alternatives rejected

- One external command receipt pending from Ingress through the stage-8
  dynamics step:
  conflicts with the current two-phase ledger/barrier contract; start plus
  exchange receipts preserve it.
- PhysX-first publication or irreversible debit-first publication: either can
  expose a partial owner result when later validation fails.
- Per-substep package callbacks: create hidden re-entry and make package
  runtime timing part of authority.
- Authoritative private `f64`: unnecessary for the first reservoir/wrench law
  and weaker than the current fixed-point numeric baseline.
- New public `arcane_requests[]` in `PhysicsStepInputV1`: the first consumer can
  compile a private exchange record into the existing external-force path.
- Camera/depth or package-supplied target IDs: do not preserve authoritative
  targeting/query provenance across live, headless and replay.
- Hardcoded `FireballSystem`, `HealSystem` or privileged first-party magic:
  violates ADR-008/020 and prevents package dogfooding.
- One scalar mana plus scripted physical effects: cannot close source,
  throughput, conversion and rollback semantics.
- Full arcane field/ley/ecology before one rigid consumer: state, solver and
  persistence cost have no product evidence.
- A universal spell graph in the first milestone: no second consumer proves
  reusable nodes, execution semantics or public extension need.
- Direct physical mutation: creates a second rigid/continuum/tree writer.
- Mana equals health, soul or identity: collapses independent authority and
  makes resurrection/divine semantics accidental.
- Multiplayer/network replication in the base track: outside SPEC-00 v1 and
  unnecessary to prove offline systemic magic.

## Consequences

- SPEC-41, SPEC-42 and this ADR are Proposed; no current schema, crate, save,
  command kind, package primitive or ProductCheck is added.
- SPEC-39 remains the physical owner/coupling specialization; SPEC-41 composes
  it with non-physical substrates without superseding it.
- The first Arcane-to-PhysX edge is fully gated by
  `ARCANE-RIGID-COUPLING-P1`; `WORLD-DYNAMICS-P1` waits for one transaction
  involving two promoted new substrate owners beyond Physical Embodiment.
- The next action is A0B numeric profile/fixture/evidence closure, not arcane
  runtime code.
- Future magic effects receive completion credit only from their real
  destination-owner checks, never from presentation or an arcane-only test.
