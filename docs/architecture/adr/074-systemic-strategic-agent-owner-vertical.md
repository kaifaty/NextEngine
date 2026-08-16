# ADR-074: Systemic Strategic Agent owner vertical

| Field | Value |
|---|---|
| ID | ADR-074 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-16 |
| Last verified | 2026-08-16 |
| Normative dependencies | [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-08](../08-audio-navigation-and-world-services.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-19](../19-rpg-domain-and-narrative-state.md), [SPEC-20](../20-world-simulation-and-population-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](../22-schema-registry-compatibility-and-migration.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](../25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-32](../32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-020](020-rpg-domain-authority-and-extension-boundary.md), [ADR-021](021-deterministic-population-residency-and-time-advance.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](048-direct-exact-project-lock.md), [ADR-052](052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-056](056-deterministic-strategic-agent-and-belief-driven-goap.md), [ADR-072](072-deterministic-population-tier-and-graph-navigation-vertical.md), [ADR-073](073-deterministic-cognition-owner-vertical.md) |
| Supersedes | SPEC-32 clauses that describe the bounded R4d social/work/economy consumer as Proposed; ADR-073 current-only authoring V5, activation V6 and Replay V8 designations; ADR-073 epistemic, deterministic-planning and separate-owner invariants are retained |
| Superseded by | none |

## Context

ADR-073 completed the deterministic R4c cognition owner vertical but stopped
before any social or domain outcome. R4d requires one concrete production NPC
that begins without food or currency, learns of a real job, accepts it,
reaches and completes the activity, receives committed wages, buys and
consumes food, and survives a threat interruption. Every visible result must
come from the ordinary Agent, Memory, World Services and RPG owners rather
than a test fixture or a monolithic NPC state.

ADR-046 forbids speculative frameworks and alpha compatibility scaffolding.
The qualifying consumer therefore admits only the contracts used by this
scenario, its typed failure branches, exact save/replay closure, bounded bulk
time and four-tier 100-NPC cadence evidence. Broader bargaining, jobs markets,
taxes, crime, factions, macro-economy, episodic-memory breadth and learned
strategic/tactical policies remain outside this decision.

## Decision

1. `SpeechActKindV1` is the closed 15-kind semantic taxonomy: `Inform`, `Ask`,
   `Request`, `Offer`, `CounterOffer`, `Accept`, `Reject`, `Promise`, `Warn`,
   `Threaten`, `Thank`, `Apologize`, `Insult`, `Praise` and `Gossip`.
   `StructuredSpeechActV1` binds stable participants, topic, optional bounded
   `SpeechClaimV1`, response/reply identity, creation and expiry ticks.
   `SpeechClaimV1` carries confidence and bounded cited-belief provenance; it
   has no `is_lie`, verified-truth or hidden-world flag.
2. The current production proof uses exactly one authored
   `Ask -> Inform -> Offer -> Accept` work exchange and one later `Threaten`
   act. The other taxonomy values are validated contract breadth, not evidence
   of shipped bargaining or social-memory behavior. An accepted act remains a
   proposal until the relevant domain owner commits it; wording, LLM, ASR and
   TTS are optional presentation/interpretation paths and are absent from
   correctness.
3. RPG Framework adds the twelfth `RpgAggregatePayloadV1` kind,
   `CommitmentPayloadV1`. It owns issuer, recipient, work, workplace, currency,
   wage and the closed `Offered -> Accepted -> Fulfilled` path with
   `Cancelled` from Offered or Accepted. The ninth closed RPG operation is
   `TransitionCommitment`. Only a validated RPG command changes commitment
   state; Agent or speech data cannot do so directly.
4. `WorldActivityCatalogV1` is one required content root binding the existing
   worker, commitment, workplace, duration and exact systemic-work profile.
   `WorldActivitySnapshotV1` is the separate World Services authority for the
   closed sequence `Unassigned -> Assigned -> Working -> Completed`.
   Transitions require exact revisions and respectively
   `AcceptedCommitment`, `WorkplacePresence` and `ElapsedWork` evidence.
   `WorldActivityCommandV1` and `WorldActivityChangedV1` use the ordinary
   command/event boundary; activity never writes RPG or physical pose.
5. The R4c planner and private executive retain their epistemic and bounded
   GOAP rules. R4d adds only `CommitSocialExchange`, `AwaitActivity` and
   `SettleSystemicExchange` intent kinds plus production-backed systemic
   affordance executions. Hidden owner state is revalidated at execution.
   Planning can report route/job unavailability, while owner execution uses
   the closed `IntentStale`, `JobUnavailable`, `ActivityIncomplete`,
   `InsufficientCurrency` and `InventoryUnavailable` failures. Every rejected
   branch publishes no partial RPG event, aggregate, ledger or activity write.
6. Work acceptance is a one-operation RPG transaction from Offered to
   Accepted. After exact activity completion, settlement is one atomic
   nine-operation RPG transaction: employer wage debit, worker wage credit,
   worker food-price debit, seller credit, seller-to-worker item transfer,
   item consumption, hunger reduction, satiety increase and commitment
   fulfillment. A stale revision, absent job, unavailable route/activity,
   insufficient currency or invalid inventory/resource bound rejects the
   complete transaction.
7. The authored threat enters the existing emergency priority band. It
   suspends the ordinary systemic goal and later produces an explicit resume
   or bounded replan; it does not erase the prior goal or mutate another owner
   from the Agent layer.
8. `core_r4d` retains the twelve fixed Runtime stages. The command-kind
   registry has seven entries. The schedule has four systems: the new
   single-shard World Activity boundary runs after cognition at
   `AgentPlanning`, stages priority-280 internal Outcome work, and participates
   in the same final preflight, archive, receipt and atomic owner publication.
   Runtime, streaming, routine, population, activity, Agent and Memory
   generations publish together or none does.
9. Tier cognition is exact scheduling evidence over the existing 100-record
   population. Due Active, Simulated, Abstract and Dormant records map to
   `FullEvaluation`, `ReducedEvaluation`, `AbstractMaintenanceOnly` and
   `DormantWakeCheckOnly`. The report is canonically ordered, bounded to 100
   work items and always records zero fabricated outcomes. It does not claim
   that Abstract/Dormant work performed traversal, trade, combat or quests.
10. `ReferenceGameDriverV2::advance_bulk_time` is the first bounded bulk-time
    consumer. One request admits `1..=4096` ticks and executes every tick
    through the ordinary prepare/validate/commit path, stopping immediately
    after the first authoritative or presentation-observable boundary or at
    budget exhaustion. Zero/oversized requests fail before mutation. Stepped
    and bulk runs compare complete state, command/event counts, owner
    revisions, checkpoint root and application root at every returned
    boundary.
11. Project source/cook advance to V6 and activation to
    `ActivatedProjectV7`. The activity catalog changes reference-alpha roots
    `31 -> 32` and entries `117 -> 118`; the four-region/64-chunk and 100-record
    population shapes are unchanged. Activation requires the exact activity,
    systemic profile, commitment, population/navigation, cognition and RPG
    bootstrap closure before publication.
12. The application/save root adds the full-tuple activity segment:

    ```text
    nextengine.world-services / nextengine.world-activity-snapshot / world-activity / 1
    ```

    Reference alpha therefore publishes nine descriptors: Runtime, RPG,
    Physics, streaming, routine, population, activity, Agent and Memory.
    Current-only `ReplayManifestV9` requires the eight non-routine owners,
    permits only the routine slot to be absent, replays activity/cognition/RPG
    commands through ordinary batches and compares all sorted descriptors plus
    the application and ledger roots. V8 and earlier are typed unsupported;
    `SaveManifestV2` and `WorldCheckpointV4` retain their generic wire shapes.

## Product impact

The reference 32-tick production scenario now commits five structured speech
acts, three activity transitions and the accepted/fulfilled commitment plus
the atomic work/currency/trade/food settlement. It finishes with 52 domain
events, 23 RPG events, four paired Agent/Memory revisions and four Decision
Traces at ticks 1, 4, 5 and 6. Threat interruption/resume and stale, no-route,
no-job and no-money branches are typed and non-partial.

This closes the bounded deterministic R4d product increment without an LLM or
learned policy. It does not implement generic scheduling, a jobs marketplace,
macroeconomics, physical corridor following, broad social/episodic memory or
optional R8 learned behavior. It also does not close B-12, paired
Windows/Linux evidence or v1 shipping.

## Relevant product checks

| Check | Scenario | Result / required interpretation |
|---|---|---|
| focused contracts/Agent/World/Runtime/reference tests | Canonical speech/claim/commitment/activity round trips; happy path; stale/no-route/no-job/no-money; threat resume; exact tier dispatch; bulk/stepped equivalence | Passing branches use production owners; every failure retains prior roots and publishes no partial result |
| `play` | Run the current 32-tick reference scenario through V6 cook, V7 activation and production Runtime | PASS: 52 events, 23 RPG events, five speech acts, activity revision 3, four paired cognition revisions/traces and completed systemic settlement |
| `persistence-replay` | Save/load/restart and replay the systemic world | PASS: 19 ticks, two generations, 18 RPG events, nine descriptors and exact activity/Agent/Memory/application/ledger continuation; V8 rejects before projection |
| `content-package` | Cook/activate the valid project and malformed activity/profile/owner variants | PASS: 32 roots, 118 entries and 64 chunks publish as one `ActivatedProjectV7`; malformed candidates publish nothing |
| `host-check` | Format, strict Clippy, workspace/integration/doc tests and repository boundary scan on `x86_64-unknown-linux-gnu` | PASS with Rust 1.97.1 |
| `performance --scenario r4-100npc --mode report` | 1,000 warm-up + 10,000 measured production ticks over exact population/navigation/tier-dispatch work | Workload completed with 83,344 navigation and cognition work items, queue depth 16, zero defer/drop/starvation/fabricated outcomes and exact roots. Outer verdict is `NOT_RUN(PERF_TARGET_FINGERPRINT_UNSUPPORTED_HOST)`. Local navigation p95/p99 `4292/4870 us` and integrated p95/p99 `15618/16300 us` exceed their report-only `1250/1500` and `8000/12000 us` rows; cognition dispatch `759/838 us` is within its row. This is diagnostic evidence, not B-12 PASS. |

## Alternatives considered

- A monolithic NPC/economy snapshot was rejected because Agent, Memory, World
  Services and RPG own different mutable facts.
- A generic jobs market, scheduler or resource framework was rejected because
  one consumer does not justify a second runtime architecture.
- Quest/dialogue state as an implicit work promise was rejected because a
  commitment is a distinct RPG-owned transaction invariant.
- Direct currency/item/resource mutation from Agent or activity code was
  rejected because it would bypass RPG validation and atomicity.
- Skipped-tick bulk simulation was rejected because it could cross an
  observable command boundary without producing the ordinary evidence.
- Fabricated Abstract outcomes were rejected; unsupported work remains
  maintenance/defer evidence only.
- An LLM or learned policy dependency was rejected because the complete
  deterministic ADR-056 path is mandatory and sufficient.

## Consequences

- SPEC-02/03/06/08/09/12/13/15/17/19/20/21/22/24/25/32, the architecture
  index, routing, traceability, glossary and roadmap describe R4d as current.
- Current pre-v1 names are authoring/source/cook V6, activation V7 and Replay
  V9. ADR-046 still supplies no migration promise.
- RPG now has twelve aggregate payload kinds and nine operation variants.
  World Services has one additional separately persisted activity owner.
- The R4 workload now includes tier-cognition timing/counters/roots, but its
  unsupported-host report and failed local absolute rows remain non-gating.
- Advanced social/economy breadth and learned strategic/tactical behavior stay
  future work; their absence does not weaken this deterministic vertical.

## Supersession

A successor must preserve or explicitly replace the epistemic boundary,
separate owner authority, typed non-partial failures, ordinary command/ledger
path, no-fabrication tier rule and stepped/bulk boundary equivalence. Rollback
removes activity content/system/command/segment, systemic intent executions,
commitment extension and Replay V9 together and restores the complete ADR-073
R4c profile; partially readable owner roots or orphan schemas are forbidden.
