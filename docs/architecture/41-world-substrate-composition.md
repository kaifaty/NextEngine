# SPEC-41: Proposed world-substrate composition

| Field | Value |
|---|---|
| ID | SPEC-41 |
| Status | Proposed |
| Version | 1.3 |
| Last verified | 2026-08-17 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [SPEC-31](31-autonomous-quest-lifecycle-and-narrative-director.md), [SPEC-39](39-layered-physical-world.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-020](adr/020-rpg-domain-authority-and-extension-boundary.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-078](adr/078-world-substrate-and-arcane-physical-interaction-track.md), [ADR-081](adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md) |
| Specializations | [SPEC-42](42-arcane-substrate-and-physical-magic.md), [SPEC-43](43-thermochemical-material-processes.md), [SPEC-44](44-neural-assisted-world-simulation.md) |
| Candidate revision note | Version 1.3 applies ADR-081 successor-stage, composition, identity, checkpoint-epoch, capacity, fault-domain, budget and shadow-only neural guardrails without changing the current runtime profile |

## Status and purpose

This SPEC connects gameplay, RPG state, physical embodiment and future world
substrates without creating a universal world solver, a shared mutable state
store or a second command runtime. `WorldDynamics` is an architectural name for
their fixed-stage composition. It is not a new state owner, public service,
crate requirement or generic plugin bus.

Current Accepted owners and paths remain unchanged. Proposed additions are the
bounded Arcane owner in SPEC-42 and Thermochemical material owner in SPEC-43.
Continuum and living structures retain their independent SPEC-38/39/40 gates.
SPEC-44 is an optional proposal producer, not a substrate owner. Vital, soul,
identity, divine, atmosphere and ontological domains have no current authority
or implementation obligation.

## World owner map

| Concern | Sole owner | Cross-owner rule |
|---|---|---|
| Tick, command identity, ledger, schedule and atomic publication | Core Runtime | No substrate creates a private clock, command queue, RNG or receipt path. |
| Character, inventory, equipment, skill/proficiency and current bounded character resources | RPG Framework | A substrate reads revision-bound views and proposes typed operations; it never copies RPG fields. |
| Ability definitions, package reducer state and semantic effect proposals | Mechanics Runtime | First-party and community magic use the same package/capability/`EffectRequestV1` path. |
| Rigid/articulated and promoted non-rigid physical state | Physical Embodiment owners from SPEC-26/39 | External substrates send canonical batches; only the physical owner writes pose, velocity, contact, mechanical sample/structure state or topology. |
| Future arcane reservoirs, channels, executions and fields | Arcane owner from SPEC-42 | RPG skill and Mechanics definitions are references, not duplicate arcane state. |
| Future material composition, enthalpy, phase and reaction progress | Thermochemical owner from SPEC-43 | Stable parcel attachments reference physical participants; consequences cross typed atomic batches. |
| Optional learned solver advice | no state owner; SPEC-44 proposal producer | Advice is revision-bound, stateless and validated by one classical owner; it publishes no world state or exchange. |
| Future tissue, growth and disease | no current owner | A Vital owner requires a concrete production consumer and separate SPEC/ADR. |
| Future soul, true-name, oath or continuity state | no current owner | Identity is not inferred from mana, RPG identity or `PersistentId`. |
| Presentation and diagnostics | Presentation | Read-only extraction; VFX, camera and renderer timing never select an outcome. |

An authored object may participate in several rows, but every mutable field
still belongs to exactly one owner. Similar words such as energy, health,
reserve, damage or identity do not authorize duplicated state.

## Composition layers

```text
L0 Exact project/profile/capability closure
 ↓
L1 Runtime identity, command ledger and fixed schedule
 ↓
L2 Mechanics/RPG validation and immutable cross-owner read set
 ↓
L3 Peer owner candidate states (physical, thermochemical, arcane, later vital/identity)
 ↓
L4 Typed exchange batches and one declared composite commit
 ↓
L5 Committed outcomes, owner segments and semantic queries
 ↓
L6 Read-only presentation and diagnostics
```

Layers are authority and transaction boundaries, not a permission for an upper
layer to mutate a lower one. Each new owner or edge declares exact inputs,
writes, order, numeric profile, capacities, failure semantics and evidence. A
global `WorldState` map, generic quantity bag or arbitrary domain callback is
forbidden.

## Command-to-outcome transaction

Every cross-owner action starts through the current production path. The first
Arcane-to-PhysX profile uses two transactions rather than keeping an external
command open across the current stage-5/stage-8 boundary:

1. player, AI, package, script or tool emits a bounded mechanics/command
   proposal;
2. Runtime authenticates capabilities, normalizes the canonical command body,
   computes ADR-022 identity and admits it through the existing ledger;
3. domain validators freeze the exact owner/profile revisions and construct an
   immutable start plan;
4. the due Ingress transaction atomically publishes only the validated owner
   start/reservation state and an ordinary terminal `CommandReceipt`; that
   receipt means `execution started`, not `physical effect succeeded`;
5. at the declared stage-8 boundary (`PhysicalStep` for the current physical-
   only profile or successor `WorldDynamicsStep`), active owners compute private
   candidate states and typed exchange batches without another package callback;
6. Runtime validates batch uniqueness, revisions, bounds, conservation/cost
   receipts and all candidate roots;
7. participating owner states plus exchange receipts publish together, or none
   publishes;
8. stage-9 Outcome may publish completion/failure facts through its one existing
   `InternalSystem` batch; it cannot re-enter gameplay in the same tick;
9. semantic queries and presentation derive only from committed results.

An external command never remains pending merely to await stage-8 physics. A
future substrate that cannot use the start-receipt plus exchange-receipt split
requires a consumer-backed schedule/ledger decision before implementation; it
cannot add a hidden barrier or same-tick re-entry.

## Successor `WorldDynamicsStep` and composition profile

The current twelve-stage runtime profile is unchanged: its stage 8 remains the
physical-state-only `PhysicalStep`. The first production profile containing a
non-physical Arcane or Thermochemical owner introduces a successor profile
whose stage 8 is named `WorldDynamicsStep`; it retains the same stage count and
relative order. The successor profile binds a closed owner set and exact read/
write access sets, so an inactive owner has no default segment or empty step.

One runtime-owned `WorldDynamicsCompositionProfile` binds the complete acyclic
owner/edge DAG, stable topological order, cadence, reducers, capacities,
candidate/root closure, checkpoint segments, fault domain and performance row.
Every canonical rigid input is merged before exactly one PhysX integration;
downstream owners then consume the frozen contact/load projection. Pairwise
couplers cannot each integrate PhysX or publish independently when more than
one edge is active.

The stage transaction is `Freeze -> Prepare -> Execute once -> Validate all ->
Publish all or fault`. Owners mutate only private candidate storage. A failed
post-execution validation publishes no candidate state and faults the declared
domain; continuation by rolling a mutated PhysX adapter back is forbidden until
an exact clone/restore proof receives a separate Accepted decision.

## Typed exchange edges

Each cross-owner edge is a separately versioned profile, not one universal
coupler interface. A canonical exchange record contains at least:

- exact `world_namespace`, source and destination owner IDs and the applicable
  destination `world_id`;
- expected source and destination revisions/roots, tick/substep, edge profile,
  stable source/destination participant IDs and operation slot;
- source/destination definition and profile revisions plus command identity
  when the record is command-derived;
- fixed-point quantities, units, reference frames and moment/COM references;
- declared source debit, destination effect and residual/dissipation receipt;
- finite capacities, exact canonical order and stable failure code.

An exact command retry is intercepted by the command ledger and creates no new
exchange record. Inside one newly constructed closed owner batch, exactly one
record is allowed for a declared key: an exact duplicate or a different hash
under that key is an internal invariant failure that rejects the complete
participating step. The runtime never chooses by arrival, worker or GPU
completion order. Direct references to another owner's ECS/backend buffers are
forbidden.

`world generation`, save/checkpoint generation and an undefined owner
generation are forbidden exchange-key fields. Epoch identity belongs to the
schedule and checkpoint closure, never to an exchange key.

## Representation, streaming and persistence

There is no current generic `RepresentationManager`. Each owner declares its
own bounded representation ladder and `Prepare -> Validate -> Commit ->
Stabilize` transfer. A later shared scheduler may coordinate already-proven
transitions, but cannot own their state or invent a global relevance score.

Authoritative representation choice may use only canonical simulation facts,
world residency, stable identities and manifest integer tokens. Camera,
visibility, measured frame time, wall clock, cache warmth and GPU completion
are forbidden. Exact active persistence precedes any lossy summary. A dirty or
future-affecting owner cannot be silently reconstructed from immutable content.

Every promoted owner adds one hash-bound owner segment to a successor composite
world checkpoint. Cross-owner receipts needed for continuation are in the same
atomic save closure; sidecars and independently published substrate saves are
forbidden. Load validates all required segments before replacing the target
world. Pre-v1 formats remain current-only under ADR-046.

The first PhysX-coupled production profile additionally binds a fixed positive
checkpoint-epoch length. At every scheduled epoch boundary the live run
reconstructs a fresh PhysX scene from canonical owner state, validates its
continuation witness and swaps it atomically. Save requests wait for that same
barrier; the uninterrupted comparison run executes identical barriers. An
unbounded world cannot rely on ADR-059's bounded training-prefix replay.

## Mechanics, AI and semantic queries

Magic and other systemic mechanics remain `AbilityDefinition`/affordance/
`EffectRequest` consumers of SPEC-13. A data-driven effect may select a typed
owner operation, but package code cannot construct a trusted exchange batch,
debit a reservoir, write a physical transform or declare success.

AI receives capability-filtered immutable semantic facts such as available
reserve, overload risk or committed physical consequence. It does not inspect
raw field, particle, solver or hidden target state. Planning is a proposal;
owner validation remains the oracle. Events describe committed facts and never
replace owner state or permit same-tick mutation.

Optional learned assistance follows SPEC-44. N0 diagnostics and N1 proposals
run only after the classical owner step as report/shadow work. They cannot
alter production initialization, work, failure classification, exchange or
root. Runtime solver advice requires a later consumer-backed Accepted ADR and
a mechanically checkable admissibility certificate under ADR-081.

## Failure and fallback

Capability denial, insufficient source quantity/throughput, ineligible target,
worst-case capacity denial or explicit cancel before owner freeze is an
ordinary gameplay rejection or deterministic execution termination. It affects
that action only, publishes no destination effect and returns any unused
reservation under the profile. Content, topology and batch capacities are
pre-admitted from declared worst-case bounds.

Nonfinite value, fixed-point overflow, post-freeze missing/stale participant,
duplicate exchange key, different bytes under one key, conservation/cost
failure, exhaustion beyond a reserved bound, backend rejection or corrupt
checkpoint is an internal invariant failure. It rejects the complete
participating step and transitions its bound domain through `Running ->
Faulted -> DiagnosticSaved -> Closed | ExplicitRestore`. Invalid project
content/profile or missing required capability fails before world activation.
The first primary gameplay profile maps a fatal world-dynamics fault to the
whole application session; only independently provisioned test/training scene
slots may declare narrower isolation.

Before a Proposed substrate capability is activated, a project may omit it or
use a separately authored ordinary mechanic. After activation there is no
silent VFX-only success, free resource use, scripted transform, owner freeze,
backend switch or retry-to-green. Required-capability absence rejects project
activation; an active fatal failure uses the exact fault domain above and the
last complete checkpoint remains the only restorable authority.

## Promotion boundary

This composition model has no standalone current ProductCheck. A concrete
substrate first passes its own reference, production-path, coupling,
persistence, cross-target and conditional performance checks. A future
`WORLD-DYNAMICS-P1` is created only when two independently promoted **new
substrate owners beyond the existing Physical Embodiment owner** participate
in one production transaction. The first Arcane-to-PhysX edge is covered by
its own coupling check. An Arcane-to-Thermochemical edge is the first currently
identified candidate for the composition check, after both base owners pass.
The check must prove exclusive writers, all-or-none publication, exact
receipt/event order and presentation independence. A neural model cannot
satisfy the owner count.

Before the first integrated consumer, a successor `GameplayBudgetMatrix` adds
one mutually exclusive `world-dynamics-step` row. Its measurement window covers
all physical substeps, canonical merges, owner solves, validation and root
publication in one gameplay tick. Standalone solver and PHYS-P4 numbers remain
diagnostic evidence and cannot be added together as an integrated PASS.

No public `WorldDynamics`, generic domain, raw query, spell graph or exchange
API is added by this Proposed SPEC. A production consumer introduces only the
smallest current-only contracts it exercises, through a new Accepted ADR.
