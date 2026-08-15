# ADR-072: Deterministic population tiers and graph-navigation vertical

| Field | Value |
|---|---|
| ID | ADR-072 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-16 |
| Last verified | 2026-08-16 |
| Normative dependencies | [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-08](../08-audio-navigation-and-world-services.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-20](../20-world-simulation-and-population-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](../22-schema-registry-compatibility-and-migration.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](../25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-021](021-deterministic-population-residency-and-time-advance.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-051](051-r3a-packaged-chunk-streaming-commit-boundary.md), [ADR-052](052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-063](063-run-level-performance-evidence-and-fixed-gate-batches.md) |
| Supersedes | The clauses that describe R4b population tiers, engine-owned graph navigation and the representative 100-NPC workload as wholly Proposed; current-only Replay V6 and V3/V4 project API designations are replaced by the bounded successors below |
| Superseded by | none |

## Context

R4a proved one authored calendar/routine transition but intentionally left
population membership, logical placement, tier cadence and navigation without
a production consumer. ADR-016 already fixed the representative population at
16 active, 32 near and 52 background records. ADR-021 retained the required
ownership invariants: one durable `PersistentId` survives tier changes,
logical placement belongs to World Services, physical pose belongs to Physics,
and an abstract system may not fabricate an active traversal result.

The smallest production-backed R4b slice is therefore not a generic scheduler,
navmesh stack or cognition system. It is one typed 100-record catalog, one
engine-owned graph over the existing 64 chunks, one durable population owner
and one courier whose exact logical transfer is visible in save and replay.
That consumer and its failure checks now exist.

## Decision

1. `WorldPopulationCatalogV1` contains exactly 100 records sorted by
   `PersistentId`. The admitted cadence mix is exactly `16/32/52`; periods are
   `3/15/60` simulation ticks and phase is
   `little_endian_u64(SHA256("nextengine.npc-cadence-phase.v1\0" ||
   PersistentId)[0..8]) % period`. Camera state, renderer visibility, wall
   time, CPU load and completion order never select a tier or due record.
2. The durable tiers are the closed values `Dormant`, `Abstract`, `Simulated`
   and `Active`. They are World Services facts and are separate from physical
   LOD. The snapshot stores one stable subject ID, record revision, home
   region/node, current region/node and tier for every record.
3. `WorldNavigationCatalogV1` is the current engine-owned baseline. The
   reference catalog contains one revision-bound node per existing world chunk
   and one tile per region. Nodes, tiles and undirected positive-integer-cost
   edges are canonical sorted content. A query binds catalog asset, graph
   revision, start, goal and the closed `AbstractTransfer` capability.
4. Routing uses deterministic Dijkstra search. Equal-cost candidates are
   resolved by the canonical node/path ordering represented by the
   engine-owned IDs. `NavigationRoutePlanV1` binds the ordered node path,
   integer total cost and a canonical plan hash. No Recast/Detour type, raw
   polygon reference, runtime entity or task handle enters content, snapshots,
   commands, saves or replay.
5. The reference courier starts `Dormant` in relay-station and performs exactly
   seven committed revisions: `Dormant -> Abstract`, one route-plan-bound
   abstract transfer to frontier, `Abstract -> Simulated -> Active ->
   Simulated -> Abstract -> Dormant`. The same `PersistentId` survives every
   step. All other population records retain revision zero in this bounded
   product slice.
6. `CommitAbstractTransfer` succeeds only for an `Abstract` record against the
   exact catalog revisions, source/target facts and route-plan hash. An attempt
   to transfer an `Active` record returns `PHYSICAL_TRAVERSAL_REQUIRED` and
   changes neither logical placement nor physical pose. Stale route, catalog,
   record or owner generation rejects before publication.
7. `WorldPopulationOwnerV1` is a separate World Services owner. At existing
   stage 6 it scans the fixed sorted catalog, executes one graph query for each
   due record and produces an exact service report. The courier stages at most
   one internal Outcome command for existing stage 9. There is no new Runtime
   stage, generic job graph, persisted queue or second placement authority.
8. The internal command is `nextengine.command.world-population` V1 from
   `nextengine.system.world-population-boundary`, capability
   `nextengine.capability.world-population-commit`, priority 260 and sequence
   equal to the expected record revision. `core_r4b` contains five command
   kinds and two ordered World Services systems while retaining the existing
   twelve stages. One determinism-bundle builder supplies registry, schedule
   and runtime-profile hashes to cooker, activation, Runtime, save and replay.
9. Runtime, routine, population and optional streaming use one prepared,
   validated and final-preflight transaction. The first live write happens
   only after every owner base, canonical descriptor and application root is
   valid; commit then performs no fallible work. A stale candidate publishes
   none of the owners.
10. The population snapshot is a separate segment with owner
    `nextengine.world-services`, schema
    `nextengine.world-population-snapshot`, segment `world-population` and
    version 1. The reference application root has six sorted full-tuple
    segments: Runtime, RPG, Physics, streaming, routine and population.
    `WorldCheckpointV4` and `SaveManifestV2` retain their generic wire shapes.
11. Replay advances to current-only `ReplayManifestV7`. V7 requires the
    initial population segment, carries population commands/events in the
    existing Outcome vectors and compares six-owner roots/descriptors.
    Restore independently proves that each nonzero population record revision
    closes over the dedicated stream's committed command, event, owner delta
    and receipt history. Missing or forged evidence fails with
    `WORLD_POPULATION_LEDGER_CLOSURE_INVALID` before any owner publication.
    Retired pre-v1 replay versions are typed unsupported formats, not readers.
12. Project authoring/source/cook advance to V4 and activation to V5. The
    population and navigation catalogs are two ordinary domain-relevant root
    assets, changing reference-alpha root count `28 -> 30` and content entries
    `114 -> 116`; the partition remains four regions and 64 chunk bindings.
13. `r4-100npc.v1` is an implemented report-only production workload with
    1,000 warm-up and 10,000 measured ticks. It verifies exact due counts,
    one query per due record, queue depth, zero deferral/drop/starvation,
    population/application/ledger roots and snapshot/body bounds. It is not a
    hard gate and cannot close B-12 without ADR-063-compatible THOTH baseline
    and fixed-batch evidence.

## Product impact

Reference alpha now has durable tiered population state and a real logical
multi-region navigation consumer. The courier changes region through the same
command ledger and six-owner save/replay closure used by the application. The
path remains fully offline and needs no `ai-host`, learned model or optional
navigation adapter.

R4b does not close R4. It supplies the substrate for R4c cognition and R4d
systemic work/currency/trade/food behavior, but does not implement beliefs,
GOAP, physical path following, bulk time, generic resource scheduling or
systemic economy.

## Product checks and evidence

| Check | Current signal |
|---|---|
| `fast` | Canonical catalog/snapshot/command/event codecs, 100-record and 16/32/52 closure, route tie-breaking, stale route/publication rollback, exact five-kind/two-system determinism bundle and active-transfer rejection pass |
| `play` / reference scenario | The courier emits exactly seven population events under one identity and ends revision 7, `Dormant`, at the frontier goal; existing quest/routine behavior remains exact |
| `content-package` | V4/V5 cooker/publication/activation produces exactly 30 root IDs, 116 entries, four regions and 64 chunks; corrupt/missing closure fails before publication |
| `persistence-replay` | Save/load and Replay V7 preserve the population snapshot, six descriptors and application root; a calendar-valid population snapshot without its ledger receipt fails before publication |
| `r4-100npc.v1` report | Exact measured counts are active `53,335`, near `21,339`, background `8,670`, total queries `83,344`, maximum queue depth `16`, and zero deferred/dropped/starved work. Due/population/application/ledger roots are stable |

The 2026-08-16 local release report is intentionally not hard evidence. The
host does not match the THOTH fingerprint, so outer status is `NOT_RUN` with
`PERF_TARGET_FINGERPRINT_UNSUPPORTED_HOST`. Its navigation p50/p95/p99 was
`2,716/4,305/4,875 us` and integrated World Services tick p50/p95/p99 was
`11,079/12,646/13,249 us`; these exceed ADR-016 targets and therefore cannot be
promoted to `PASS`, hidden by a relaxed budget or used to close B-12.

## Alternatives considered

- Active logical teleport — rejected because it invents a physical traversal
  outcome and creates two pose authorities.
- Recast/Detour public types — rejected because vendor identities are not
  stable content/save/replay contracts; a later adapter must reproduce the
  engine-owned query result.
- Camera-distance tier selection — rejected because presentation state cannot
  mutate authoritative simulation cadence.
- Generic scheduler, async query graph or persisted timing wheel — rejected
  because the fixed 100-record scan is bounded and no second accepted consumer
  needs a shared framework.
- Opaque placement/residency hashes — rejected because they cannot prove the
  record, node, tier or transfer closure used by the production consumer.
- Hard performance promotion from one unsupported host report — rejected by
  ADR-036/063 evidence authority.

## Consequences and supersession

- SPEC-08/20/25 current sections and routing/traceability now include the
  bounded population/navigation contracts; their broader navmesh, spatial
  query, reservation, bulk-time and cognition sections remain Proposed.
- ADR-052 remains the authority for the unchanged derived calendar and routine
  behavior. Its Replay V6 and V3/V4 project designations are historical R4a
  boundaries superseded only by the current R4b successors here.
- ADR-021's durable identity, single-owner, deterministic cadence and
  no-fabricated-outcome invariants are realized for this slice; its removed
  migration/bulk-first representation is not restored.
- ADR-046 remains in force: these V1/V4/V5/V7 alpha contracts are current-only
  and no migration surface is implied.

## Rollback

Remove the population/navigation catalogs, owner, command kind/system,
sixth segment, Replay V7 and `r4-100npc` dispatch as one product increment and
restore R4a current formats. Do not retain orphan schemas, accept a population
snapshot without ledger closure, or relabel the report-only measurement as a
gate result.
