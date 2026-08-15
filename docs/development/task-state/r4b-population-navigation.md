# R4b population tiers and graph navigation — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-16 |
| Task key | `r4b-population-navigation` |
| Scope | One typed 100-NPC population, four-tier courier lifecycle and deterministic two-region graph transfer through authoring, runtime, save, replay and report-only performance |
| Definition of done | The production reference consumer passes the R4b ProductChecks; SPEC-08/20/25 and the new R4b ADR are promoted in the same changeset without a B-12 or full-R4 claim |
| Authority | Working context only; Accepted SPEC/ADR, checked-in contracts and `docs/roadmap.md` outrank this file |

## Resume in 60 seconds

- **Current conclusion:** The bounded R4b cut is implemented and promoted: two domain-relevant content roots, one World Services population owner and one engine-owned graph query are exercised by the exact 100-NPC consumer.
- **Why:** The reference courier completes all seven revisions under one PersistentId; V4/V5 project activation, six-owner save/load, Replay V7 and the report-only workload all pass their mapped correctness checks.
- **Next action:** Start the R4c cognition vertical on this immutable population/navigation substrate; do not widen R4b into a scheduler or physical path follower.
- **Current blocker:** None.
- **Do not retry:** Opaque placement/residency hashes, camera-distance tier authority, direct active-NPC teleport, generic scheduler/job framework, or a hard performance claim from the report-only workload.
- **Reconsider when:** A later R4c consumer demonstrates that the current immutable views are insufficient while preserving ADR-072 ownership and replay closure.

## Product outcome

The reference alpha contains exactly 100 stable NPC PersistentIds. Sixteen use
the 3-tick active cadence, 32 the 15-tick near cadence and 52 the 60-tick
background cadence; each phase is derived by ADR-016. One background courier
starts Dormant in the relay-station region, becomes Abstract, commits one
validated abstract graph transfer to the frontier region, becomes Simulated
then Active, and returns through the same tiers to Dormant. All changes use
Outcome commands and the joint Runtime + World Services transaction.

The navigation baseline is an engine-owned, content-revision-bound graph with
one node per authored world chunk and deterministic non-negative-cost routing.
An Abstract transfer may commit only against the exact route-plan hash and
catalog revision. Active transfer returns `PHYSICAL_TRAVERSAL_REQUIRED` and
does not mutate logical placement or physical pose.

## Required contract cut

1. `WorldPopulationCatalogV1` with exactly 100 sorted records, fixed cadence
   classes/phases, stable initial placement, home/goal chunk and initial tier.
2. `WorldNavigationCatalogV1`, typed query/result and stable shortest-path
   tie-breaking over chunk-bound graph nodes/tiles.
3. `WorldPopulationSnapshotV1`, tier/transfer command, event and canonical
   owner segment under `nextengine.world-services`.
4. Current-only `ProjectAuthoringManifestV4 -> NeutralProjectSourceV4 ->
   CookedProjectV4 -> ActivatedProjectV5`, with roots `28 -> 30` and content
   entries `114 -> 116`; previous alpha public names are retired.
5. `core_r4b` registry/schedule/determinism bundle with a fifth Outcome-only
   command kind and a population system in the existing fixed stage order.
6. Joint Runtime + routine + population + optional streaming prepare,
   validate, final-preflight and no-fail commit.
7. Six-owner application/save closure and current-only Replay V7.
8. `r4-100npc.v1` report-only workload with 1,000 warm-up and 10,000 measured
   ticks, exact no-starvation counts/roots and ADR-016 navigation/World
   Services metrics; no hard gate or B-12 claim.

## Invariants

- World Services owns durable population membership, logical region/chunk,
  tier and cadence; Runtime owns only command/receipt state; Physics remains
  the sole traversal/pose authority.
- Tier and physical LOD are separate. Camera distance, wall time and worker
  completion order cannot select tier, due work, route or transfer outcome.
- The same courier PersistentId survives every tier and both regions; no
  despawn/recreate identity swap is permitted.
- Every tier/placement mutation is `Prepare -> Validate -> Commit -> Stabilize`;
  stale route, stale revision, unavailable node/tile or active traversal need
  leaves every owner unchanged.
- Due order is `(service_stage, cadence_class, PersistentId)` and cadence is
  `active=3`, `near=15`, `background=60` with the ADR-016 SHA-256 phase.
- Routing uses integer costs, sorted node/edge identity and explicit graph
  revision. No raw vendor poly refs or Recast/Detour types enter public state.
- The existing 12 stages are retained. R4b adds no general-purpose scheduler,
  resource framework, cognition, bulk time advance, trade, perception or
  learned model.

## Execution plan

1. [completed] Canonical population/navigation contracts, codecs and failure tests.
2. [completed] Current-only authoring/cook/activation V4/V5 and reference content.
3. [completed] World Services population owner, routing, tier/transfer staging and joint commit.
4. [completed] Six-owner save/load and current-only Replay V7 parity.
5. [completed] Reference courier branch and exact 100-NPC production checks.
6. [completed] `r4-100npc` report-only workload and fixed-count/permutation checks.
7. [completed] Format/check/clippy, `play`, `persistence-replay`, `content-package`, boundary/host checks.
8. [completed] Promote ADR-072, the SPEC delta, routing/traceability and roadmap without a B-12 or full-R4 claim.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `cargo run -p xtask -- play` | `PASS` | 32 ticks, 35 events and seven population revisions preserve the reference scenario and one courier identity |
| `cargo run -p xtask -- persistence-replay` | `PASS` | 19-tick save/load/Replay V7 path preserves six-owner closure; final state root `8ed134d5…66e4`, ledger root `98c5efa1…070d` |
| `cargo run -p xtask -- content-package` | `PASS` | Current V4/V5 package contains exactly 116 records and 64 chunks with the population/navigation roots |
| `cargo run -p xtask -- host-check` | `PASS` | fmt, clippy `-D warnings`, complete workspace tests and boundary scan pass on Rust 1.97.1 Linux |
| Focused live-runtime smoke | `PASS` | 900 ticks produce exactly 908 command bodies after adding one routine and seven population commands |
| Release `r4-100npc.v1` local report | `NOT_RUN` hard verdict / report-only data | Exact 53,335 active, 21,339 near, 8,670 background due records, 83,344 queries, max queue 16 and zero defer/drop/starvation; host fingerprint is unsupported |
| ADR-016 timing rows on the local host | `FAIL` as diagnostic comparison, not hard evidence | Navigation p95/p99 `4,305/4,875 us`; joint tick p95/p99 `12,646/13,249 us`; B-12 remains open and no budget is relaxed |

## Decisions that still constrain the work

### D-001 — Bounded abstract transfer, no fabricated active traversal

- **Observation:** R4b requires placement/transfer and graph navigation, while SPEC-08 and ADR-021 reserve actual active traversal outcomes for Physics.
- **Evidence:** SPEC-08 navigation handoff and ADR-021 no-fabricated-activity invariant.
- **Decision:** Commit one route-bound transfer only while the courier is Abstract; reject the same operation in Active with a typed physical-traversal requirement.
- **Rejected alternatives:** Teleport an Active subject, copy logical coordinates into Physics, or defer all transfer behavior to an unused API.
- **Consequences:** The consumer proves both the successful logical path and the no-mutation physical boundary.
- **Uncertainty:** Full active path following remains future R4/R5 work.
- **Reconsider when:** A later accepted physical locomotion consumer supplies an engine-owned route-to-intent contract.

### D-002 — Direct bounded cadence service, not a generic scheduler

- **Observation:** Exactly 100 records and three periods are already fixed by ADR-016; SPEC-23 remains Proposed.
- **Evidence:** R4 roadmap scope and the R4a guard against premature scheduler promotion.
- **Decision:** Scan the sorted 100-record catalog and execute due graph queries directly inside the population owner stage.
- **Rejected alternatives:** New job graph, async work queue, dynamic priority system or camera-driven cadence.
- **Consequences:** Complexity is bounded and measurable; later scheduler work must preserve these exact semantics.
- **Uncertainty:** Scaling beyond the fixed workload is not claimed.
- **Reconsider when:** A second accepted consumer demonstrates shared scheduling requirements.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: one deterministic Dijkstra query per due NPC fits ADR-016 budgets | Exact bounded workload completes with no deferral, drop or starvation | Unsupported-host report exceeds both navigation and joint-tick targets | **Not supported by current timing evidence.** Optimize only under a separately scoped performance task; retain the exact semantics and seek compatible THOTH evidence. |
| H2: adding the population owner to the existing joint transaction preserves rollback | Stale route, owner generation and streaming permutations all retain every owner; save/replay ledger negatives fail before publication | No contrary focused or workspace result | **Supported.** Keep the joint prepare/validate/final-preflight/no-fail commit boundary. |

## Required context

Read these sources in precedence order before acting:

1. `docs/architecture/agent-routing.md`, R4b population/navigation rows.
2. SPEC-08, SPEC-20, SPEC-25 and ADR-016/021/046.
3. SPEC-02/03/09/12/21/22/24 and ADR-022/025/030/036/049/063.
4. `docs/roadmap.md` and the completed R4a task state.

## Next action

1. Hand the Accepted ADR-072 contracts and current R4b roots to the R4c task.
2. The next production slice must consume immutable population/calendar/route
   views for deterministic beliefs and bounded GOAP while adding no hidden
   mutation or model-dependent authority.
3. Keep R4b performance as report-only until a compatible THOTH fixed-batch
   run can support a hard verdict; do not claim B-12 from the local report.

## Do not retry

- Opaque empty placement/residency hashes — they cannot prove content/runtime closure; reconsider only if an Accepted external catalog contract replaces the typed roots.
- Generic scheduler scaffolding — no second accepted consumer; reconsider only after SPEC-23 gains a production-backed decision.
- Active logical teleport — violates Physical authority; reconsider only with an Accepted physical traversal result contract.
- Hard `r4-100npc` verdict — no calibrated THOTH baseline/batch exists; reconsider only after ADR-063 evidence is produced.

## Handoff

- **Workspace state:** R4b implementation and its ADR/SPEC/roadmap promotion are complete on `codex/architecture-foundation-promotion`.
- **Checks:** `play`, `persistence-replay`, `content-package`, `host-check`, focused live-runtime smoke, workspace check/tests and boundary scan pass.
- **Remaining risk:** current direct Dijkstra scan exceeds ADR-016 timing targets on the unsupported local host; B-12 and hard R4 performance evidence remain open.
- **Promotion needed:** none for R4b. The next roadmap increment is R4c cognition, with its own production-backed ADR/task state.
