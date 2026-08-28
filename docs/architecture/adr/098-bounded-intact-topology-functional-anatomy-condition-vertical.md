# ADR-098: Bounded intact-topology functional-anatomy condition vertical

| Field | Value |
|---|---|
| ID | ADR-098 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-28 |
| Last verified | 2026-08-28 |
| Normative dependencies | [PRODUCT-FA-001](../../product/functional-anatomy-and-character-embodiment.md), [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-19](../19-rpg-domain-and-narrative-state.md), [SPEC-22](../22-schema-registry-compatibility-and-migration.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-36](../36-functional-tissue-condition-and-injury.md), [ADR-020](020-rpg-domain-authority-and-extension-boundary.md), [ADR-027](027-physics-motor-and-animation-layering.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-066](066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md), [ADR-075](075-product-grounded-functional-anatomy-and-character-embodiment.md) |
| Supersedes | Narrowly supersedes ADR-075 and SPEC-36 clauses that kept every exact injury contract Proposed: the BodySchema-bound unilateral profile, intact-topology RPG condition/treatment operations, derived capability envelope and fixed-PD clamp defined here are current. Fracture/topology, ordinary limp/fall/crawl behavior, UI, surface severity, LOD scale and learned routes remain Proposed. |
| Superseded by | Not superseded |

## Context

R7 shipped the procedural physical-character baseline and immutable Linux v1
release. R8 now has a concrete first consumer: the reference project needs one
BodySchema-bound unilateral condition path which can distinguish intact,
partial knee-extension capacity and declared tendon/nerve zero transmission
for both the player and an NPC, then recover through the approved staged
treatment sequence.

Promoting the entire SPEC-36 matrix would invent fracture, topology, UI and LOD
contracts before those consumers exist. Keeping all exact records Proposed
would leave the implemented condition path without a governed owner, wire
identity or failure boundary.

## Decision

### Immutable anatomy profile

`BodySchemaAssetV1` MAY carry one `FunctionalAnatomyProfileV1`. The current
reference profile is bound to the exact existing `BodySchemaV1` hash and names
one left-lower-limb region, one knee-extensor functional group, the existing
left-knee actuator and its positive direction. It defines exact partial and
post-repair Q16 capacities. It does not alter `BodySchemaV1`, physical topology,
action layout or learned-policy identity.

Project Authoring V7 exposes only an optional exact profile ID. The reference
project declares the current profile; creator projects which do not consume
functional anatomy omit it and retain an honest unsupported route. Unknown or
mismatched profile IDs fail before cook publication.

### RPG condition and Mechanics proposal

RPG owns a separate `BodyCondition` aggregate bound to one Character,
BodySchema hash, anatomy-profile hash and region. The current closed intact-
topology impairment set is:

- `Intact`;
- `PartialKneeExtensor`;
- `TendonTransmissionLost`;
- `NerveControlLost`.

The current systemic band is `Stable | Impaired`; the current recovery stages
are `Untreated → Stabilized → Repaired → Rehabilitated`. Medical and magical
treatment use the same owner transition. Repair converts a zero-transmission
state to the declared partial post-repair capacity; rehabilitation returns it
to intact/stable.

Mechanics consumes the immutable profile plus exact RPG snapshot and compiles
only `ApplyBodyImpairment` or `AdvanceBodyTreatment` operations. The ordinary
RPG plan builder rechecks revision, expected state and profile hash and emits
ordered body-condition events. First-party player and NPC paths use the same
compiler and transaction; no privileged mutation is admitted.

### Derived motor capability

Physical Embodiment reconstructs `BodyCapabilityEnvelopeV1` from the compiled
BodySchema, exact profile and committed body-condition aggregate. The envelope
binds subject, BodySchema/profile/condition hashes, condition revision and a
sorted actuator-direction capacity set. It is a reconstructible view and MUST
NOT become another durable condition owner.

The current fixed-PD controller validates subject/schema/channel identity and
applies the directional capability clamp after the ordinary effort bound and
before the effort-rate bound. Its previous effort is first clamped into the new
capability interval, so declared zero transmission becomes zero immediately
rather than leaking prior positive effort through the rate limiter. The
opposite actuator direction remains available. Missing, malformed or foreign
envelopes reject the whole step; the caller retains the last valid safe route.

### Current-only schema boundary

This increment advances the current RPG aggregate snapshot version to `4` and
RPG command schema version to `4`. `BodySchemaAssetV1` canonical bytes also add
the optional profile field and the reference asset revision advances. These
alpha project/RPG/save/replay inputs remain exact-current under ADR-046 and
SPEC-22: older bytes are not inferred, defaulted or migrated. The immutable
published `v1.0.0` distribution remains exact historical release evidence; no
persisted RPG format was declared a publicly supported predecessor.

## Product impact

The engine now has a real, deterministic first R8 condition path: identical
player/NPC state can lose half or all positive left-knee effort, survive exact
RPG snapshot round trips and recover through medical or magical repair without
changing controller authority. This is sufficient to promote the condition/
capability subset, but not to claim the full injury experience.

Stable fracture, retained passive topology, detachment, ordinary locomotion
adaptation, body UI, visible injury severity, 16/64/distant LOD and learned
injury conditioning remain outside this decision.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| focused contracts/RPG/Mechanics/Motor | Canonical profile/aggregate/command/event/envelope, stale/invalid transition, player/NPC compiler and directional fixed-PD vectors | Exact round trips; staged transition; intact `150000000`, partial `75001144`, zero `0`, rehabilitation restores `150000000` µN·m in the current reference vector | Reject complete input/transaction/step and retain prior valid state |
| `physical-character` / `INJURY-CONDITION-P1` | Cook/activate reference project twice; run Mechanics → RPG → capability → fixed-PD for player and NPC | Two subjects, ten committed transitions, identical effort matrix and digest `2ce8bed4…d65b5`; repeated whole check is exact | Check fails; no full injury or topology claim |
| `play` | Existing offline reference loop with current initial body-condition owners | Existing gameplay remains complete; body-condition additions create no privileged path | Current procedural controller remains the supported fallback |
| `persistence-replay` | Current snapshot/save/Replay closure includes the RPG owner bytes | Exact current roots and early incompatible-version rejection | Reject before world mutation |
| `content-package` | Reference profile cooks/activates; creator-smoke omits it | Exact reference dependency/hash closure and honest creator unsupported route | Reject unknown/mismatched profile before publication |

`platform` is not triggered because no platform/backend boundary changed.
Full workload performance is not credited: the existing PD path is unchanged
when no envelope is supplied, and SPEC-36 scale budgets remain Proposed.

## Considered alternatives

- **Promote fracture and retained topology now.** Rejected because no authored
  break-site/topology consumer, atomic Physics transaction or fall/crawl
  behavior exists yet.
- **Store capability beside the RPG condition.** Rejected because it creates a
  parallel mutable authority and can drift from BodySchema/profile revisions.
- **Apply capability after rate limiting.** Rejected because a zero-capacity
  transition would leak prior effort for multiple substeps.
- **Put condition fields inside Character.** Rejected because the separate
  aggregate keeps revision scope, future region multiplicity and treatment
  writes independent from inventory/resources.
- **Require the profile in every creator project.** Rejected because projects
  without an injury consumer must not pretend to support it.

## Consequences

- `BodyCondition`, two RPG operations and two events are current contracts.
- Mechanics and motor expose no backend, ECS or mutable RPG handle.
- BodySchema generation/action layout and the current fixed-PD baseline remain
  stable; only the optional capability-consuming path adds a clamp flag.
- Current project/content/RPG roots change and are validated as one coherent
  current-only generation.
- Full `INJURY-EMBODIMENT-P1` remains `NOT_RUN`; only the narrower
  `INJURY-CONDITION-P1` subset can pass.

## Supersession

Adding fracture/topology, new anatomy regions, additional systemic bands,
ordinary injury-adapted locomotion, UI/surface state, workload LOD or changing
the controller/owner boundary requires a later consumer-backed Accepted ADR.
