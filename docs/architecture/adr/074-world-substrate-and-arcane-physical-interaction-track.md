# ADR-074: World substrate and arcane physical-interaction track

| Field | Value |
|---|---|
| ID | ADR-074 |
| Status | Proposed |
| Version | 1.0 |
| Decision date | 2026-08-16 |
| Last verified | 2026-08-16 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-19](../19-rpg-domain-and-narrative-state.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](../25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](../30-presentation-extraction-and-render-content.md), [SPEC-31](../31-autonomous-quest-lifecycle-and-narrative-director.md), [SPEC-36](../36-continuum-material-physics.md), [SPEC-37](../37-layered-physical-world.md), [SPEC-38](../38-structural-vegetation-physics.md), [SPEC-39](../39-world-substrate-composition.md), [SPEC-40](../40-arcane-substrate-and-physical-magic.md), [ADR-008](008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-020](020-rpg-domain-authority-and-extension-boundary.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-027](027-physics-motor-and-animation-layering.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-071](071-canonical-physics-material-lineage.md), [ADR-073](073-layered-physical-world-and-living-structures-track.md) |
| Supersedes | none; proposes a post-v1 world-substrate composition and arcane research lane without changing Accepted current semantics |
| Superseded by | none |

## Context

The physical-world research paper correctly converges on one causal world with
specialized owners, typed coupling, representation transitions, persistence
and semantic queries. Most of that requirement is already captured more
strictly by SPEC-37. Copying its proposed `PhysicalWorldCore`, generic domain
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

Adopt SPEC-39 as a conceptual `WorldDynamics` composition only. Runtime remains
the schedule/ledger/commit owner; RPG and Mechanics retain their Accepted
stores and public paths; Physical Embodiment retains physical state. New
substrates join as peer owners through revision-bound immutable projections,
typed exchange batches and one declared all-or-nothing transaction.

Do not add a generic domain trait, global quantity map, shared mutable world
database, universal `CouplingGraph` API or `RepresentationManager` contract.
Reuse the existing fixed stages and add only consumer-specific schedule/owner
segments when evidence proves the need.

### Add one bounded Arcane owner

Create the Proposed SPEC-40 arcane lane. It owns only arcane quantity,
reservations, throughput and active execution state. RPG continues to own
skills, inventory/equipment and current character resources. Mechanics owns
ability/package definitions and proposal state. PhysX remains the only rigid
writer. Mana is not health, physical energy already deposited in another
owner, soul, identity or divine standing.

The candidate authority is CPU fixed-point publication with private `f64`
inside a fixed step. GPU is optional presentation/correspondence only. Every
physical effect has a named source, finite debit, conversion/loss receipt and
atomic destination result. No effect receives permission to set transforms,
delete material or emit damage as a substitute for the destination owner.

### Start with one telekinesis vertical

The first production-shaped consumer is one package-authored telekinesis
ability acting on one real PhysX rigid fixture through the ordinary player
action, `EffectRequestV1` and `WorldCommand` path. A sealed reservoir is debited
and one canonical force/torque batch is applied. The vertical proves resource
accounting, rigid ownership, atomic failure, exact persistence/replay and
game/headless/package parity.

A0 must freeze exact units, conservation/source model, reservoir and throughput
profile, numeric scales, schedule, target fixture, force/work law, traces,
capacities, thresholds and performance budget before code. The recommended
starting law is a closed reservoir with no regeneration during the oracle, but
that recommendation is not frozen until A0 closes.

### Defer general spell graphs and metaphysics

The first ability compiles to a closed typed execution plan. A generic bounded
spell graph waits for at least two production abilities that need shared
composition. Ambient/regional fields, ley networks, ecology, channels as
detailed anatomy, artifacts, runes and anti-magic are later packages with
their own consumers and state/evidence.

Thermal, continuum and vegetation coupling can start only after the destination
owner's relevant gate passes. Vital/tissue, soul/identity, divine remote
sources, teleportation, matter creation, transformation, resurrection and
causality/time changes require separate SPEC/ADR. SPEC-31's inert divine-
standing intent does not grant arcane or identity authority.

### Exact active state before summaries

The first arcane owner stores complete future-affecting active state and
coupling receipts in one successor composite checkpoint. Exact save/restart
precedes regional summaries, field LOD, sleep or lossy ecology persistence.
Representation changes use canonical facts and integer budgets; camera,
visibility, measured frame time, wall clock and GPU completion are forbidden.

## Failure and fallback

Invalid capability/profile, insufficient quantity, throughput violation,
stale revision, nonfinite/overflow, capacity excess, batch collision,
conversion/work mismatch, backend rejection or corrupt persistence publishes
no partial debit, physical state, receipt or event. The prior complete
generation remains authoritative.

Before activation, projects may omit arcane content or author an ordinary
mechanic. A project requiring unsupported arcane capability fails activation.
After activation there is no free cast, scripted transform/damage, VFX-only
success, frozen owner, GPU switch or retry-to-green.

## Promotion and stop conditions

The [arcane roadmap](../../plans/arcane-world/README.md) remains
`PLANNED / NOT_ACTIVE` while A0 is open. After A0 closure the serial
reservoir/transfer oracle may run. Only `ARCANE-RESERVOIR-REF-P1 = PASS` allows
the main R8 row to become an active research track.

Production promotion additionally requires `ARCANE-MECHANICS-P1`,
`ARCANE-RIGID-COUPLING-P1`, `ARCANE-PERSISTENCE-P1`,
`ARCANE-CROSS-TARGET-P1`, the declared performance budget and a
consumer-backed Accepted ADR. Failing the closed law/coupling corpus keeps the
track research-only; changing the source model, physical budget, target scope
or GPU authority requires an explicit new decision.

## Alternatives rejected

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

- SPEC-39, SPEC-40 and this ADR are Proposed; no current schema, crate, save,
  command kind, package primitive or ProductCheck is added.
- SPEC-37 remains the physical owner/coupling specialization; SPEC-39 composes
  it with non-physical substrates without superseding it.
- The next action is A0 law/profile/evidence closure, not arcane runtime code.
- Future magic effects receive completion credit only from their real
  destination-owner checks, never from presentation or an arcane-only test.
