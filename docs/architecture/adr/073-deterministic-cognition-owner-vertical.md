# ADR-073: Deterministic cognition owner vertical

| Field | Value |
|---|---|
| ID | ADR-073 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-16 |
| Last verified | 2026-08-16 |
| Normative dependencies | [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-08](../08-audio-navigation-and-world-services.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-20](../20-world-simulation-and-population-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](../22-schema-registry-compatibility-and-migration.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-32](../32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [ADR-005](005-offline-first-ai-process-boundary.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-020](020-rpg-domain-authority-and-extension-boundary.md), [ADR-021](021-deterministic-population-residency-and-time-advance.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](048-direct-exact-project-lock.md), [ADR-052](052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-056](056-deterministic-strategic-agent-and-belief-driven-goap.md), [ADR-072](072-deterministic-population-tier-and-graph-navigation-vertical.md) |
| Supersedes | Clauses that describe the complete R4c cognition core as Proposed; current-only authoring V4, activation V5 and Replay V7 designations are replaced by the bounded successors below |
| Superseded by | [ADR-074](074-systemic-strategic-agent-owner-vertical.md) for the R4d systemic extension and current-only V6/V7/Replay V9 designations; R4c epistemic, planning and separate-owner invariants remain current |

## Context

R4b established one immutable 100-record population, stable logical placement,
integer cadence and an engine-owned graph, but deliberately performed no
cognition. ADR-056 already accepted the strategic invariant: an offline
deterministic Utility selector and bounded belief-driven GOAP planner must be a
complete fallback without an LLM or learned policy. ADR-046 still forbids
shipping speculative schemas before a production consumer exists.

The smallest qualifying R4c product slice is one existing population subject
whose authored beliefs, own RPG health and World Services location/route are
consumed through the ordinary fixed-stage Runtime. It must prove canonical goal
selection, bounded planning, emergency interruption/resume, separate owner
persistence and exact replay without beginning the R4d social/economy scenario.
That consumer and its fail-closed checks now exist.

## Decision

1. `AgentCognitionCatalogV1` is one required domain-relevant root asset. It
   binds exactly one existing `WorldPopulationCatalogV1` subject, integer
   evaluation start/period, bounded belief retrieval, Q16 switch threshold,
   emergency health threshold, GOAP depth/node limits, stable goal/action/fact
   IDs and sorted authored seed beliefs. Reference alpha evaluates at ticks
   `1 + 3n`, retrieves at most four beliefs and limits planning to depth `2`
   and eight expanded nodes.
2. The stage-7 `EpistemicViewV1` contains only the catalog revision, retrieved
   Memory beliefs, the subject's own revision-bound RPG health, its population
   record/location and the engine-owned revision-bound route result. Hidden
   aggregates, presentation visibility, camera state, wall time and arbitrary
   world truth are not planner inputs. A route-query failure removes the route
   affordance and cannot be converted into fabricated knowledge.
3. `AgentMemorySnapshotV1` is the separate Memory Service authority for the
   current bounded semantic belief set, provenance, confidence, contradiction,
   revision and last retrieval tick. R4c admits authored-seed and owner-
   projection provenance tags plus deterministic bounded retrieval; episodic
   records, social claims and consolidation remain R4d/future scope.
4. `DriveViewV1` is derived, not persisted needs authority. Candidate goals use
   bounded Q16 components, closed priority bands and canonical ordering. Goal
   inertia applies the authored switch threshold; an emergency band suspends
   the ordinary goal in a bounded stack and exiting emergency performs explicit
   `EmergencyExitResume` rather than silently replacing history.
5. The planner is deterministic bounded GOAP over read-only
   `SemanticAffordanceV1` values. R4c has exactly the logical-route request and
   hold-position execution kinds. Expansion/depth exhaustion or a missing
   affordance emits a typed `PlanningFailureV1` and no owner proposal; it never
   commits a partial plan or mutates RPG/World state.
6. `AgentCognitionSnapshotV1` is the Agent Runtime authority for active and
   suspended goals, plan/cursor, private task lifecycle, pending typed intent,
   hysteresis, decision RNG state and last epistemic hash. The private
   executive emits only `StrategicAgentIntentV1`; the R4c consumer does not
   claim route traversal or another domain outcome. `DecisionTraceV1` is a
   reproducible bounded diagnostic projection and is neither persisted nor an
   input to gameplay.
7. `ScheduleManifestV1::core_r4c` retains the twelve accepted stages and adds
   one single-shard cognition system at `AgentPlanning` (stage 7). It reads
   immutable Agent, Memory, RPG and World Services projections, then stages
   `nextengine.command.agent-cognition` V1 for the existing internal Outcome
   barrier at stage 9. The command uses the dedicated internal principal,
   stream, capability and priority `270`; event subscribers cannot re-enter the
   same tick.
8. Runtime, routine, population, cognition and optional streaming are one
   prepared/validated/final-preflight publication. The cognition command emits
   `nextengine.event.agent-decision-committed` only after validation. Agent and
   Memory revisions advance together or neither owner publishes; a stale base,
   catalog mismatch or impossible proposal aborts before any partial ledger or
   owner mutation.
9. The application/save root adds two full-tuple segments:

   ```text
   nextengine.agent-runtime / nextengine.agent-cognition-snapshot / agent-cognition / 1
   nextengine.memory-service / nextengine.agent-memory-snapshot / agent-memory / 1
   ```

   Reference alpha therefore has eight segments: Runtime, RPG, Physics,
   streaming, routine, population, Agent and Memory. `SaveManifestV2` and
   `WorldCheckpointV4` retain their generic wire shapes. Current-only
   `ReplayManifestV8` requires population, Agent and Memory, permits only the
   already optional routine slot, replays cognition commands/events through the
   ordinary Outcome vectors and compares the eight sorted descriptors plus the
   application root. V7 and earlier are typed unsupported formats, not
   migration readers.
10. Project authoring/source/cook advance to V5 and activation to V6. The
    cognition catalog changes reference-alpha roots `30 -> 31` and content
    entries `116 -> 117`; the four-region/64-chunk and 100-record population
    shapes are unchanged. Activation requires the exact cognition root and its
    bound population/RPG/bootstrap closure before atomic publication.
11. The initial decision RNG state is derived from the immutable world RNG root
    and cognition subject under a versioned domain. Every future-affecting
    value is persisted; planner queues, caches, traces and optional inference
    buffers remain reconstructible. No `ai-host`, network, model artifact or
    GPU path participates in correctness.

## Product impact

Reference alpha now runs one complete offline Strategic Agent cognition loop
through production content, Runtime command admission, joint owner publication,
save/load and replay. The ordinary scenario produces eleven canonical
decisions in 32 ticks. A focused production branch lowers the subject's health,
observes `EmergencyInterrupt`, restores health and observes
`EmergencyExitResume` on the next authored evaluation boundaries.

This decision completes R4c only. It does not implement structured speech acts,
trust updates, commitments, work, currency, trade, food, physical route
following, generic scheduling, episodic/social memory or learned behavior.
Those remain R4d or optional R8 work and cannot infer an R4, B-12 or v1 shipping
claim.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| focused cognition tests | Repeat equal epistemic inputs; remove an affordance; reduce planner budget; vary hidden authority | Canonical decision equality; typed missing-affordance/budget failures produce no proposal; hidden facts change no view or decision | Safe no-proposal result with no owner mutation |
| production emergency branch | Run stage 7/9 at ticks `1`, `4`, `7` with RPG health `100 -> 20 -> 100` | Ordinary goal, exact emergency interrupt/suspension, then exact resume with paired Agent/Memory revisions | Abort the joint tick on invalid command/owner closure |
| `play` | Run the 32-tick reference scenario through cooker, activation and production Runtime | Eleven cognition events/traces, Agent/Memory revision 11, zero planning failures and unchanged RPG gameplay correctness | Typed activation/runtime failure before partial publication |
| `persistence-replay` | Save/load/restart and replay the joint world with cognition enabled | Eight descriptors, Agent/Memory bytes, cognition commands/events, application and ledger roots match uninterrupted execution | Reject missing/corrupt/retired closure before publication |
| `content-package` | Cook/activate authoring V5 and inject missing/corrupt cognition/profile bindings | Exactly 31 roots and 117 entries publish as one `ActivatedProjectV6`; every malformed candidate publishes nothing | Preserve the previous project/package generation |
| conditional `performance --scenario r4-100npc --mode report` | Run the retained population workload with the R4c determinism profile | Exact due/query/no-starvation and authoritative roots remain stable; timing stays report-only on an unsupported host | `NOT_RUN`, never a B-12 pass |

## Alternatives considered

- A test-only planner was rejected because it would not exercise content,
  command admission, owner publication, save or replay.
- One monolithic NPC blob was rejected because Agent Runtime, Memory Service,
  RPG and World Services own different mutable facts.
- Direct world-truth planning was rejected because it breaks the epistemic
  boundary and makes hidden information affect decisions.
- A public mutable task trait or generic scheduler was rejected because the
  one bounded private executive needs neither shared mutable context nor a new
  scheduling framework.
- Executing navigation, work or economy outcomes in R4c was rejected because
  it would fabricate unimplemented owner results and collapse the R4c/R4d
  boundary.
- Learned goal or tactical policies were rejected as an R4 dependency; they
  remain optional R8 adapters behind the complete deterministic route.

## Consequences

- SPEC-02/03/06/17/20/21/22/24/32, the architecture index, routing,
  traceability, glossary and roadmap describe the R4c current contracts.
- Current pre-v1 names are V5 authoring, V6 activation and Replay V8. ADR-046
  still provides no migration promise; older ADR format names remain
  historical boundaries.
- Agent and Memory snapshots are always restored and published as a pair. A
  missing, duplicate, corrupt or revision-mismatched segment fails before
  changing any live owner.
- R4d must reuse these epistemic/goal/plan owners and add only production-
  backed social/economy semantics through their real domain owners.

## Supersession

A successor must preserve the epistemic and owner boundaries or explicitly
supersede them with production evidence. Rollback removes the cognition catalog,
system/command/event, Agent and Memory segments and Replay V8 together and
restores the complete R4b format/profile set; orphan cognition schemas or
partially readable owner roots are forbidden.
