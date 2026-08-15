# R4a derived calendar and authored routine — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / AUTHORING_COOK_LAYER` |
| Updated | 2026-08-15 |
| Task key | `r4a-world-routine` |
| Scope | One authored relay-keeper `Duty -> Rest` boundary through cook, runtime, save and replay |
| Definition of done | The production reference consumer passes the R4a ProductChecks and SPEC-20/ADR-052 can be promoted in the same changeset |
| Authority | Working context only; Accepted SPEC/ADR, checked-in contracts and `docs/roadmap.md` outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R141 consumed its authority and stopped without retry; R4a is the sole active roadmap increment.
- **Current implementation point:** Public routine/calendar, four-entry command registry, 12-stage schedule and the single determinism bundle are materialized; runtime/project no longer use the private registry or opaque profile constant.
- **Next action:** Cut authoring/project/mechanics APIs to V3/V2/V4 and cook the typed catalog plus binding into the reference package.
- **Promotion guard:** Keep SPEC-20 and ADR-052 `Proposed` until authoring, cook, activation, runtime, persistence and Replay V6 production paths pass all required checks.
- **Scope guard:** No navigation, population tiers, bulk time, transfer, cognition, 100-NPC workload, learned model or B-12 claim.

## Product outcome

The existing relay keeper in the active relay-station chunk follows one authored
routine. Exact integer-rational calendar projection crosses a single
`Duty -> Rest` boundary, publishes a typed internal command/event, persists a
separate World Services owner segment and deterministically gates the existing
`nextengine.reference-alpha.interaction.accept-frontier-relay` interaction.

The Duty path continues to produce the existing successful journal outcome.
The Rest path rejects availability without mutating RPG state. Save/restart and
current-only replay reproduce the same five-owner application closure.

## Required contract cut

1. Public canonical `CommandKindRegistryV1`, `ScheduleManifestV1` and one
   engine-owned `RuntimeDeterminismProfileV1` bundle builder.
2. Typed `WorldRoutineCatalogV1`, interaction binding, snapshot, command,
   event and activity condition contracts.
3. `ProjectAuthoringManifestV3 -> NeutralProjectSourceV3 -> CookedProjectV3 ->
   ActivatedProjectV4`, with one new catalog root (`27 -> 28`) and content entry
   (`113 -> 114`).
4. `InteractionDefinitionV2`, `RpgDefinitionRegistryV2` and
   `RuntimeBootstrapV4`; retired V1/V3 API names are removed rather than kept as
   overloads.
5. A separate `nextengine.world-services / nextengine.world-routine-snapshot /
   world-routine / v1` owner segment and a joint runtime/routine commit.
6. `ReferenceGameDriverV2`, V2 live/checkpoint/outcome types and current-only
   Replay V6 with typed streaming input and sorted owner descriptors.

## Invariants

- The authored catalog is the only keeper-ID authority; no `[0x64; 16]`
  constant remains in production or tests.
- Calendar projection is O(1), integer-only and derived from SimulationTick;
  R4a introduces no mutable second clock.
- Exactly one authored boundary can become due; no generic scheduler, second
  stage, barrier or cadence state is introduced.
- The routine command is Outcome-only at priority `250`; RPG remains priority
  `200`. The canonical registry contains exactly four entries.
- World routine proposal validation occurs before command batch, archive,
  receipt or owner-state publication.
- Runtime, routine and optional streaming state prepare and validate together,
  then commit only after the final joint preflight succeeds.
- `ProjectLockV3` and `WorldCheckpointV4` keep their wire shapes. The lock hash
  is built from the canonical registry/schedule/profile bundle; the checkpoint
  field remains the existing three-owner root.
- R4a application/save/replay closure contains runtime, RPG, physics,
  world-streaming and world-routine owner segments.

## Execution plan

1. Contract layer and exact codecs/hashes.
2. Shared registry/schedule/profile materialization.
3. Authoring/cook/activation V3/V4 chain.
4. Runtime routine owner, stage-6 proposal and joint commit.
5. Reference interaction/live/save/replay V6 consumer.
6. Focused malformed-input and deterministic repeat tests.
7. `fast`, `play`, `persistence-replay`, `content-package`, host-check and
   conditional report-only performance smoke.
8. Promote SPEC-20/ADR-052 and update roadmap only after every gate passes.

## Current evidence

- `next_contracts` passes `195/195` library tests with calendar overflow,
  catalog/snapshot round-trip, registry/schedule and bundle coverage.
- `next_runtime` passes `42/42` library tests after replacing its private
  three-entry registry with the public four-entry value.
- Workspace `cargo check --workspace --all-targets` passes.
- Focused `cargo clippy -p next_contracts -p next_runtime --all-targets --
  -D warnings` passes.

## Required context

1. [Agent routing](../../architecture/agent-routing.md), R4a row.
2. [SPEC-20](../../architecture/20-world-simulation-and-population-lifecycle.md)
   and [ADR-052](../../architecture/adr/052-derived-world-calendar-and-authored-routine-vertical.md).
3. Routed SPEC-02/03/08/09/12/13/17/18/19/21/22/24/25/29 and
   ADR-008/016/019/021/022/025/030/034/046/047/048/051.
4. [Current roadmap](../../roadmap.md).

## ProductChecks

| Check | Required evidence |
| --- | --- |
| `fast` | Calendar arithmetic, canonical codecs/hashes, malformed proposal rollback, joint commit and retired API/replay probes |
| `play` | Duty success, Rest rejection, boundary command/event, exact repeat roots and no hardcoded keeper identity |
| `persistence-replay` | Five-owner save/restart and current-only Replay V6 parity |
| `content-package` | Typed V3/V2/V4 chain, exact `28/114` counts and invalid catalog/binding/profile rejection |
| host-check | Workspace policy and architecture-sensitive integration checks |
| performance | Fixed-stage smoke recorded `REPORT_ONLY`; no B-12 closure |

## Handoff

- **Workspace:** clean at R4a activation; R141 implementation/docs are commits
  `75661a1` and `320e9f1`.
- **Training authority:** no R141 retry, R142 or downstream training work.
- **Architecture status:** SPEC-20 v3.1 and ADR-052 remain `Proposed`.
- **Current risk:** the public contract/API cut is broad; avoid compatibility
  shims that leave retired types or duplicate canonical authorities.
