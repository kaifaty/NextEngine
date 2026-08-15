# R4a derived calendar and authored routine — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-15 |
| Task key | `r4a-world-routine` |
| Scope | One authored relay-keeper `Duty -> Rest` boundary through cook, runtime, save and replay |
| Definition of done | The production reference consumer passes the R4a ProductChecks and SPEC-20/ADR-052 can be promoted in the same changeset |
| Authority | Working context only; Accepted SPEC/ADR, checked-in contracts and `docs/roadmap.md` outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R4a passed its production and documentation gates; SPEC-20/ADR-052 are `Accepted` and the increment is complete.
- **Current implementation point:** The full typed V3/V2/V4 authoring path, runtime routine owner, joint commit, five-owner persistence and current-only Replay V6 are production consumers.
- **Next action:** R4b tiers + graph navigation + 100 NPC is the next eligible roadmap increment and is outside this completed changeset.
- **Promotion result:** Format/check/clippy, focused rollback/retired-format coverage, `play`, `persistence-replay`, `content-package`, `boundary-scan` and `host-check` pass. Performance metrics remain report-only and make no B-12 claim.
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

1. [complete] Contract layer and exact codecs/hashes.
2. [complete] Shared registry/schedule/profile materialization.
3. [complete] Authoring/cook/activation V3/V4 chain.
4. [complete] Runtime routine owner, stage-6 proposal and joint commit.
5. [complete] Reference interaction/live/save/replay V6 consumer.
6. [complete] Focused malformed-input and deterministic repeat tests.
7. [complete] `fast`, `play`, `persistence-replay`, `content-package`,
   host-check and conditional report-only performance smoke.
8. [complete] Promote SPEC-20/ADR-052 and update roadmap after every gate.

## Current evidence

| Check | Result |
| --- | --- |
| `fast` composition | `cargo fmt --all -- --check`, workspace all-target check/clippy, focused positive/failure tests and `boundary-scan` pass |
| `host-check` | `PASS` on `x86_64-unknown-linux-gnu`, Rust `1.97.1` |
| `play` | `PASS`: 32 ticks, 28 events, 13 RPG events, revision 41; state root `64b2318210a938d6e23327af56fcfc372b1e3ba76e8c61f0ef261b0f0f0a75e1` |
| `persistence-replay` | `PASS`: 19 ticks, 2 generations, 8 RPG events, current chunk `relay-station`; final state root `2f9ab39b00d4891bf699d7f0a0d6dc576f925bc486297eba5e239b0da7142043` |
| `content-package` | `PASS`: 28 roots, 114 records, 64 chunks, 2 mechanics, 1 Luau and 1 Wasm package; composition lock `22811df68d2de1b193d54a98f5b3adb74d9b72e7de1995144525573097e12c47` |
| performance smoke | 900 live ticks / 901 commands, stable live root `fce912bf569c5c0228a7ce345f5c827aa3b5936e078b008da7274632313ebe04`; all six metrics `REPORT_ONLY`; outer result `NOT_RUN / PERF_TARGET_FINGERPRINT_UNSUPPORTED_HOST` |

Negative coverage includes stage-6/stage-9 fatal rollback, stale joint commit,
load-time routine/ledger closure rejection and typed retired Replay V5
rejection. The ordinary live path validates the same fixed-order joint owner
commit without rebuilding application evidence every tick; scenario,
checkpoint and replay paths retain full evidence-bearing validation.

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

- **Workspace:** R4a implementation, checks and documentation are complete on
  the roadmap branch; earlier R141 implementation/docs remain commits
  `75661a1` and `320e9f1`.
- **Training authority:** no R141 retry, R142 or downstream training work.
- **Architecture status:** SPEC-20 v3.1 and ADR-052 are `Accepted`.
- **Next scope:** R4b may introduce tiers/navigation/100-NPC contracts only
  with their production consumer and mapped ProductChecks; no generic
  scheduler or compatibility shim was admitted by R4a.
