# ADR-052: Derived world calendar and authored routine vertical

| Field | Value |
|---|---|
| ID | ADR-052 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-09 |
| Last verified | 2026-08-15 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-08](../08-audio-navigation-and-world-services.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](../18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-19](../19-rpg-domain-and-narrative-state.md), [SPEC-20](../20-world-simulation-and-population-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](../22-schema-registry-compatibility-and-migration.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](../25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-29](../29-platform-host-and-application-session.md), [ADR-008](008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-019](019-canonical-player-actions-and-presentation-authority.md), [ADR-021](021-deterministic-population-residency-and-time-advance.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-025](025-schema-content-and-migration-authority.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-034](034-player-targeting-replay-v5-and-mapping-provenance.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-047](047-simple-application-session-and-save-on-close.md), [ADR-048](048-direct-exact-project-lock.md), [ADR-051](051-r3a-packaged-chunk-streaming-commit-boundary.md) |
| Supersedes | Partially ADR-021's retired mutable calendar/tier/bulk-first representation, ADR-034/046/047's Replay V5 designation, ADR-048's authoring-v2/ActivatedProjectV3/Replay-V5 forms and ADR-051's ActivatedProjectV3/replay format freeze; exact scope is listed in `Supersession` |
| Superseded by | none |

## Context

R3 is complete and the next product risk is not a generic scheduler or learned
NPC policy. It is the absence of one player-visible production consumer for
calendar and population state. SPEC-20 version 2.0 intentionally removed the
old speculative schemas after ADR-046, while ADR-021 retained only durable
identity, single ownership, no-wall-clock authority and no fabricated outcome
as current invariants.

The smallest useful consumer is the existing relay keeper: one authored
`Duty/Rest` routine gates the exact
`nextengine.reference-alpha.interaction.accept-frontier-relay` path in the
current active chunk and survives save/restart/replay. Navigation, residency
tiers, bulk time and learned behavior are not required to prove this boundary.

That consumer and its ProductChecks now exist in the production path; the
decision is Accepted under ADR-046 without admitting the excluded R4b/R4c/R4d
scope.

## Decision

Next Engine implements the contract in SPEC-20 with these rules:

1. Runtime remains the sole owner of `SimulationTick`. World Services owns an
   immutable, project-bound integer-rational mapping from `SimulationTick` to
   nominal `WorldTick`; there is no independently incremented mutable world
   clock in R4a.
2. World Services owns durable routine membership, committed activity and
   revision. The catalog's `PersistentId` becomes the production
   `quest_giver_character_id`; world bootstrap consumes it and removes the
   reference-session ID constant. RPG, UI, Agent and streaming receive
   immutable projections and retain their own state.
3. Typed authoring supplies one domain-relevant catalog asset containing the
   calendar profile and one relay-keeper `Duty -> Rest` boundary. Generic
   property bags, independent no-consumer profile/routine assets and
   reference-game hardcoding are not production content paths.
4. At existing stage 6, World Services computes the end-of-tick boundary from
   checked integer arithmetic and stages at most one internal command for the
   existing stage-9 Outcome batch. A narrow prepared/validated Runtime +
   routine + optional-streaming transaction extends the current R3 paired
   commit, so all live owners publish together or none do. No new stage,
   scheduler, command barrier or generic transaction framework is added.
   Its evidence-bearing validation path precomputes the sorted owner
   descriptors and application root; commit returns
   `WorldServicesTickCommitV1` around the unchanged Runtime `TickReport`,
   streaming/routine projections, optional streaming receipt, exact
   descriptors and root. The ordinary live driver uses the same joint
   generation validation and final preflight without materializing discarded
   evidence on every tick; checkpoint/save/replay state materializes that
   evidence before publication. These are `next_runtime` workspace results
   rather than durable schemas; a later Save retains its ordinary independent
   freeze/snapshot boundary.
5. Ingress in tick `T` observes the state committed at the end of `T - 1`.
   The tick-`T` routine Outcome enters the tick-`T` root/presentation and gates
   interaction beginning at `T + 1`. Fresh activation requires
   `next_tick == anchor_simulation_tick`; load validates activity against the
   last completed `next_tick - 1`, with a separate no-completed-tick anchor
   case.
6. `InteractionDefinitionV2.availability_condition_or_none` binds the condition
   to `accept-frontier-relay` only. It reads the committed World Services
   projection: `Duty` permits the existing dialogue/quest/relationship
   transaction and journal update; `Rest` returns
   `InteractionAvailabilityV1::WorldRoutineActivityUnavailable`, then an
   accepted input receipt carries zero derived commands and no RPG plan is
   built. The separate completion interaction carries no condition and is
   unchanged.
7. Routine state uses a separate `world-routine` segment under owner
   `nextengine.world-services`, alongside the existing streaming segment.
   Segment identity is the complete `(owner, schema, segment)` tuple.
8. `SaveManifestV2`, `WorldCheckpointV4`, `ProjectLockV3`,
   `SchemaRegistryManifestV2` and `ContentManifestV1` keep their generic wire
   semantics. Authoring/neutral/cooked become V3,
   `InteractionDefinition`/`RpgDefinitionRegistry` become V2, activation
   becomes `ActivatedProjectV4`, and replay becomes current-only V6.
   `ReplayTickManifestV6` retains the ordered V5 tick facts under the V6 type,
   adds typed `WorldStreamingReplayInputV1` begin/complete assignment, and
   carries the routine command/event through the generic Outcome vectors; V6
   never nests a retired V5 tick or relies on runner-hardcoded transition
   ticks. It also records the typed `InteractionAvailabilityV1` values/hash
    rather than relying on a diagnostic string. Replay V6 carries a complete
    five-segment R4a closure and sorted full `SaveSegmentDescriptor` values.
    Load exact-matches routine activity/revision against the dedicated stream's
    genesis or single committed sequence-`0` receipt/body/event/delta closure.
    `WorldCheckpointV4.state_root` remains the three-segment core root; the
   application uses a separate five-segment root.
9. The admitted profile is one subject, one authored transition and at most one
   due transition per simulation tick. Recurring schedules, larger population,
   tiers, navigation, bulk time and learned policy remain future increments.
10. The internal command is outcome-only schema
    `nextengine.command.world-routine` V1 from system
    `nextengine.system.world-routine-boundary`, capability
    `nextengine.capability.world-routine-commit`, priority 250, stream slot/epoch
    `0/0` and sequence equal to expected record revision. Its canonical
    `CommandKindRegistryV1` entry uses command-kind ID
    `nextengine.command-kind.world-routine`, unscoped capability and
    `SHA256("nextengine.command-validator.world-routine.v1\0")` validator
    profile. SPEC-20 also closes the exact V1 entries/profile hashes for the
    existing noop, RPG and physical commands, so the promoted registry has one
    unambiguous four-entry golden vector. The V1 registry wire schema is
    retained; its canonical map bytes/hash replace the private descriptor hash.
    The routine command's single owner-write delta has the exact SPEC-20
    framing used by `transaction_result_root`. Exact stage-6/stage-9 internal
    proposal checks run before Outcome admission; an impossible or non-exact
    dedicated proposal returns the single fatal
    `WORLD_ROUTINE_INTERNAL_INVARIANT` and creates no batch member, archive
    entry or receipt. Existing generic command/ledger rejection,
    no-reservation and terminal-receipt semantics remain unchanged for their
    respective submitted-command branches.
11. R4a materializes the SPEC-21 canonical `CommandKindRegistryV1` and
    `ScheduleManifestV1` rather than private version counters or opaque fixed
    hashes. SPEC-20 closes `RuntimeStageId` as the twelve existing numbered
    SPEC-02 steps. The schedule has exactly the two existing Ingress-2 and
    Outcome-9 ordinal-zero admission barriers, no reducers, and one registered
    World Services producer at existing stage-index 6
    `WorldStreamingCommit`, with empty ordering edges, exact SPEC-20 access
    keys, shard plan `nextengine.shard-plan.world-routine-single`,
    `PersistentId` order and no reducer. A single engine-owned builder derives
    both hashes and the exact
    `RuntimeDeterminismProfileV1`; cooker, activation, Runtime bootstrap, save
    and replay consume that same result. `ProjectLockV3` keeps its wire shape,
    but a duplicate hand-written profile-hash constant is forbidden and stale
    old-profile locks fail before activation.

## Algorithm and resource choice

For R4a, direct computation is the constraint-optimal implementation:

- calendar projection is `O(1)` with checked `u128` intermediates;
- boundary selection and due discovery are `O(1)` for the single routine;
- authoritative memory is `O(1)`.

A persisted priority queue would duplicate derivable state and add recovery
invariants without reducing the limiting cost. A reconstructible min-heap
(`O(log n)` update) or timing wheel may be considered in R4b only after the
100-NPC workload measures direct scanning as a bottleneck. It cannot become a
public schema or second authority merely for asymptotic preference.

## Product impact

The reference project has its first observable living-world behavior: the
relay keeper changes between work and rest on
deterministic in-world time, and the exact quest-acceptance interaction follows
that fact after save/restart/replay. The implementation exercises
production content, command, event, interaction and persistence boundaries
without waiting for navigation, models or a 100-NPC benchmark.

R4a does not close R4 or its blockers. The keeper does not move, no off-screen
outcome is synthesized, `r4-100npc` remains unavailable, and gameplay remains
fully offline without `ai-host`.

## Relevant ProductChecks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `play` / `WORLD-ROUTINE-P1` | Fork two runs from the same `Duty`/quest-available pre-boundary state: accept in branch A; cross to `Rest` without accepting, query and try the same interaction in branch B | Branch A performs the existing dialogue `offer -> accepted`, quest `available -> active`, relationship `0 -> 7` transaction and shows `Frontier Relay - Active`; branch B query returns `WORLD_ROUTINE_ACTIVITY_UNAVAILABLE`, the accepted input receipt has zero derived commands, the available state remains and the journal shows `Frontier Relay - Available` | Stationary authored routine remains complete; no navigation/model path is attempted |
| `persistence-replay` / `WORLD-ROUTINE-REPLAY-P1` | Save/restart and replay with `next_tick` immediately before/after a boundary plus typed begin/complete streaming assignments and routine snapshot/stream mismatch faults | Commands, events, owner-delta/receipt-chain roots, full segment descriptors, typed interaction availability/hash and five-segment application root are exact without runner-hardcoded transition ticks | Reject corrupt/incompatible input and retain the prior valid generation |
| `content-package` / `WORLD-ROUTINE-CONTENT-P1` | Cook/activate the V3/V2/V4 chain and malformed/duplicate/binding/old-profile variants | Exact typed catalog/condition, `28` reference root IDs, `114` content entries and canonical registry/schedule/profile lock activate; every invalid closure or bootstrap ID collision fails before publication/tick | Preserve the prior project/world and source bytes |
| `fast` | Calendar arithmetic, codecs, stage-6/stage-9 internal-invariant faults and owner-delta receipt roots, stream/snapshot closure, joint commit, four-entry canonical registry/schedule/profile/project-lock roots, tuple ordering and retired replay tests | Stable exact bytes/diagnostics; malformed dedicated proposals return `WORLD_ROUTINE_INTERNAL_INVARIANT` before batch/archive/receipt publication, with no hardcoded keeper ID, opaque profile constant, owner-only lookup or mutation backdoor | Reject the candidate before mutation |
| conditional `performance --scenario smoke --mode report` | Ordinary fixed-stage path with the O(1) stage-6 producer enabled | Exact roots remain stable and the routine span/logical charge is attributed; the result stays `REPORT_ONLY` | Do not run `r4-100npc` or claim B-12 closure |

## Alternatives considered

- Separate mutable `WorldTick` advanced by a command every gameplay tick —
  rejected for R4a because it duplicates Runtime cadence state and creates
  permanent ledger/body/archive churn without a player need.
- `WorldTick == SimulationTick` — rejected because it permanently couples
  calendar rate to the selected 20/30/60 Hz gameplay profile.
- Full ADR-021 calendar/tier/bulk schema — rejected because ADR-046 removed the
  unconsumed obligations and the relay-keeper consumer needs none of them.
- Calendar or routine fields in RPG — rejected because quest/dialogue state
  and world-time authority would acquire two owners.
- Merge routine state into the streaming snapshot — rejected because content
  residency and routine activity have independent schema/version/failure
  lifecycles; the generic manifest already supports multiple tuples per owner.
- Generic neutral properties or reference-game constants — rejected because
  malformed schedules would bypass a typed public cooker/validation path.
- Navigation or physical movement as the first signal — rejected because it
  expands the first consumer into placement, path and traversal contracts.
- Generic scheduler, persisted due heap, JPS or flow fields now — rejected
  because no measured R4a constraint requires them.

## Consequences

- `crates/contracts` gains the routine catalog/snapshot/command/event schemas,
  `InteractionDefinitionV2`, `RpgDefinitionRegistryV2` and the Replay V6
  descriptor vector described by SPEC-20.
- Runtime/application API closure advances to `RuntimeBootstrapV4`,
  `ReferenceGameDriverV2`, `ReferenceLiveStateV2`,
  `ReferenceStageCheckpointV2` and `ReferenceRunOutcomeV2`; their retired
  pre-v1 predecessors are not readers or fallbacks.
- Project authoring/activation, application composition, save/load/close and
  replay carry the separate routine segment through production paths.
- Replay V6 replaces V5 as the only current pre-v1 replay; no migration reader
  is retained.
- The runtime uses its existing stage graph and one Outcome batch; it adds the
  engine-declared World Services producer/command kind and a narrow joint
  prepared/validated commit with optional streaming. The unchanged Runtime
  `TickReport` is wrapped by `WorldServicesTickCommitV1` on the
  evidence-bearing path, whose exact segment descriptors and application root
  are prepared before publication; the ordinary live path skips unused
  evidence while retaining the same joint preflight/commit. The wrapper does
  not claim to carry canonical owner bytes.
- The existing SPEC-21 V1 registry/schedule schemas are finally materialized by
  one canonical determinism-bundle builder. Their changed hashes flow through
  the unchanged `RuntimeDeterminismProfileV1` and `ProjectLockV3` wire shapes;
  cooker/activation no longer compare an independent opaque profile constant.
- Invalid catalog/content/bootstrap/state fails before partial activation or
  mutation.
- Fixed-stage performance smoke is recorded as `REPORT_ONLY`; R4a does not
  activate `r4-100npc` or close B-12.
- Roadmap work may proceed to tiers/navigation/100 NPC because R4a passes.

## Supersession

This Accepted decision:

- partially supersedes ADR-021's retired mutable `WorldCalendarStateV1` and
  full tier/bulk-first design for the current R4a representation, while
  preserving durable identity, World Services ownership, no wall-clock
  authority and no fabricated outcomes;
- partially supersedes ADR-046's clauses naming population/calendar as wholly
  Proposed and `ReplayManifestV5` as the current replay, replacing them only
  with the implemented narrow R4a contract and current-only Replay V6;
- partially supersedes ADR-034's designation of Replay V5 and its hard-coded
  compare shape while retaining V2 mapping receipts and every targeting/query
  provenance fact in Replay V6;
- partially supersedes ADR-047 item 8 only for the Replay V5 wire shape while
  retaining `SaveManifestV2`, `WorldCheckpointV4`, the two-slot store and all
  simple-session/save-on-close semantics;
- partially supersedes ADR-048 items 1, 5 and 8 only where they name authoring v2,
  `ActivatedProjectV3` and Replay V5, while retaining direct exact
  `ProjectLockV3`, atomic activation and current-only rejection;
- partially supersedes ADR-051 item 2 and its scope clause only where they
  freeze `ActivatedProjectV3` and replay format, while retaining pinned
  generation, packaged chunk fetch and the paired existing-stage streaming
  boundary;
- leaves ADR-046 consumer-admission and current-only migration policy intact.
